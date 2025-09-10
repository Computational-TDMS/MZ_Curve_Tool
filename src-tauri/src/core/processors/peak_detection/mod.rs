//! 峰检测模块
//! 
//! 包含多种峰检测算法的实现

pub mod cwt_detector;
pub mod simple_detector;
pub mod peak_finder_detector;

use crate::core::data::{Curve, Peak, ProcessingError};
use serde_json::Value;

/// 峰检测器trait
pub trait PeakDetector {
    fn name(&self) -> &str;
    fn detect_peaks(&self, curve: &Curve, config: &Value) -> Result<Vec<Peak>, ProcessingError>;
}

/// 峰检测器枚举
#[derive(Debug)]
pub enum PeakDetectorEnum {
    CWT(cwt_detector::CWTDetector),
    Simple(simple_detector::SimpleDetector),
    PeakFinder(peak_finder_detector::PeakFinderDetector),
}

impl PeakDetector for PeakDetectorEnum {
    fn name(&self) -> &str {
        match self {
            PeakDetectorEnum::CWT(d) => d.name(),
            PeakDetectorEnum::Simple(d) => d.name(),
            PeakDetectorEnum::PeakFinder(d) => d.name(),
        }
    }

    fn detect_peaks(&self, curve: &Curve, config: &Value) -> Result<Vec<Peak>, ProcessingError> {
        match self {
            PeakDetectorEnum::CWT(d) => d.detect_peaks(curve, config),
            PeakDetectorEnum::Simple(d) => d.detect_peaks(curve, config),
            PeakDetectorEnum::PeakFinder(d) => d.detect_peaks(curve, config),
        }
    }
}

/// 创建峰检测器
pub fn create_detector(method: &str) -> Result<PeakDetectorEnum, ProcessingError> {
    match method {
        "cwt" => Ok(PeakDetectorEnum::CWT(cwt_detector::CWTDetector)),
        "simple" => Ok(PeakDetectorEnum::Simple(simple_detector::SimpleDetector)),
        "peak_finder" => Ok(PeakDetectorEnum::PeakFinder(peak_finder_detector::PeakFinderDetector)),
        _ => Err(ProcessingError::ConfigError(format!("不支持的检测方法: {}", method))),
    }
}
