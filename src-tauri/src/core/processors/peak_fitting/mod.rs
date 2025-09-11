//! 峰拟合模块
//! 
//! 提供多峰拟合和峰拆分算法的实现

pub mod peak_shapes;
pub mod parameter_optimizer;
pub mod advanced_algorithms;
pub mod multi_peak_fitter;
pub mod controllers;

use crate::core::data::{Curve, Peak, ProcessingError};
use serde_json::Value;

// 重新导出控制器模块的主要类型
pub use controllers::{
    ComponentRegistry, ComponentType, ComponentFactory, Component, ProcessingData,
    StrategyController, ProcessingMode, ProcessingStrategy, ProcessingContext, StrategyRule,
    WorkflowController, ProcessingStage, StageResult, WorkflowConfig, ErrorHandlingMode,
    ConfigManager, ConfigSource, ConfigValidator,
    StrategyBuilder, PredefinedStrategyBuilder, StrategyRuleBuilder,
    PeakProcessingController,
};

/// 峰拟合器trait
pub trait PeakFitter {
    fn name(&self) -> &str;
    fn fit_peak(&self, peak: &Peak, curve: &Curve, config: &Value) -> Result<Peak, ProcessingError>;
}

/// 峰拟合器枚举
#[derive(Debug)]
pub enum PeakFitterEnum {
    MultiPeak(multi_peak_fitter::MultiPeakFitter),
}

impl PeakFitter for PeakFitterEnum {
    fn name(&self) -> &str {
        match self {
            PeakFitterEnum::MultiPeak(fitter) => fitter.name(),
        }
    }

    fn fit_peak(&self, peak: &Peak, curve: &Curve, config: &Value) -> Result<Peak, ProcessingError> {
        match self {
            PeakFitterEnum::MultiPeak(fitter) => fitter.fit_peak(peak, curve, config),
        }
    }
}

/// 创建峰拟合器
pub fn create_fitter(fitter_type: &str) -> Result<PeakFitterEnum, ProcessingError> {
    match fitter_type {
        "multi_peak" => Ok(PeakFitterEnum::MultiPeak(multi_peak_fitter::MultiPeakFitter::new())),
        _ => Err(ProcessingError::ConfigError(format!("不支持的拟合方法: {}", fitter_type))),
    }
}

/// 创建带自定义优化算法的峰拟合器
pub fn create_fitter_with_optimizer(
    fitter_type: &str, 
    algorithm: parameter_optimizer::OptimizationAlgorithm
) -> Result<PeakFitterEnum, ProcessingError> {
    match fitter_type {
        "multi_peak" => Ok(PeakFitterEnum::MultiPeak(
            multi_peak_fitter::MultiPeakFitter::with_optimizer(algorithm)
        )),
        _ => Err(ProcessingError::ConfigError(format!("不支持的拟合方法: {}", fitter_type))),
    }
}