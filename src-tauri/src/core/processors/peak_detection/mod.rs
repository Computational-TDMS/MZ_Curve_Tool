//! 峰检测模块
//! 
//! 包含多种峰检测算法的实现

pub mod cwt_detector;
pub mod simple_detector;
pub mod peak_finder_detector;

use crate::core::data::{Curve, Peak, ProcessingError, DataContainer, ProcessingResult};
use crate::core::processors::core::Processor;
use serde_json::Value;
use async_trait::async_trait;

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

#[async_trait]
impl Processor for PeakDetectorEnum {
    fn name(&self) -> &str {
        match self {
            PeakDetectorEnum::CWT(d) => d.name(),
            PeakDetectorEnum::Simple(d) => d.name(),
            PeakDetectorEnum::PeakFinder(d) => d.name(),
        }
    }

    fn description(&self) -> &str {
        match self {
            PeakDetectorEnum::CWT(_) => "连续小波变换峰检测器",
            PeakDetectorEnum::Simple(_) => "简单峰检测器",
            PeakDetectorEnum::PeakFinder(_) => "峰查找器",
        }
    }

    fn processor_type(&self) -> crate::core::processors::core::ProcessorType {
        crate::core::processors::core::ProcessorType::PeakDetection
    }
    
    fn supported_methods(&self) -> Vec<String> {
        vec![
            "cwt".to_string(),
            "simple".to_string(),
            "peak_finder".to_string(),
        ]
    }

    fn config_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "method": {
                    "type": "string",
                    "enum": ["cwt", "simple", "peak_finder"]
                }
            }
        })
    }

    async fn process(&self, input: DataContainer, config: serde_json::Value) -> Result<ProcessingResult, ProcessingError> {
        if input.curves.is_empty() {
            return Err(ProcessingError::DataError("没有可处理的曲线数据".to_string()));
        }

        let curve = &input.curves[0];
        
        let peaks = self.detect_peaks(curve, &config)?;
        
        // 将检测到的峰添加到曲线中
        let mut result_curves = input.curves.clone();
        if let Some(result_curve) = result_curves.first_mut() {
            result_curve.peaks = peaks.clone();
        }

        Ok(ProcessingResult {
            curves: result_curves,
            peaks,
            metadata: input.metadata,
        })
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
