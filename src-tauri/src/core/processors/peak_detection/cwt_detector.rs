//! CWT峰检测器
//! 
//! 基于连续小波变换的峰检测算法

use crate::core::data::{Curve, Peak, ProcessingError, PeakType, DetectionAlgorithm};
use crate::core::processors::peak_detection::PeakDetector;
use serde_json::Value;
use uuid::Uuid;

/// CWT峰检测器
#[derive(Debug)]
pub struct CWTDetector;

impl PeakDetector for CWTDetector {
    fn name(&self) -> &str {
        "cwt_detector"
    }

    fn detect_peaks(&self, curve: &Curve, config: &Value) -> Result<Vec<Peak>, ProcessingError> {
        let min_width = config["cwt_min_width"].as_u64().unwrap_or(1) as usize;
        let max_width = config["cwt_max_width"].as_u64().unwrap_or(10) as usize;
        let sensitivity = config["sensitivity"].as_f64().unwrap_or(0.5);

        // 简化的CWT实现（不依赖外部库）
        let cwt_result = self.perform_cwt_simple(&curve.y_values, min_width, max_width)?;
        
        // 基于CWT结果的峰值检测
        self.detect_peaks_from_cwt(curve, &cwt_result, sensitivity)
    }
}

impl CWTDetector {
    /// 简化的CWT实现
    fn perform_cwt_simple(&self, signal: &[f64], min_width: usize, max_width: usize) -> Result<Vec<Vec<f64>>, ProcessingError> {
        let mut cwt_result = Vec::new();
        
        for width in min_width..=max_width {
            let mut cwt_row = Vec::new();
            
            for i in 0..signal.len() {
                let mut sum = 0.0;
                let mut weight_sum = 0.0;
                
                // 简化的Morlet小波
                for j in 0..signal.len() {
                    let distance = (i as f64 - j as f64) / width as f64;
                    let weight = (-distance.powi(2) / 2.0).exp() * (2.0 * std::f64::consts::PI * distance).cos();
                    
                    if j < signal.len() {
                        sum += signal[j] * weight;
                        weight_sum += weight.abs();
                    }
                }
                
                if weight_sum > 0.0 {
                    cwt_row.push(sum / weight_sum);
                } else {
                    cwt_row.push(0.0);
                }
            }
            
            cwt_result.push(cwt_row);
        }
        
        Ok(cwt_result)
    }

    /// 从CWT结果检测峰
    fn detect_peaks_from_cwt(&self, curve: &Curve, cwt_result: &[Vec<f64>], sensitivity: f64) -> Result<Vec<Peak>, ProcessingError> {
        let mut peaks = Vec::new();
        
        if cwt_result.is_empty() {
            return Ok(peaks);
        }

        // 计算CWT响应的最大值
        let mut max_cwt: f64 = 0.0;
        for row in cwt_result {
            for &value in row {
                max_cwt = max_cwt.max(value.abs());
            }
        }
        
        let threshold = max_cwt * sensitivity;

        // 在CWT结果中寻找峰值
        for i in 1..curve.y_values.len() - 1 {
            let mut max_response = 0.0;
            let mut best_scale = 0;
            
            // 找到最大响应的尺度
            for (scale_idx, row) in cwt_result.iter().enumerate() {
                if i < row.len() && row[i].abs() > max_response {
                    max_response = row[i].abs();
                    best_scale = scale_idx;
                }
            }
            
            if max_response > threshold {
                // 检查是否为局部最大值
                let mut is_peak = true;
                for j in (i.saturating_sub(2))..i {
                    if j < cwt_result[best_scale].len() && cwt_result[best_scale][j] >= cwt_result[best_scale][i] {
                        is_peak = false;
                        break;
                    }
                }
                
                if is_peak {
                    for j in (i + 1)..((i + 3).min(cwt_result[best_scale].len())) {
                        if cwt_result[best_scale][j] >= cwt_result[best_scale][i] {
                            is_peak = false;
                            break;
                        }
                    }
                }
                
                if is_peak {
                    let mut peak = Peak::new(
                        format!("peak_{}", Uuid::new_v4()),
                        curve.id.clone(),
                        curve.x_values[i],
                        curve.y_values[i],
                        PeakType::Gaussian,
                    );
                    
                    peak.set_detection_parameters(
                        DetectionAlgorithm::CWT,
                        threshold,
                        0.95
                    );
                    
                    // 添加CWT相关信息
                    peak.add_metadata("cwt_scale".to_string(), serde_json::json!(best_scale));
                    peak.add_metadata("cwt_response".to_string(), serde_json::json!(max_response));
                    
                    peaks.push(peak);
                }
            }
        }

        Ok(peaks)
    }
}
