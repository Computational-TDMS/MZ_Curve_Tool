//! 流水线管理器
//! 
//! 负责协调各种处理器的执行，提供统一的流水线接口

use serde_json::Value;

use crate::core::data::ProcessingError;
use crate::core::processors::base::Processor;
use crate::core::data::container::SerializableDataContainer;

/// 流水线步骤类型
#[derive(Debug, Clone)]
pub enum PipelineStep {
    /// 峰检测步骤
    PeakDetection {
        method: String,
        config: Value,
    },
    /// 峰拟合步骤
    PeakFitting {
        method: String,
        config: Value,
    },
    /// 峰增强步骤
    PeakEnhancement {
        method: String,
        config: Value,
    },
    /// 曲线还原步骤
    CurveReconstruction {
        method: String,
        config: Value,
    },
    /// 基线校正步骤
    BaselineCorrection {
        method: String,
        config: Value,
    },
}

/// 流水线执行结果
#[derive(Debug, Clone)]
pub struct PipelineResult {
    pub success: bool,
    pub container: SerializableDataContainer,
    pub execution_time: u64,
    pub steps_completed: Vec<String>,
    pub error: Option<String>,
}

/// 流水线管理器
pub struct PipelineManager {
    steps: Vec<PipelineStep>,
}

impl PipelineManager {
    /// 创建新的流水线管理器
    pub fn new() -> Self {
        Self {
            steps: Vec::new(),
        }
    }
    
    /// 添加峰检测步骤
    pub fn add_peak_detection(mut self, method: &str, config: Value) -> Self {
        self.steps.push(PipelineStep::PeakDetection {
            method: method.to_string(),
            config,
        });
        self
    }
    
    /// 添加峰拟合步骤
    pub fn add_peak_fitting(mut self, method: &str, config: Value) -> Self {
        self.steps.push(PipelineStep::PeakFitting {
            method: method.to_string(),
            config,
        });
        self
    }
    
    /// 添加峰增强步骤
    pub fn add_peak_enhancement(mut self, method: &str, config: Value) -> Self {
        self.steps.push(PipelineStep::PeakEnhancement {
            method: method.to_string(),
            config,
        });
        self
    }
    
    /// 添加曲线还原步骤
    pub fn add_curve_reconstruction(mut self, method: &str, config: Value) -> Self {
        self.steps.push(PipelineStep::CurveReconstruction {
            method: method.to_string(),
            config,
        });
        self
    }
    
    /// 添加基线校正步骤
    pub fn add_baseline_correction(mut self, method: &str, config: Value) -> Self {
        self.steps.push(PipelineStep::BaselineCorrection {
            method: method.to_string(),
            config,
        });
        self
    }
    
    /// 执行流水线
    pub async fn execute(&self, mut container: SerializableDataContainer) -> Result<PipelineResult, ProcessingError> {
        let start_time = std::time::Instant::now();
        let mut completed_steps = Vec::new();
        
        for step in &self.steps {
            match step {
                PipelineStep::PeakDetection { method, config } => {
                    container = self.execute_peak_detection(container, method, config).await?;
                    completed_steps.push(format!("PeakDetection({})", method));
                }
                PipelineStep::PeakFitting { method, config } => {
                    container = self.execute_peak_fitting(container, method, config).await?;
                    completed_steps.push(format!("PeakFitting({})", method));
                }
                PipelineStep::PeakEnhancement { method, config } => {
                    container = self.execute_peak_enhancement(container, method, config).await?;
                    completed_steps.push(format!("PeakEnhancement({})", method));
                }
                PipelineStep::CurveReconstruction { method, config } => {
                    container = self.execute_curve_reconstruction(container, method, config).await?;
                    completed_steps.push(format!("CurveReconstruction({})", method));
                }
                PipelineStep::BaselineCorrection { method, config } => {
                    container = self.execute_baseline_correction(container, method, config).await?;
                    completed_steps.push(format!("BaselineCorrection({})", method));
                }
            }
        }
        
        let execution_time = start_time.elapsed().as_millis() as u64;
        
        Ok(PipelineResult {
            success: true,
            container,
            execution_time,
            steps_completed: completed_steps,
            error: None,
        })
    }
    
    /// 执行峰检测
    async fn execute_peak_detection(
        &self,
        container: SerializableDataContainer,
        method: &str,
        config: &Value,
    ) -> Result<SerializableDataContainer, ProcessingError> {
        // 使用现有的PeakAnalyzer进行峰检测
        let peak_analyzer = crate::core::processors::peak_analyzer::PeakAnalyzer::new_with_overlapping_processing(
            method,
            "gaussian",
            None
        ).map_err(|e| ProcessingError::ConfigError(format!("PeakAnalyzer创建失败: {}", e)))?;
        
        let data_container = container.to_data_container();
        
        // 只进行峰检测，不进行拟合
        let mut detection_config = config.clone();
        detection_config["fitting_method"] = serde_json::json!("none");
        
        let result = peak_analyzer.process(data_container, detection_config).await?;
        
        Ok(SerializableDataContainer::from(crate::core::data::DataContainer {
            metadata: result.metadata,
            spectra: Vec::new(),
            curves: result.curves,
            peaks: result.peaks,
        }))
    }
    
    /// 执行峰拟合
    async fn execute_peak_fitting(
        &self,
        container: SerializableDataContainer,
        method: &str,
        config: &Value,
    ) -> Result<SerializableDataContainer, ProcessingError> {
        // 使用现有的PeakAnalyzer进行峰拟合
        let peak_analyzer = crate::core::processors::peak_analyzer::PeakAnalyzer::new_with_overlapping_processing(
            "simple", // 使用简单检测方法
            method,
            None
        ).map_err(|e| ProcessingError::ConfigError(format!("PeakAnalyzer创建失败: {}", e)))?;
        
        let data_container = container.to_data_container();
        
        // 只进行峰拟合，跳过检测
        let mut fitting_config = config.clone();
        fitting_config["detection_method"] = serde_json::json!("none");
        fitting_config["fitting_method"] = serde_json::json!(method);
        
        let result = peak_analyzer.process(data_container, fitting_config).await?;
        
        Ok(SerializableDataContainer::from(crate::core::data::DataContainer {
            metadata: result.metadata,
            spectra: Vec::new(),
            curves: result.curves,
            peaks: result.peaks,
        }))
    }
    
    /// 执行峰增强
    async fn execute_peak_enhancement(
        &self,
        container: SerializableDataContainer,
        _method: &str,
        _config: &Value,
    ) -> Result<SerializableDataContainer, ProcessingError> {
        // 峰增强功能暂时使用现有的PeakAnalyzer
        // 这里可以后续扩展为专门的峰增强器
        let mut enhanced_container = container.clone();
        
        // 为每个峰计算质量评分
        for peak in &mut enhanced_container.peaks {
            let quality_score = peak.get_quality_score();
            peak.add_metadata("quality_score".to_string(), serde_json::json!(quality_score));
        }
        
        enhanced_container.add_metadata("enhancement_completed".to_string(), serde_json::json!(true));
        
        Ok(enhanced_container)
    }
    
    /// 执行曲线还原
    async fn execute_curve_reconstruction(
        &self,
        container: SerializableDataContainer,
        _method: &str,
        _config: &Value,
    ) -> Result<SerializableDataContainer, ProcessingError> {
        // 曲线还原功能暂时返回原始容器
        // 这里可以后续扩展为专门的曲线还原器
        let mut reconstructed_container = container.clone();
        
        // 为每个峰生成拟合曲线数据点
        for peak in &reconstructed_container.peaks {
            if !peak.fit_parameters.is_empty() {
                // 这里可以生成拟合曲线的数据点
                // 暂时跳过具体实现
            }
        }
        
        reconstructed_container.add_metadata("reconstruction_completed".to_string(), serde_json::json!(true));
        
        Ok(reconstructed_container)
    }
    
    /// 执行基线校正
    async fn execute_baseline_correction(
        &self,
        container: SerializableDataContainer,
        method: &str,
        config: &Value,
    ) -> Result<SerializableDataContainer, ProcessingError> {
        // 使用现有的BaselineProcessor进行基线校正
        let baseline_processor = crate::core::processors::baseline_correction::BaselineProcessor::new();
        let data_container = container.to_data_container();
        
        let mut correction_config = config.clone();
        correction_config["method"] = serde_json::json!(method);
        
        let result = baseline_processor.process(data_container, correction_config).await?;
        
        Ok(SerializableDataContainer::from(crate::core::data::DataContainer {
            metadata: result.metadata,
            spectra: Vec::new(),
            curves: result.curves,
            peaks: result.peaks,
        }))
    }
}

impl Default for PipelineManager {
    fn default() -> Self {
        Self::new()
    }
}
