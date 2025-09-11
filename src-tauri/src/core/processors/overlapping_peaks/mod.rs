//! 重叠峰处理模块
//! 
//! 包含多种重叠峰处理算法的实现

pub mod fbf_preprocessor;
pub mod sharpen_cwt_preprocessor;
pub mod emg_nlls_fitter;
pub mod extreme_overlap_processor;

use crate::core::data::{Curve, Peak, ProcessingError, DataContainer, ProcessingResult};
use crate::core::processors::core::Processor;
use serde_json::Value;
use async_trait::async_trait;

/// 重叠峰处理器trait
pub trait OverlappingPeakProcessor {
    fn name(&self) -> &str;
    fn process_overlapping_peaks(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        config: &Value,
    ) -> Result<Vec<Peak>, ProcessingError>;
}

/// 重叠峰处理器枚举
#[derive(Debug)]
pub enum OverlappingPeakProcessorEnum {
    FBF(fbf_preprocessor::FBFPreprocessor),
    SharpenCWT(sharpen_cwt_preprocessor::SharpenCWTPreprocessor),
    EMGNLLS(emg_nlls_fitter::EMGNLLSFitter),
    ExtremeOverlap(extreme_overlap_processor::ExtremeOverlapProcessor),
}

impl OverlappingPeakProcessor for OverlappingPeakProcessorEnum {
    fn name(&self) -> &str {
        match self {
            OverlappingPeakProcessorEnum::FBF(p) => p.name(),
            OverlappingPeakProcessorEnum::SharpenCWT(p) => p.name(),
            OverlappingPeakProcessorEnum::EMGNLLS(p) => p.name(),
            OverlappingPeakProcessorEnum::ExtremeOverlap(p) => p.name(),
        }
    }

    fn process_overlapping_peaks(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        config: &Value,
    ) -> Result<Vec<Peak>, ProcessingError> {
        match self {
            OverlappingPeakProcessorEnum::FBF(p) => p.process_overlapping_peaks(peaks, curve, config),
            OverlappingPeakProcessorEnum::SharpenCWT(p) => p.process_overlapping_peaks(peaks, curve, config),
            OverlappingPeakProcessorEnum::EMGNLLS(p) => p.process_overlapping_peaks(peaks, curve, config),
            OverlappingPeakProcessorEnum::ExtremeOverlap(p) => p.process_overlapping_peaks(peaks, curve, config),
        }
    }
}

#[async_trait]
impl Processor for OverlappingPeakProcessorEnum {
    fn name(&self) -> &str {
        match self {
            OverlappingPeakProcessorEnum::FBF(p) => p.name(),
            OverlappingPeakProcessorEnum::SharpenCWT(p) => p.name(),
            OverlappingPeakProcessorEnum::EMGNLLS(p) => p.name(),
            OverlappingPeakProcessorEnum::ExtremeOverlap(p) => p.name(),
        }
    }

    fn description(&self) -> &str {
        match self {
            OverlappingPeakProcessorEnum::FBF(_) => "前向背景拟合重叠峰处理器",
            OverlappingPeakProcessorEnum::SharpenCWT(_) => "CWT锐化重叠峰处理器",
            OverlappingPeakProcessorEnum::EMGNLLS(_) => "EMG NLLS重叠峰处理器",
            OverlappingPeakProcessorEnum::ExtremeOverlap(_) => "极端重叠峰处理器",
        }
    }

    fn processor_type(&self) -> crate::core::processors::core::ProcessorType {
        crate::core::processors::core::ProcessorType::OverlappingPeaks
    }
    
    fn supported_methods(&self) -> Vec<String> {
        vec![
            "fbf".to_string(),
            "sharpen_cwt".to_string(),
            "emg_nlls".to_string(),
            "extreme_overlap".to_string(),
        ]
    }

    fn config_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "method": {
                    "type": "string",
                    "enum": ["fbf", "sharpen_cwt", "emg_nlls", "extreme_overlap"]
                }
            }
        })
    }

    async fn process(&self, input: DataContainer, config: serde_json::Value) -> Result<ProcessingResult, ProcessingError> {
        if input.curves.is_empty() {
            return Err(ProcessingError::DataError("没有可处理的曲线数据".to_string()));
        }

        let curve = &input.curves[0];
        
        let processed_peaks = self.process_overlapping_peaks(&curve.peaks, curve, &config)?;
        
        // 将处理后的峰添加到曲线中
        let mut result_curves = input.curves.clone();
        if let Some(result_curve) = result_curves.first_mut() {
            result_curve.peaks = processed_peaks.clone();
        }

        Ok(ProcessingResult {
            curves: result_curves,
            peaks: processed_peaks,
            metadata: input.metadata,
        })
    }

}

/// 创建重叠峰处理器
pub fn create_overlapping_processor(method: &str) -> Result<OverlappingPeakProcessorEnum, ProcessingError> {
    match method {
        "fbf" => Ok(OverlappingPeakProcessorEnum::FBF(fbf_preprocessor::FBFPreprocessor::new())),
        "sharpen_cwt" => Ok(OverlappingPeakProcessorEnum::SharpenCWT(sharpen_cwt_preprocessor::SharpenCWTPreprocessor::new())),
        "emg_nlls" => Ok(OverlappingPeakProcessorEnum::EMGNLLS(emg_nlls_fitter::EMGNLLSFitter::new())),
        "extreme_overlap" => Ok(OverlappingPeakProcessorEnum::ExtremeOverlap(extreme_overlap_processor::ExtremeOverlapProcessor::new())),
        _ => Err(ProcessingError::ConfigError(format!("不支持的重叠峰处理方法: {}", method))),
    }
}

/// 重叠峰处理策略
#[derive(Debug, Clone)]
pub enum OverlappingPeakStrategy {
    /// 单峰处理
    SinglePeak,
    /// 轻度重叠 - 使用FBF
    LightOverlap,
    /// 中度重叠 - 使用锐化+CWT
    MediumOverlap,
    /// 极度重叠+低信噪比 - 使用锐化+CWT预热，然后EMG-NLLS
    ExtremeOverlapLowSNR,
}

impl OverlappingPeakStrategy {
    /// 根据峰特征自动选择策略
    pub fn auto_select(peaks: &[Peak], curve: &Curve) -> Self {
        if peaks.len() <= 1 {
            return Self::SinglePeak;
        }
        
        // 计算峰间距离和重叠程度
        let mut min_distance = f64::INFINITY;
        let mut max_overlap = 0.0_f64;
        
        for i in 0..peaks.len() {
            for j in (i + 1)..peaks.len() {
                let distance = (peaks[i].center - peaks[j].center).abs();
                let overlap = (peaks[i].fwhm + peaks[j].fwhm) / 2.0 - distance;
                min_distance = min_distance.min(distance);
                max_overlap = max_overlap.max(overlap);
            }
        }
        
        // 计算信噪比
        let snr = Self::estimate_snr(curve);
        
        // 根据重叠程度和信噪比选择策略
        if max_overlap < 0.1 {
            Self::SinglePeak
        } else if max_overlap < 0.5 {
            Self::LightOverlap
        } else if max_overlap < 1.0 {
            Self::MediumOverlap
        } else if snr < 10.0 {
            Self::ExtremeOverlapLowSNR
        } else {
            Self::MediumOverlap
        }
    }
    
    /// 估计信噪比
    fn estimate_snr(curve: &Curve) -> f64 {
        if curve.y_values.is_empty() {
            return 0.0;
        }
        
        let max_signal = curve.y_values.iter().fold(0.0_f64, |a, &b| a.max(b));
        let noise_level = curve.y_values.iter().sum::<f64>() / curve.y_values.len() as f64;
        
        if noise_level > 0.0 {
            max_signal / noise_level
        } else {
            0.0
        }
    }
    
    /// 获取对应的处理方法
    pub fn get_processor_method(&self) -> &'static str {
        match self {
            Self::SinglePeak => "none",
            Self::LightOverlap => "fbf",
            Self::MediumOverlap => "sharpen_cwt",
            Self::ExtremeOverlapLowSNR => "extreme_overlap",
        }
    }
}
