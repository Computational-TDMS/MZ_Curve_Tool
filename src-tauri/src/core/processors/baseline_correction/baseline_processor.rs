use async_trait::async_trait;
use serde_json::Value;
use crate::core::data::{DataContainer, ProcessingResult, ProcessingError};
use super::{
    BaselineAlgorithm, BaselineConfig, BaselineMethod,
    LinearBaselineCorrector, PolynomialBaselineCorrector, 
    MovingAverageBaselineCorrector, AsymmetricLeastSquaresCorrector
};
use crate::core::processors::base::Processor;

/// 基线校准处理器
pub struct BaselineProcessor {
    /// 可用的基线校准算法
    algorithms: std::collections::HashMap<String, Box<dyn BaselineAlgorithm + Send + Sync>>,
}

impl BaselineProcessor {
    pub fn new() -> Self {
        let mut algorithms: std::collections::HashMap<String, Box<dyn BaselineAlgorithm + Send + Sync>> = 
            std::collections::HashMap::new();
        
        // 注册所有可用的算法
        algorithms.insert("linear".to_string(), Box::new(LinearBaselineCorrector::new()));
        algorithms.insert("polynomial".to_string(), Box::new(PolynomialBaselineCorrector::new()));
        algorithms.insert("moving_average".to_string(), Box::new(MovingAverageBaselineCorrector::new()));
        algorithms.insert("asymmetric_least_squares".to_string(), Box::new(AsymmetricLeastSquaresCorrector::new()));
        
        Self { algorithms }
    }
    
    /// 从配置创建基线配置
    fn create_baseline_config(&self, config: &Value) -> Result<BaselineConfig, ProcessingError> {
        let method_str = config.get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("linear");
        
        let method = match method_str {
            "linear" => BaselineMethod::Linear,
            "polynomial" => {
                let degree = config.get("degree")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(2) as u32;
                BaselineMethod::Polynomial { degree }
            }
            "moving_average" => {
                let window_size = config.get("window_size")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(21) as usize;
                BaselineMethod::MovingAverage { window_size }
            }
            "asymmetric_least_squares" => {
                let lambda = config.get("lambda")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let p = config.get("p")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let max_iterations = config.get("max_iterations")
                    .and_then(|v| v.as_u64())
                    .unwrap_or(100) as usize;
                BaselineMethod::AsymmetricLeastSquares { lambda, p, max_iterations }
            }
            _ => return Err(ProcessingError::ConfigError(
                format!("Unknown baseline correction method: {}", method_str)
            )),
        };
        
        let preserve_original = config.get("preserve_original")
            .and_then(|v| v.as_bool())
            .unwrap_or(true);
        
        let output_baseline = config.get("output_baseline")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        
        let mut custom_params = std::collections::HashMap::new();
        if let Some(params) = config.get("custom_params") {
            if let Some(obj) = params.as_object() {
                for (key, value) in obj {
                    custom_params.insert(key.clone(), value.clone());
                }
            }
        }
        
        Ok(BaselineConfig {
            method,
            preserve_original,
            output_baseline,
            custom_params,
        })
    }
    
    /// 选择适当的算法
    fn select_algorithm(&self, method: &BaselineMethod) -> Result<&(dyn BaselineAlgorithm + Send + Sync), ProcessingError> {
        let algorithm_name = match method {
            BaselineMethod::Linear => "linear",
            BaselineMethod::Polynomial { .. } => "polynomial",
            BaselineMethod::MovingAverage { .. } => "moving_average",
            BaselineMethod::AsymmetricLeastSquares { .. } => "asymmetric_least_squares",
            BaselineMethod::Manual { .. } => {
                return Err(ProcessingError::ProcessError(
                    "Manual baseline correction not yet implemented".to_string()
                ));
            }
        };
        
        self.algorithms.get(algorithm_name)
            .map(|alg| alg.as_ref())
            .ok_or_else(|| ProcessingError::ConfigError(
                format!("Algorithm not found: {}", algorithm_name)
            ))
    }
}

#[async_trait]
impl Processor for BaselineProcessor {
    fn name(&self) -> &str {
        "Baseline Correction Processor"
    }
    
    fn description(&self) -> &str {
        "Corrects baseline drift in mass spectrometry data using various algorithms including linear, polynomial, moving average, and asymmetric least squares methods"
    }
    
    fn config_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "method": {
                    "type": "string",
                    "enum": ["linear", "polynomial", "moving_average", "asymmetric_least_squares"],
                    "default": "linear",
                    "description": "Baseline correction method to use"
                },
                "degree": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 10,
                    "default": 2,
                    "description": "Polynomial degree (for polynomial method)"
                },
                "window_size": {
                    "type": "integer",
                    "minimum": 3,
                    "default": 21,
                    "description": "Window size for moving average (must be odd)"
                },
                "lambda": {
                    "type": "number",
                    "minimum": 0,
                    "default": 0.0,
                    "description": "Smoothing parameter for asymmetric least squares (0 for auto)"
                },
                "p": {
                    "type": "number",
                    "minimum": 0,
                    "maximum": 1,
                    "default": 0.0,
                    "description": "Asymmetry parameter for asymmetric least squares (0 for auto)"
                },
                "max_iterations": {
                    "type": "integer",
                    "minimum": 1,
                    "default": 100,
                    "description": "Maximum iterations for asymmetric least squares"
                },
                "preserve_original": {
                    "type": "boolean",
                    "default": true,
                    "description": "Whether to preserve original data"
                },
                "output_baseline": {
                    "type": "boolean",
                    "default": false,
                    "description": "Whether to output baseline curve"
                },
                "custom_params": {
                    "type": "object",
                    "description": "Custom parameters for specific algorithms"
                }
            },
            "required": ["method"]
        })
    }
    
    async fn process(
        &self,
        input: DataContainer,
        config: Value,
    ) -> Result<ProcessingResult, ProcessingError> {
        // 创建基线配置
        let baseline_config = self.create_baseline_config(&config)?;
        
        // 选择算法
        let algorithm = self.select_algorithm(&baseline_config.method)?;
        
        // 处理所有曲线
        let mut processed_curves = Vec::new();
        let mut baseline_curves = Vec::new();
        let mut processing_stats = Vec::new();
        
        for curve in &input.curves {
            // 执行基线校准
            let result = algorithm.correct_baseline(curve, &baseline_config)
                .map_err(|e| ProcessingError::ProcessError(e.to_string()))?;
            
            // 添加校准后的曲线
            processed_curves.push(result.corrected_curve);
            
            // 添加基线曲线（如果需要）
            if let Some(baseline_curve) = result.baseline_curve {
                baseline_curves.push(baseline_curve);
            }
            
            // 记录统计信息
            processing_stats.push(serde_json::to_value(result.statistics)
                .map_err(ProcessingError::SerializationError)?);
        }
        
        // 创建输出容器
        let mut output_container = input.clone();
        output_container.curves = processed_curves;
        
        // 记录基线曲线数量
        let baseline_curves_count = baseline_curves.len();
        
        // 添加基线曲线到输出
        if !baseline_curves.is_empty() {
            output_container.curves.extend(baseline_curves);
        }
        
        // 更新元数据
        output_container.metadata.insert(
            "baseline_correction_applied".to_string(),
            serde_json::Value::Bool(true)
        );
        output_container.metadata.insert(
            "baseline_correction_method".to_string(),
            serde_json::Value::String(match baseline_config.method {
                BaselineMethod::Linear => "linear".to_string(),
                BaselineMethod::Polynomial { degree } => format!("polynomial_degree_{}", degree),
                BaselineMethod::MovingAverage { window_size } => format!("moving_average_window_{}", window_size),
                BaselineMethod::AsymmetricLeastSquares { lambda, p, .. } => {
                    format!("asymmetric_least_squares_lambda_{}_p_{}", lambda, p)
                },
                BaselineMethod::Manual { .. } => "manual".to_string(),
            })
        );
        output_container.metadata.insert(
            "baseline_correction_stats".to_string(),
            serde_json::Value::Array(processing_stats.clone())
        );
        
        let mut result = ProcessingResult::new();
        result.curves = output_container.curves;
        result.peaks = output_container.peaks;
        result.metadata = output_container.metadata;
        
        // 添加处理元数据
        result.add_metadata("processor".to_string(), serde_json::Value::String(self.name().to_string()));
        result.add_metadata("method".to_string(), serde_json::Value::String(match baseline_config.method {
            BaselineMethod::Linear => "linear".to_string(),
            BaselineMethod::Polynomial { degree } => format!("polynomial_degree_{}", degree),
            BaselineMethod::MovingAverage { window_size } => format!("moving_average_window_{}", window_size),
            BaselineMethod::AsymmetricLeastSquares { lambda, p, .. } => {
                format!("asymmetric_least_squares_lambda_{}_p_{}", lambda, p)
            },
            BaselineMethod::Manual { .. } => "manual".to_string(),
        }));
        result.add_metadata("curves_processed".to_string(), serde_json::Value::Number(serde_json::Number::from(input.curves.len())));
        result.add_metadata("baseline_curves_generated".to_string(), serde_json::Value::Number(serde_json::Number::from(baseline_curves_count)));
        result.add_metadata("processing_stats".to_string(), serde_json::Value::Array(processing_stats));
        
        Ok(result)
    }
}

impl Default for BaselineProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for BaselineProcessor {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BaselineProcessor")
            .field("algorithms_count", &self.algorithms.len())
            .finish()
    }
}

/// 基线校准工厂函数
pub fn create_baseline_processor() -> BaselineProcessor {
    BaselineProcessor::new()
}

/// 快速基线校准函数
pub async fn quick_baseline_correction(
    input: DataContainer,
    method: &str,
) -> Result<ProcessingResult, ProcessingError> {
    let processor = BaselineProcessor::new();
    let config = serde_json::json!({
        "method": method,
        "preserve_original": true,
        "output_baseline": false
    });
    
    processor.process(input, config).await
}
