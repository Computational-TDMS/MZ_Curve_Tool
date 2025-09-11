//! 处理器核心模块
//! 
//! 提供统一的处理器接口和工厂模式

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

use crate::core::data::{DataContainer, ProcessingResult, ProcessingError, Curve, Peak};
use crate::core::processors::peak_fitting::PeakFitter;

/// 处理器类型枚举
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum ProcessorType {
    // 数据提取器
    DTExtractor,
    TICExtractor,
    XICExtractor,
    
    // 基线校正
    BaselineCorrection,
    
    // 峰检测
    PeakDetection,
    
    // 峰拟合
    PeakFitting,
    
    // 重叠峰处理
    OverlappingPeaks,
    
    // 峰分析（组合处理器）
    PeakAnalysis,
}

/// 处理器配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessorConfig {
    pub processor_type: ProcessorType,
    pub method: String,
    pub parameters: HashMap<String, Value>,
}

impl ProcessorConfig {
    pub fn new(processor_type: ProcessorType, method: String) -> Self {
        Self {
            processor_type,
            method,
            parameters: HashMap::new(),
        }
    }
    
    pub fn with_parameter(mut self, key: String, value: Value) -> Self {
        self.parameters.insert(key, value);
        self
    }
    
    pub fn get_parameter<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.parameters.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }
    
    pub fn get_parameter_or_default<T>(&self, key: &str, default: T) -> T
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.get_parameter(key).unwrap_or(default)
    }
}

/// 统一的处理器trait
#[async_trait]
pub trait Processor: Send + Sync {
    /// 处理器名称
    fn name(&self) -> &str;
    
    /// 处理器描述
    fn description(&self) -> &str;
    
    /// 处理器类型
    fn processor_type(&self) -> ProcessorType;
    
    /// 支持的算法方法
    fn supported_methods(&self) -> Vec<String>;
    
    /// 配置模式
    fn config_schema(&self) -> Value;
    
    /// 处理数据
    async fn process(
        &self,
        input: DataContainer,
        config: serde_json::Value,
    ) -> Result<ProcessingResult, ProcessingError>;
    
    /// 验证配置
    fn validate_config(&self, config: &ProcessorConfig) -> Result<(), ProcessingError> {
        if !self.supported_methods().contains(&config.method) {
            return Err(ProcessingError::ConfigError(format!(
                "不支持的方法: {}，支持的方法: {:?}",
                config.method,
                self.supported_methods()
            )));
        }
        Ok(())
    }
}

/// 处理器工厂
pub struct ProcessorFactory;

impl ProcessorFactory {
    /// 创建处理器
    pub fn create_processor(config: ProcessorConfig) -> Result<Arc<dyn Processor>, ProcessingError> {
        match config.processor_type {
            ProcessorType::DTExtractor => {
                // 暂时返回错误，因为DTExtractor没有实现Processor trait
                Err(ProcessingError::ConfigError("DTExtractor暂时不可用".to_string()))
            },
            ProcessorType::TICExtractor => {
                // 暂时返回错误，因为TICExtractor没有实现Processor trait
                Err(ProcessingError::ConfigError("TICExtractor暂时不可用".to_string()))
            },
            ProcessorType::XICExtractor => {
                // 暂时返回错误，因为XICExtractor没有实现Processor trait
                Err(ProcessingError::ConfigError("XICExtractor暂时不可用".to_string()))
            },
            ProcessorType::BaselineCorrection => {
                // 暂时返回错误，因为BaselineProcessor没有实现Processor trait
                Err(ProcessingError::ConfigError("BaselineProcessor暂时不可用".to_string()))
            },
            ProcessorType::PeakDetection => {
                let detector = crate::core::processors::peak_detection::create_detector(&config.method)?;
                Ok(std::sync::Arc::new(detector))
            },
            ProcessorType::PeakFitting => {
                // 创建峰拟合器
                let fitter_enum = crate::core::processors::peak_fitting::create_fitter(&config.method)?;
                Ok(std::sync::Arc::new(PeakFittingProcessor::new(fitter_enum)))
            },
            ProcessorType::OverlappingPeaks => {
                let processor = crate::core::processors::overlapping_peaks::create_overlapping_processor(&config.method)?;
                Ok(std::sync::Arc::new(processor))
            },
            ProcessorType::PeakAnalysis => {
                crate::core::processors::peak_analysis::PeakAnalyzer::create(&config.method)
            },
        }
    }
    
    /// 获取所有支持的处理器类型
    pub fn get_supported_types() -> Vec<ProcessorType> {
        vec![
            ProcessorType::DTExtractor,
            ProcessorType::TICExtractor,
            ProcessorType::XICExtractor,
            ProcessorType::BaselineCorrection,
            ProcessorType::PeakDetection,
            ProcessorType::PeakFitting,
            ProcessorType::OverlappingPeaks,
            ProcessorType::PeakAnalysis,
        ]
    }
}

/// 处理器链
pub struct ProcessorChain {
    processors: Vec<Arc<dyn Processor>>,
    configs: Vec<ProcessorConfig>,
}

impl ProcessorChain {
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
            configs: Vec::new(),
        }
    }
    
    /// 添加处理器
    pub fn add_processor(mut self, config: ProcessorConfig) -> Result<Self, ProcessingError> {
        let processor = ProcessorFactory::create_processor(config.clone())?;
        processor.validate_config(&config)?;
        
        self.processors.push(processor);
        self.configs.push(config);
        Ok(self)
    }
    
    /// 执行处理链
    pub async fn execute(&self, mut input: DataContainer) -> Result<ProcessingResult, ProcessingError> {
        let mut result = ProcessingResult {
            curves: input.curves.clone(),
            peaks: vec![], // peaks现在在curves中
            metadata: HashMap::new(),
        };
        
        for (processor, config) in self.processors.iter().zip(self.configs.iter()) {
            let input_container = DataContainer {
                curves: result.curves.clone(),
                metadata: result.metadata.clone(),
                spectra: vec![], // 空的spectra
            };
            
            result = processor.process(input_container, serde_json::to_value(&config)?).await?;
        }
        
        Ok(result)
    }
}

/// 智能处理器选择器
pub struct SmartProcessorSelector;

impl SmartProcessorSelector {
    /// 根据数据特征自动选择最佳处理器配置
    pub fn select_optimal_config(
        data: &DataContainer,
        processor_type: ProcessorType,
    ) -> Result<ProcessorConfig, ProcessingError> {
        match processor_type {
            ProcessorType::PeakDetection => {
                Self::select_peak_detection_config(data)
            },
            ProcessorType::PeakFitting => {
                Self::select_peak_fitting_config(data)
            },
            ProcessorType::OverlappingPeaks => {
                Self::select_overlapping_peaks_config(data)
            },
            _ => {
                // 对于其他类型，使用默认配置
                Ok(ProcessorConfig::new(processor_type, "default".to_string()))
            }
        }
    }
    
    /// 选择峰检测配置
    fn select_peak_detection_config(data: &DataContainer) -> Result<ProcessorConfig, ProcessingError> {
        // 分析数据特征
        let noise_level = Self::estimate_noise_level(data);
        let peak_density = Self::estimate_peak_density(data);
        
        let method = if noise_level > 0.1 {
            "cwt" // 高噪声使用CWT
        } else if peak_density > 0.5 {
            "peak_finder" // 高密度使用峰查找器
        } else {
            "simple" // 低噪声低密度使用简单方法
        };
        
        Ok(ProcessorConfig::new(ProcessorType::PeakDetection, method.to_string())
            .with_parameter("sensitivity".to_string(), Value::Number(serde_json::Number::from_f64(0.5).unwrap()))
            .with_parameter("threshold_multiplier".to_string(), Value::Number(serde_json::Number::from_f64(3.0).unwrap())))
    }
    
    /// 选择峰拟合配置
    fn select_peak_fitting_config(data: &DataContainer) -> Result<ProcessorConfig, ProcessingError> {
        // 分析峰特征
        let peak_complexity = Self::estimate_peak_complexity(data);
        
        let method = if peak_complexity > 0.7 {
            "adaptive_hybrid" // 高复杂度使用自适应混合
        } else if peak_complexity > 0.4 {
            "emg" // 中等复杂度使用EMG
        } else {
            "gaussian" // 低复杂度使用高斯
        };
        
        Ok(ProcessorConfig::new(ProcessorType::PeakFitting, method.to_string())
            .with_parameter("fitting_mode".to_string(), Value::String("adaptive".to_string())))
    }
    
    /// 选择重叠峰处理配置
    fn select_overlapping_peaks_config(data: &DataContainer) -> Result<ProcessorConfig, ProcessingError> {
        let overlap_level = Self::estimate_overlap_level(data);
        
        let method = if overlap_level > 0.8 {
            "extreme_overlap"
        } else if overlap_level > 0.5 {
            "sharpen_cwt"
        } else if overlap_level > 0.2 {
            "fbf"
        } else {
            "none"
        };
        
        Ok(ProcessorConfig::new(ProcessorType::OverlappingPeaks, method.to_string()))
    }
    
    /// 估计噪声水平
    fn estimate_noise_level(data: &DataContainer) -> f64 {
        if data.curves.is_empty() {
            return 0.0;
        }
        
        let curve = &data.curves[0];
        let mean: f64 = curve.y_values.iter().sum::<f64>() / curve.y_values.len() as f64;
        let variance: f64 = curve.y_values.iter()
            .map(|&y| (y - mean).powi(2))
            .sum::<f64>() / curve.y_values.len() as f64;
        
        variance.sqrt() / mean.max(1e-6)
    }
    
    /// 估计峰密度
    fn estimate_peak_density(data: &DataContainer) -> f64 {
        if data.curves.is_empty() {
            return 0.0;
        }
        
        let curve = &data.curves[0];
        let data_points = curve.x_values.len();
        let peak_count = curve.peaks.len();
        
        peak_count as f64 / data_points as f64
    }
    
    /// 估计峰复杂度
    fn estimate_peak_complexity(data: &DataContainer) -> f64 {
        if data.curves.is_empty() || data.curves[0].peaks.is_empty() {
            return 0.0;
        }
        
        let peaks = &data.curves[0].peaks;
        let mut complexity_sum = 0.0;
        for peak in peaks {
            // 基于峰的不对称性和拖尾计算复杂度
            let asymmetry = (peak.left_hwhm - peak.right_hwhm).abs() / (peak.left_hwhm + peak.right_hwhm).max(1e-6);
            let tailing = if peak.left_hwhm > 0.0 && peak.right_hwhm > 0.0 {
                (peak.right_hwhm / peak.left_hwhm - 1.0).abs()
            } else {
                0.0
            };
            
            complexity_sum += asymmetry + tailing;
        }
        
        complexity_sum / peaks.len() as f64
    }
    
    /// 估计重叠水平
    fn estimate_overlap_level(data: &DataContainer) -> f64 {
        if data.curves.is_empty() || data.curves[0].peaks.len() < 2 {
            return 0.0;
        }
        
        let peaks = &data.curves[0].peaks;
        let mut overlap_count = 0;
        let mut total_pairs = 0;
        
        for i in 0..peaks.len() {
            for j in (i + 1)..peaks.len() {
                let peak1 = &peaks[i];
                let peak2 = &peaks[j];
                
                let distance = (peak1.center - peak2.center).abs();
                let combined_width = (peak1.fwhm + peak2.fwhm) / 2.0;
                
                if distance < combined_width {
                    overlap_count += 1;
                }
                total_pairs += 1;
            }
        }
        
        if total_pairs > 0 {
            overlap_count as f64 / total_pairs as f64
        } else {
            0.0
        }
    }
}

/// 峰拟合处理器适配器
#[derive(Debug)]
pub struct PeakFittingProcessor {
    fitter: crate::core::processors::peak_fitting::PeakFitterEnum,
}

impl PeakFittingProcessor {
    pub fn new(fitter: crate::core::processors::peak_fitting::PeakFitterEnum) -> Self {
        Self { fitter }
    }
}

#[async_trait]
impl Processor for PeakFittingProcessor {
    fn name(&self) -> &str {
        self.fitter.name()
    }
    
    fn description(&self) -> &str {
        "峰拟合处理器"
    }
    
    fn processor_type(&self) -> ProcessorType {
        ProcessorType::PeakFitting
    }
    
    fn supported_methods(&self) -> Vec<String> {
        vec![
            "gaussian".to_string(),
            "lorentzian".to_string(),
            "emg".to_string(),
            "bi_gaussian".to_string(),
            "multi_peak".to_string(),
            "nlc".to_string(),
        ]
    }
    
    fn config_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "min_peak_width": {
                    "type": "number",
                    "default": 0.1
                },
                "max_peak_width": {
                    "type": "number", 
                    "default": 10.0
                }
            }
        })
    }
    
    async fn process(
        &self,
        input: DataContainer,
        config: serde_json::Value,
    ) -> Result<ProcessingResult, ProcessingError> {
        let mut result_peaks = Vec::new();
        
        // 对每个峰进行拟合
        for curve in &input.curves {
            for peak in &curve.peaks {
                let fitted_peak = self.fitter.fit_peak(peak, curve, &config)?;
                result_peaks.push(fitted_peak);
            }
        }
        
        Ok(ProcessingResult {
            curves: input.curves,
            peaks: result_peaks,
            metadata: input.metadata,
        })
    }
}
