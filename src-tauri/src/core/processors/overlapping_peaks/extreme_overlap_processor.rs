//! 极度重叠+低信噪比峰处理流程
//! 
//! 实现完整的极度重叠峰处理流程：锐化+CWT预热 → EMG-NLLS拟合

use crate::core::data::{Curve, Peak, ProcessingError};
use crate::core::processors::overlapping_peaks::{
    OverlappingPeakProcessor, OverlappingPeakStrategy,
    sharpen_cwt_preprocessor::SharpenCWTPreprocessor,
    emg_nlls_fitter::EMGNLLSFitter,
};
use serde_json::Value;

/// 极度重叠峰处理器
#[derive(Debug)]
pub struct ExtremeOverlapProcessor {
    /// 锐化+CWT预处理器
    sharpen_cwt_preprocessor: SharpenCWTPreprocessor,
    /// EMG-NLLS拟合器
    emg_nlls_fitter: EMGNLLSFitter,
    /// 信噪比阈值
    snr_threshold: f64,
    /// 重叠度阈值
    overlap_threshold: f64,
}

impl OverlappingPeakProcessor for ExtremeOverlapProcessor {
    fn name(&self) -> &str {
        "extreme_overlap_processor"
    }

    fn process_overlapping_peaks(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        config: &Value,
    ) -> Result<Vec<Peak>, ProcessingError> {
        if peaks.len() < 2 {
            return Ok(peaks.to_vec());
        }
        
        // 1. 评估峰的重叠程度和信噪比
        let (overlap_degree, snr) = self.assess_peak_conditions(peaks, curve);
        
        // 2. 判断是否需要极度重叠处理
        if overlap_degree < self.overlap_threshold || snr > self.snr_threshold {
            // 使用标准处理流程
            return self.standard_processing(peaks, curve, config);
        }
        
        // 3. 执行极度重叠+低信噪比处理流程
        self.extreme_overlap_processing(peaks, curve, config)
    }
}

impl ExtremeOverlapProcessor {
    /// 创建新的极度重叠峰处理器
    pub fn new() -> Self {
        Self {
            sharpen_cwt_preprocessor: SharpenCWTPreprocessor::new()
                .with_parameters(2.0, (1, 30), 7, 0.05), // 增强参数
            emg_nlls_fitter: EMGNLLSFitter::new()
                .with_parameters(200, 1e-8, 0.001), // 更严格的收敛条件
            snr_threshold: 10.0,
            overlap_threshold: 1.0,
        }
    }
    
    /// 设置参数
    pub fn with_parameters(
        mut self,
        snr_threshold: f64,
        overlap_threshold: f64,
        sharpen_strength: f64,
        cwt_scales: (usize, usize),
        max_iterations: usize,
    ) -> Self {
        self.snr_threshold = snr_threshold;
        self.overlap_threshold = overlap_threshold;
        self.sharpen_cwt_preprocessor = SharpenCWTPreprocessor::new()
            .with_parameters(sharpen_strength, cwt_scales, 7, 0.05);
        self.emg_nlls_fitter = EMGNLLSFitter::new()
            .with_parameters(max_iterations, 1e-8, 0.001);
        self
    }
    
    /// 评估峰的条件
    fn assess_peak_conditions(&self, peaks: &[Peak], curve: &Curve) -> (f64, f64) {
        // 计算重叠程度
        let overlap_degree = self.calculate_overlap_degree(peaks);
        
        // 计算信噪比
        let snr = self.calculate_signal_to_noise_ratio(curve);
        
        (overlap_degree, snr)
    }
    
    /// 计算重叠程度
    fn calculate_overlap_degree(&self, peaks: &[Peak]) -> f64 {
        if peaks.len() < 2 {
            return 0.0;
        }
        
        let mut total_overlap = 0.0;
        let mut pair_count = 0;
        
        for i in 0..peaks.len() {
            for j in (i + 1)..peaks.len() {
                let distance = (peaks[i].center - peaks[j].center).abs();
                let combined_width = (peaks[i].fwhm + peaks[j].fwhm) / 2.0;
                let overlap = (combined_width - distance).max(0.0) / combined_width;
                total_overlap += overlap;
                pair_count += 1;
            }
        }
        
        if pair_count > 0 {
            total_overlap / pair_count as f64
        } else {
            0.0
        }
    }
    
    /// 计算信噪比
    fn calculate_signal_to_noise_ratio(&self, curve: &Curve) -> f64 {
        if curve.y_values.is_empty() {
            return 0.0;
        }
        
        // 计算信号强度（峰值的平均值）
        let signal_strength = curve.y_values.iter().fold(0.0_f64, |a, &b| a.max(b));
        
        // 计算噪声水平（使用中位数绝对偏差）
        let mut sorted_values = curve.y_values.clone();
        sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let median = if sorted_values.len() % 2 == 0 {
            (sorted_values[sorted_values.len() / 2 - 1] + sorted_values[sorted_values.len() / 2]) / 2.0
        } else {
            sorted_values[sorted_values.len() / 2]
        };
        
        let mad = sorted_values.iter()
            .map(|&x| (x - median).abs())
            .fold(0.0_f64, |a, b| a.max(b));
        
        let noise_level = mad * 1.4826; // MAD到标准差的转换因子
        
        if noise_level > 0.0 {
            signal_strength / noise_level
        } else {
            0.0
        }
    }
    
    /// 标准处理流程
    fn standard_processing(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        config: &Value,
    ) -> Result<Vec<Peak>, ProcessingError> {
        // 根据峰特征选择处理策略
        let strategy = OverlappingPeakStrategy::auto_select(peaks, curve);
        
        match strategy {
            OverlappingPeakStrategy::SinglePeak => Ok(peaks.to_vec()),
            OverlappingPeakStrategy::LightOverlap => {
                // 使用FBF处理
                use crate::core::processors::overlapping_peaks::fbf_preprocessor::FBFPreprocessor;
                let fbf_processor = FBFPreprocessor::new();
                fbf_processor.preprocess_overlapping_peaks(peaks, curve, config)
            }
            OverlappingPeakStrategy::MediumOverlap => {
                // 使用锐化+CWT处理
                self.sharpen_cwt_preprocessor.process_overlapping_peaks(peaks, curve, config)
            }
            OverlappingPeakStrategy::ExtremeOverlapLowSNR => {
                // 使用极度重叠处理
                self.extreme_overlap_processing(peaks, curve, config)
            }
        }
    }
    
    /// 极度重叠处理流程
    fn extreme_overlap_processing(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        config: &Value,
    ) -> Result<Vec<Peak>, ProcessingError> {
        // 步骤1：锐化+CWT预热
        let preprocessed_peaks = self.sharpen_cwt_preprocessor
            .process_overlapping_peaks(peaks, curve, config)?;
        
        // 步骤2：EMG-NLLS拟合
        let fitted_peaks = self.emg_nlls_fitter
            .process_overlapping_peaks(&preprocessed_peaks, curve, config)?;
        
        // 步骤3：后处理和验证
        let validated_peaks = self.post_process_and_validate(&fitted_peaks, curve)?;
        
        Ok(validated_peaks)
    }
    
    /// 后处理和验证
    fn post_process_and_validate(
        &self,
        peaks: &[Peak],
        curve: &Curve,
    ) -> Result<Vec<Peak>, ProcessingError> {
        let mut validated_peaks = Vec::new();
        
        for peak in peaks {
            // 验证峰的质量
            if self.validate_peak_quality(peak, curve) {
                // 添加处理历史元数据
                let mut validated_peak = peak.clone();
                validated_peak.add_metadata("extreme_overlap_processed".to_string(), serde_json::json!(true));
                validated_peak.add_metadata("processing_pipeline".to_string(), serde_json::json!("sharpen_cwt_emg_nlls"));
                validated_peak.add_metadata("processing_timestamp".to_string(), serde_json::json!(chrono::Utc::now().to_rfc3339()));
                
                validated_peaks.push(validated_peak);
            }
        }
        
        // 按峰中心位置排序
        validated_peaks.sort_by(|a, b| a.center.partial_cmp(&b.center).unwrap());
        
        Ok(validated_peaks)
    }
    
    /// 验证峰质量
    fn validate_peak_quality(&self, peak: &Peak, curve: &Curve) -> bool {
        // 检查基本参数
        if peak.amplitude <= 0.0 || peak.fwhm <= 0.0 || peak.center.is_nan() {
            return false;
        }
        
        // 检查峰是否在数据范围内
        if peak.center < curve.x_values[0] || peak.center > curve.x_values[curve.x_values.len() - 1] {
            return false;
        }
        
        // 检查拟合质量
        if peak.rsquared < 0.5 {
            return false;
        }
        
        // 检查峰宽是否合理
        let data_range = curve.x_values[curve.x_values.len() - 1] - curve.x_values[0];
        if peak.fwhm > data_range * 0.5 {
            return false;
        }
        
        // 检查峰是否与数据匹配
        if let Some(center_idx) = self.find_closest_index(&curve.x_values, peak.center) {
            let expected_intensity = self.calculate_expected_intensity(peak, curve.x_values[center_idx]);
            let actual_intensity = curve.y_values[center_idx];
            let intensity_ratio = actual_intensity / expected_intensity.max(1e-6);
            
            // 强度比应在合理范围内
            if intensity_ratio < 0.1 || intensity_ratio > 10.0 {
                return false;
            }
        }
        
        true
    }
    
    /// 计算期望强度
    fn calculate_expected_intensity(&self, peak: &Peak, x: f64) -> f64 {
        match peak.peak_type {
            crate::core::data::PeakType::EMG => {
                // 简化的EMG函数计算
                let sigma = peak.sigma;
                let tau = peak.get_metadata("tau")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(sigma * 0.5);
                
                let z = (x - peak.center) / sigma - sigma / tau;
                let erfc_arg = z / (2.0_f64.sqrt());
                let erfc_value = self.approximate_erfc(erfc_arg);
                
                peak.amplitude * (sigma / tau) * 
                (sigma / (2.0 * tau) - (x - peak.center) / tau).exp() * 
                erfc_value
            }
            _ => {
                // 默认高斯函数
                let exponent = -((x - peak.center).powi(2)) / (2.0 * peak.sigma.powi(2));
                peak.amplitude * exponent.exp()
            }
        }
    }
    
    /// 近似erfc函数
    fn approximate_erfc(&self, x: f64) -> f64 {
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;
        
        let sign = if x >= 0.0 { 1.0 } else { -1.0 };
        let x = x.abs();
        
        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
        
        sign * y
    }
    
    /// 寻找最接近的索引
    fn find_closest_index(&self, x_values: &[f64], target: f64) -> Option<usize> {
        let mut best_idx = 0;
        let mut min_diff = f64::INFINITY;
        
        for (i, &x) in x_values.iter().enumerate() {
            let diff = (x - target).abs();
            if diff < min_diff {
                min_diff = diff;
                best_idx = i;
            }
        }
        
        Some(best_idx)
    }
}
