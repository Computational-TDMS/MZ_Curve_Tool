//! 锐化+CWT预热方法
//! 
//! 实现锐化滤波和连续小波变换预热，用于极度重叠峰的处理

use crate::core::data::{Curve, Peak, ProcessingError};
use crate::core::processors::overlapping_peaks::OverlappingPeakProcessor;
use serde_json::Value;
use std::f64::consts::PI;

/// 锐化+CWT预处理器
#[derive(Debug)]
pub struct SharpenCWTPreprocessor {
    /// 锐化强度
    sharpen_strength: f64,
    /// CWT尺度范围
    cwt_scales: (usize, usize),
    /// 锐化核大小
    sharpen_kernel_size: usize,
    /// 噪声阈值
    noise_threshold: f64,
}

impl SharpenCWTPreprocessor {
    /// 创建新的锐化+CWT预处理器
    pub fn new() -> Self {
        Self {
            sharpen_strength: 1.5,
            cwt_scales: (1, 20),
            sharpen_kernel_size: 5,
            noise_threshold: 0.1,
        }
    }
    
    /// 设置参数
    pub fn with_parameters(
        mut self,
        sharpen_strength: f64,
        cwt_scales: (usize, usize),
        sharpen_kernel_size: usize,
        noise_threshold: f64,
    ) -> Self {
        self.sharpen_strength = sharpen_strength;
        self.cwt_scales = cwt_scales;
        self.sharpen_kernel_size = sharpen_kernel_size;
        self.noise_threshold = noise_threshold;
        self
    }
    
    /// 对极度重叠峰进行锐化+CWT预处理
    pub fn preprocess_extreme_overlapping_peaks(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        _config: &Value,
    ) -> Result<Vec<Peak>, ProcessingError> {
        if peaks.len() < 2 {
            return Ok(peaks.to_vec());
        }
        
        // 1. 锐化滤波
        let sharpened_curve = self.apply_sharpening_filter(curve)?;
        
        // 2. CWT分析
        let cwt_result = self.perform_cwt_analysis(&sharpened_curve)?;
        
        // 3. 基于CWT结果重新检测峰
        let enhanced_peaks = self.detect_peaks_from_cwt(&sharpened_curve, &cwt_result, peaks)?;
        
        // 4. 峰分离和优化
        let separated_peaks = self.separate_and_optimize_peaks(&enhanced_peaks, &sharpened_curve)?;
        
        Ok(separated_peaks)
    }
    
    /// 应用锐化滤波
    fn apply_sharpening_filter(&self, curve: &Curve) -> Result<Curve, ProcessingError> {
        let mut sharpened_y = curve.y_values.clone();
        
        // 创建锐化核
        let kernel = self.create_sharpening_kernel();
        let kernel_size = kernel.len();
        let half_kernel = kernel_size / 2;
        
        // 应用锐化滤波
        for i in half_kernel..(curve.y_values.len() - half_kernel) {
            let mut sum = 0.0;
            for (j, &weight) in kernel.iter().enumerate() {
                let idx = i - half_kernel + j;
                sum += curve.y_values[idx] * weight;
            }
            sharpened_y[i] = curve.y_values[i] + self.sharpen_strength * sum;
        }
        
        // 确保非负值
        for y in &mut sharpened_y {
            *y = y.max(0.0);
        }
        
        Ok(Curve {
            id: format!("{}_sharpened", curve.id),
            curve_type: curve.curve_type.clone(),
            x_values: curve.x_values.clone(),
            y_values: sharpened_y,
            x_label: curve.x_label.clone(),
            y_label: curve.y_label.clone(),
            x_unit: curve.x_unit.clone(),
            y_unit: curve.y_unit.clone(),
            metadata: curve.metadata.clone(),
            ..curve.clone()
        })
    }
    
    /// 创建锐化核
    fn create_sharpening_kernel(&self) -> Vec<f64> {
        let size = self.sharpen_kernel_size;
        let mut kernel = vec![0.0; size];
        
        // 拉普拉斯锐化核
        match size {
            3 => {
                kernel[0] = 0.0;
                kernel[1] = -1.0;
                kernel[2] = 0.0;
            }
            5 => {
                kernel[0] = 0.0;
                kernel[1] = -1.0;
                kernel[2] = 4.0;
                kernel[3] = -1.0;
                kernel[4] = 0.0;
            }
            _ => {
                // 通用拉普拉斯核
                let center = size / 2;
                kernel[center] = (size - 1) as f64;
                for i in 0..size {
                    if i != center {
                        kernel[i] = -1.0;
                    }
                }
            }
        }
        
        kernel
    }
    
    /// 执行CWT分析
    fn perform_cwt_analysis(&self, curve: &Curve) -> Result<Vec<Vec<f64>>, ProcessingError> {
        let mut cwt_result = Vec::new();
        
        // 简化的CWT实现（实际应用中应使用专业的CWT库）
        for scale in self.cwt_scales.0..=self.cwt_scales.1 {
            let mut scale_result = Vec::new();
            
            // 创建小波核
            let wavelet_kernel = self.create_morlet_wavelet(scale);
            let kernel_size = wavelet_kernel.len();
            let half_kernel = kernel_size / 2;
            
            // 应用小波变换
            for i in half_kernel..(curve.y_values.len() - half_kernel) {
                let mut cwt_value = 0.0;
                for (j, &weight) in wavelet_kernel.iter().enumerate() {
                    let idx = i - half_kernel + j;
                    cwt_value += curve.y_values[idx] * weight;
                }
                scale_result.push(cwt_value);
            }
            
            cwt_result.push(scale_result);
        }
        
        Ok(cwt_result)
    }
    
    /// 创建Morlet小波
    fn create_morlet_wavelet(&self, scale: usize) -> Vec<f64> {
        let kernel_size = scale * 6 + 1; // 确保核足够大
        let mut kernel = vec![0.0; kernel_size];
        let center = kernel_size / 2;
        
        let sigma = scale as f64 / 2.0;
        let omega = 2.0 * PI / scale as f64;
        
        for i in 0..kernel_size {
            let x = (i as f64 - center as f64) / sigma;
            let gaussian = (-x * x / 2.0).exp();
            let morlet = gaussian * (omega * x).cos();
            kernel[i] = morlet;
        }
        
        // 归一化
        let sum: f64 = kernel.iter().map(|&x| x.abs()).sum();
        if sum > 0.0 {
            for val in &mut kernel {
                *val /= sum;
            }
        }
        
        kernel
    }
    
    /// 基于CWT结果检测峰
    fn detect_peaks_from_cwt(
        &self,
        curve: &Curve,
        cwt_result: &[Vec<f64>],
        original_peaks: &[Peak],
    ) -> Result<Vec<Peak>, ProcessingError> {
        let mut enhanced_peaks = Vec::new();
        
        // 计算CWT响应的最大值
        let mut max_cwt = 0.0_f64;
        for scale_result in cwt_result {
            for &value in scale_result {
                max_cwt = max_cwt.max(value.abs());
            }
        }
        
        let threshold = max_cwt * self.noise_threshold;
        
        // 为每个原始峰寻找增强的CWT响应
        for original_peak in original_peaks {
            let mut best_scale = 0;
            let mut max_response = 0.0;
            let mut best_position = original_peak.center;
            
            // 在峰中心附近寻找最大CWT响应
            let search_range = (original_peak.fwhm * 2.0) as usize;
            let center_idx = self.find_closest_index(&curve.x_values, original_peak.center);
            
            for scale_idx in 0..cwt_result.len() {
                let scale_result = &cwt_result[scale_idx];
                
                for offset in 0..search_range {
                    let left_idx = center_idx.saturating_sub(offset);
                    let right_idx = (center_idx + offset).min(scale_result.len() - 1);
                    
                    for &idx in &[left_idx, right_idx] {
                        if idx < scale_result.len() && scale_result[idx].abs() > max_response {
                            max_response = scale_result[idx].abs();
                            best_scale = scale_idx;
                            best_position = curve.x_values[idx];
                        }
                    }
                }
            }
            
            if max_response > threshold {
                let mut enhanced_peak = original_peak.clone();
                enhanced_peak.center = best_position;
                enhanced_peak.amplitude = enhanced_peak.amplitude * (1.0 + max_response / max_cwt);
                
                // 添加CWT增强元数据
                enhanced_peak.add_metadata("cwt_enhanced".to_string(), serde_json::json!(true));
                enhanced_peak.add_metadata("cwt_scale".to_string(), serde_json::json!(best_scale + self.cwt_scales.0));
                enhanced_peak.add_metadata("cwt_response".to_string(), serde_json::json!(max_response));
                enhanced_peak.add_metadata("cwt_enhancement_factor".to_string(), serde_json::json!(1.0 + max_response / max_cwt));
                
                enhanced_peaks.push(enhanced_peak);
            } else {
                // 如果CWT响应不足，保留原始峰但标记
                let mut weak_peak = original_peak.clone();
                weak_peak.add_metadata("cwt_enhanced".to_string(), serde_json::json!(false));
                weak_peak.add_metadata("cwt_response".to_string(), serde_json::json!(max_response));
                enhanced_peaks.push(weak_peak);
            }
        }
        
        Ok(enhanced_peaks)
    }
    
    /// 分离和优化峰
    fn separate_and_optimize_peaks(
        &self,
        peaks: &[Peak],
        curve: &Curve,
    ) -> Result<Vec<Peak>, ProcessingError> {
        let mut optimized_peaks = Vec::new();
        
        for peak in peaks {
            // 基于CWT增强结果优化峰参数
            let optimized_peak = self.optimize_peak_parameters(peak, curve)?;
            optimized_peaks.push(optimized_peak);
        }
        
        // 移除重复或质量差的峰
        let filtered_peaks = self.filter_peaks_by_quality(&optimized_peaks);
        
        Ok(filtered_peaks)
    }
    
    /// 优化峰参数
    fn optimize_peak_parameters(&self, peak: &Peak, curve: &Curve) -> Result<Peak, ProcessingError> {
        let mut optimized_peak = peak.clone();
        
        // 基于CWT响应调整峰宽
        if let Some(cwt_response) = peak.get_metadata("cwt_response") {
            if let Some(response_value) = cwt_response.as_f64() {
                if let Some(max_cwt) = peak.get_metadata("max_cwt_response") {
                    if let Some(max_value) = max_cwt.as_f64() {
                        let enhancement_factor = response_value / max_value;
                        optimized_peak.fwhm *= (1.0 + enhancement_factor * 0.2).min(2.0);
                        optimized_peak.sigma = optimized_peak.fwhm / 2.355;
                    }
                }
            }
        }
        
        // 重新计算峰边界
        self.recalculate_peak_boundaries(&mut optimized_peak, curve)?;
        
        Ok(optimized_peak)
    }
    
    /// 重新计算峰边界
    fn recalculate_peak_boundaries(&self, peak: &mut Peak, curve: &Curve) -> Result<(), ProcessingError> {
        let threshold = peak.amplitude * 0.05; // 5%阈值
        
        // 寻找左边界
        let mut left_boundary = peak.center;
        for (i, &x) in curve.x_values.iter().enumerate() {
            if x < peak.center && curve.y_values[i] <= threshold {
                left_boundary = x;
                break;
            }
        }
        
        // 寻找右边界
        let mut right_boundary = peak.center;
        for (i, &x) in curve.x_values.iter().enumerate() {
            if x > peak.center && curve.y_values[i] <= threshold {
                right_boundary = x;
                break;
            }
        }
        
        peak.left_boundary = left_boundary;
        peak.right_boundary = right_boundary;
        peak.calculate_peak_span();
        
        Ok(())
    }
    
    /// 根据质量过滤峰
    fn filter_peaks_by_quality(&self, peaks: &[Peak]) -> Vec<Peak> {
        let mut filtered_peaks = Vec::new();
        
        for peak in peaks {
            // 检查峰质量
            let quality_score = peak.get_quality_score();
            let is_cwt_enhanced = peak.get_metadata("cwt_enhanced")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            
            // 保留高质量峰或CWT增强的峰
            if quality_score > 0.3 || is_cwt_enhanced {
                filtered_peaks.push(peak.clone());
            }
        }
        
        filtered_peaks
    }
    
    /// 寻找最接近的索引
    fn find_closest_index(&self, x_values: &[f64], target: f64) -> usize {
        let mut best_idx = 0;
        let mut min_diff = f64::INFINITY;
        
        for (i, &x) in x_values.iter().enumerate() {
            let diff = (x - target).abs();
            if diff < min_diff {
                min_diff = diff;
                best_idx = i;
            }
        }
        
        best_idx
    }
}

impl OverlappingPeakProcessor for SharpenCWTPreprocessor {
    fn name(&self) -> &str {
        "sharpen_cwt_preprocessor"
    }

    fn process_overlapping_peaks(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        config: &Value,
    ) -> Result<Vec<Peak>, ProcessingError> {
        self.preprocess_extreme_overlapping_peaks(peaks, curve, config)
    }
}
