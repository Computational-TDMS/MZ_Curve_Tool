//! 工作流控制器
//! 
//! 负责管理处理流程的执行和阶段控制

use std::collections::HashMap;
use std::sync::Arc;
use crate::core::data::{Curve, Peak, ProcessingError};
use super::component_registry::{ComponentRegistry, ProcessingData, ComponentType, Component};
use super::strategy_controller::{StrategyController, ProcessingStrategy, ProcessingContext};
use serde_json::Value;

/// 处理阶段
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingStage {
    /// 峰检测阶段
    PeakDetection,
    /// 重叠峰分析阶段
    OverlapAnalysis,
    /// 重叠峰处理阶段
    OverlapProcessing,
    /// 峰形分析阶段
    PeakShapeAnalysis,
    /// 拟合阶段
    Fitting,
    /// 参数优化阶段
    ParameterOptimization,
    /// 后处理阶段
    PostProcessing,
    /// 验证阶段
    Validation,
}

/// 阶段结果
#[derive(Debug, Clone)]
pub struct StageResult {
    pub stage: ProcessingStage,
    pub success: bool,
    pub data: ProcessingData,
    pub metrics: HashMap<String, f64>,
    pub metadata: HashMap<String, Value>,
    pub error: Option<String>,
}

/// 工作流配置
#[derive(Debug, Clone)]
pub struct WorkflowConfig {
    pub stages: Vec<ProcessingStage>,
    pub parallel_execution: bool,
    pub error_handling: ErrorHandlingMode,
    pub quality_threshold: f64,
    pub max_iterations: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorHandlingMode {
    /// 遇到错误时停止
    StopOnError,
    /// 跳过错误阶段继续执行
    SkipOnError,
    /// 重试错误阶段
    RetryOnError { max_retries: usize },
}

impl Default for WorkflowConfig {
    fn default() -> Self {
        Self {
            stages: vec![
                ProcessingStage::PeakDetection,
                ProcessingStage::OverlapAnalysis,
                ProcessingStage::OverlapProcessing,
                ProcessingStage::PeakShapeAnalysis,
                ProcessingStage::Fitting,
                ProcessingStage::ParameterOptimization,
                ProcessingStage::PostProcessing,
                ProcessingStage::Validation,
            ],
            parallel_execution: false,
            error_handling: ErrorHandlingMode::StopOnError,
            quality_threshold: 0.8,
            max_iterations: 100,
        }
    }
}

/// 工作流控制器
#[derive(Debug)]
pub struct WorkflowController {
    registry: Arc<ComponentRegistry>,
    strategy_controller: Arc<StrategyController>,
    config: WorkflowConfig,
}

impl WorkflowController {
    pub fn new(
        registry: Arc<ComponentRegistry>,
        strategy_controller: Arc<StrategyController>,
    ) -> Self {
        Self {
            registry,
            strategy_controller,
            config: WorkflowConfig::default(),
        }
    }
    
    pub fn with_config(
        registry: Arc<ComponentRegistry>,
        strategy_controller: Arc<StrategyController>,
        config: WorkflowConfig,
    ) -> Self {
        Self {
            registry,
            strategy_controller,
            config,
        }
    }
    
    /// 执行完整工作流
    pub fn execute_workflow(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        user_config: &Value,
    ) -> Result<Vec<Peak>, ProcessingError> {
        println!("开始执行峰处理工作流，输入峰数量: {}", peaks.len());
        
        // 1. 创建处理上下文
        let context = ProcessingContext::new(peaks.to_vec(), curve.clone());
        
        // 2. 选择处理策略
        let strategy = self.strategy_controller.select_strategy(&context)?;
        println!("选择处理策略: {}", strategy.name);
        
        // 3. 创建初始处理数据
        let mut processing_data = ProcessingData::new(peaks.to_vec(), curve.clone());
        
        // 4. 执行各个阶段
        let mut stage_results = Vec::new();
        for stage in &self.config.stages {
            let result = self.execute_stage(stage, &processing_data, &strategy, user_config)?;
            stage_results.push(result.clone());
            
            if !result.success {
                match self.config.error_handling {
                    ErrorHandlingMode::StopOnError => {
                        return Err(ProcessingError::process_error(
                            &result.error.unwrap_or_else(|| "阶段执行失败".to_string())
                        ));
                    },
                    ErrorHandlingMode::SkipOnError => {
                        println!("跳过失败的阶段: {:?}", stage);
                        continue;
                    },
                    ErrorHandlingMode::RetryOnError { max_retries } => {
                        let mut retry_count = 0;
                        let mut current_result = result.clone();
                        
                        while !current_result.success && retry_count < max_retries {
                            retry_count += 1;
                            println!("重试阶段 {:?}，第 {} 次", stage, retry_count);
                            current_result = self.execute_stage(stage, &processing_data, &strategy, user_config)?;
                        }
                        
                        if !current_result.success {
                            return Err(ProcessingError::process_error(
                                &format!("阶段 {:?} 重试 {} 次后仍然失败", stage, max_retries)
                            ));
                        }
                        
                        stage_results.push(current_result.clone());
                    }
                }
            }
            
            // 更新处理数据
            processing_data = result.data;
        }
        
        // 5. 验证最终结果
        let final_peaks = self.validate_final_result(&processing_data, &stage_results)?;
        println!("工作流执行完成，输出峰数量: {}", final_peaks.len());
        
        Ok(final_peaks)
    }
    
    /// 执行单个阶段
    fn execute_stage(
        &self,
        stage: &ProcessingStage,
        data: &ProcessingData,
        strategy: &ProcessingStrategy,
        user_config: &Value,
    ) -> Result<StageResult, ProcessingError> {
        println!("执行阶段: {:?}", stage);
        
        let start_time = std::time::Instant::now();
        let mut metrics = HashMap::new();
        let mut metadata = HashMap::new();
        
        let result = match stage {
            ProcessingStage::PeakDetection => {
                self.execute_peak_detection_stage(data, strategy, user_config)
            },
            ProcessingStage::OverlapAnalysis => {
                self.execute_overlap_analysis_stage(data, strategy, user_config)
            },
            ProcessingStage::OverlapProcessing => {
                self.execute_overlap_processing_stage(data, strategy, user_config)
            },
            ProcessingStage::PeakShapeAnalysis => {
                self.execute_peak_shape_analysis_stage(data, strategy, user_config)
            },
            ProcessingStage::Fitting => {
                self.execute_fitting_stage(data, strategy, user_config)
            },
            ProcessingStage::ParameterOptimization => {
                self.execute_parameter_optimization_stage(data, strategy, user_config)
            },
            ProcessingStage::PostProcessing => {
                self.execute_post_processing_stage(data, strategy, user_config)
            },
            ProcessingStage::Validation => {
                self.execute_validation_stage(data, strategy, user_config)
            },
        };
        
        let execution_time = start_time.elapsed().as_millis() as f64;
        metrics.insert("execution_time_ms".to_string(), execution_time);
        metadata.insert("stage_name".to_string(), Value::String(format!("{:?}", stage)));
        
        match result {
            Ok(processed_data) => {
                let success = self.evaluate_stage_quality(&processed_data, stage);
                metrics.insert("quality_score".to_string(), success as u8 as f64);
                
                Ok(StageResult {
                    stage: stage.clone(),
                    success,
                    data: processed_data,
                    metrics,
                    metadata,
                    error: None,
                })
            },
            Err(e) => {
                Ok(StageResult {
                    stage: stage.clone(),
                    success: false,
                    data: data.clone(),
                    metrics,
                    metadata,
                    error: Some(e.to_string()),
                })
            }
        }
    }
    
    /// 峰检测阶段
    fn execute_peak_detection_stage(
        &self,
        data: &ProcessingData,
        strategy: &ProcessingStrategy,
        _user_config: &Value,
    ) -> Result<ProcessingData, ProcessingError> {
        let component = self.registry.get_component(
            &ComponentType::PeakDetector,
            &strategy.peak_detection,
            &strategy.configuration,
        )?;
        
        component.process(data, &strategy.configuration)
    }
    
    /// 重叠峰分析阶段
    fn execute_overlap_analysis_stage(
        &self,
        data: &ProcessingData,
        _strategy: &ProcessingStrategy,
        _user_config: &Value,
    ) -> Result<ProcessingData, ProcessingError> {
        // 分析重叠峰特征
        let mut result_data = data.clone();
        let overlap_ratio = self.calculate_overlap_ratio(&data.peaks);
        result_data.add_intermediate_result("overlap_ratio".to_string(), Value::Number(serde_json::Number::from_f64(overlap_ratio).unwrap()));
        
        Ok(result_data)
    }
    
    /// 重叠峰处理阶段
    fn execute_overlap_processing_stage(
        &self,
        data: &ProcessingData,
        strategy: &ProcessingStrategy,
        _user_config: &Value,
    ) -> Result<ProcessingData, ProcessingError> {
        if strategy.overlap_processing == "none" {
            return Ok(data.clone());
        }
        
        let component = self.registry.get_component(
            &ComponentType::OverlapProcessor,
            &strategy.overlap_processing,
            &strategy.configuration,
        )?;
        
        component.process(data, &strategy.configuration)
    }
    
    /// 峰形分析阶段
    fn execute_peak_shape_analysis_stage(
        &self,
        data: &ProcessingData,
        strategy: &ProcessingStrategy,
        _user_config: &Value,
    ) -> Result<ProcessingData, ProcessingError> {
        let component = self.registry.get_component(
            &ComponentType::PeakAnalyzer,
            "peak_shape_analyzer",
            &strategy.configuration,
        )?;
        
        component.process(data, &strategy.configuration)
    }
    
    /// 拟合阶段
    fn execute_fitting_stage(
        &self,
        data: &ProcessingData,
        strategy: &ProcessingStrategy,
        _user_config: &Value,
    ) -> Result<ProcessingData, ProcessingError> {
        let component = self.registry.get_component(
            &ComponentType::FittingMethod,
            &strategy.fitting_method,
            &strategy.configuration,
        )?;
        
        component.process(data, &strategy.configuration)
    }
    
    /// 参数优化阶段
    fn execute_parameter_optimization_stage(
        &self,
        data: &ProcessingData,
        strategy: &ProcessingStrategy,
        _user_config: &Value,
    ) -> Result<ProcessingData, ProcessingError> {
        let component = self.registry.get_component(
            &ComponentType::ParameterOptimizer,
            &strategy.optimization_algorithm,
            &strategy.configuration,
        )?;
        
        component.process(data, &strategy.configuration)
    }
    
    /// 后处理阶段
    fn execute_post_processing_stage(
        &self,
        data: &ProcessingData,
        strategy: &ProcessingStrategy,
        _user_config: &Value,
    ) -> Result<ProcessingData, ProcessingError> {
        if let Some(post_processing) = &strategy.post_processing {
            let component = self.registry.get_component(
                &ComponentType::PostProcessor,
                post_processing,
                &strategy.configuration,
            )?;
            
            component.process(data, &strategy.configuration)
        } else {
            Ok(data.clone())
        }
    }
    
    /// 验证阶段
    fn execute_validation_stage(
        &self,
        data: &ProcessingData,
        _strategy: &ProcessingStrategy,
        _user_config: &Value,
    ) -> Result<ProcessingData, ProcessingError> {
        // 执行质量验证
        let quality_score = self.calculate_overall_quality(data);
        let mut result_data = data.clone();
        result_data.add_intermediate_result("quality_score".to_string(), Value::Number(serde_json::Number::from_f64(quality_score).unwrap()));
        
        Ok(result_data)
    }
    
    /// 评估阶段质量
    fn evaluate_stage_quality(&self, data: &ProcessingData, stage: &ProcessingStage) -> bool {
        match stage {
            ProcessingStage::PeakDetection => !data.peaks.is_empty(),
            ProcessingStage::Fitting => {
                data.peaks.iter().all(|peak| peak.amplitude > 0.0)
            },
            ProcessingStage::Validation => {
                if let Some(quality) = data.get_intermediate_result("quality_score") {
                    quality.as_f64().unwrap_or(0.0) >= self.config.quality_threshold
                } else {
                    false
                }
            },
            _ => true, // 其他阶段默认通过
        }
    }
    
    /// 计算整体质量
    fn calculate_overall_quality(&self, data: &ProcessingData) -> f64 {
        if data.peaks.is_empty() {
            return 0.0;
        }
        
        let peak_count_score = (data.peaks.len() as f64).min(10.0) / 10.0;
        let amplitude_score = data.peaks.iter()
            .map(|p| p.amplitude)
            .fold(0.0, f64::max) / 1000.0; // 假设最大振幅为1000
        
        (peak_count_score + amplitude_score) / 2.0
    }
    
    /// 验证最终结果
    fn validate_final_result(
        &self,
        data: &ProcessingData,
        stage_results: &[StageResult],
    ) -> Result<Vec<Peak>, ProcessingError> {
        // 检查所有关键阶段是否成功
        let critical_stages = vec![
            ProcessingStage::PeakDetection,
            ProcessingStage::Fitting,
            ProcessingStage::Validation,
        ];
        
        for stage in critical_stages {
            if let Some(result) = stage_results.iter().find(|r| r.stage == stage) {
                if !result.success {
                    return Err(ProcessingError::process_error(
                        &format!("关键阶段 {:?} 执行失败", stage)
                    ));
                }
            }
        }
        
        // 检查质量阈值
        if let Some(quality) = data.get_intermediate_result("quality_score") {
            let quality_score = quality.as_f64().unwrap_or(0.0);
            if quality_score < self.config.quality_threshold {
                return Err(ProcessingError::process_error(
                    &format!("质量分数 {} 低于阈值 {}", quality_score, self.config.quality_threshold)
                ));
            }
        }
        
        Ok(data.peaks.clone())
    }
    
    /// 计算重叠比例
    fn calculate_overlap_ratio(&self, peaks: &[Peak]) -> f64 {
        if peaks.len() < 2 {
            return 0.0;
        }
        
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
