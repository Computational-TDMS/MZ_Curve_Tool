//! 简单峰检测器
//! 
//! 基于局部最大值和阈值的简单峰检测算法

use crate::core::data::{Curve, Peak, ProcessingError, PeakType, DetectionAlgorithm};
use crate::core::processors::peak_detection::PeakDetector;
use serde_json::Value;
use uuid::Uuid;

/// 简单峰检测器
#[derive(Debug)]
pub struct SimpleDetector;

impl PeakDetector for SimpleDetector {
    fn name(&self) -> &str {
        "simple_detector"
    }

    fn detect_peaks(&self, curve: &Curve, config: &Value) -> Result<Vec<Peak>, ProcessingError> {
        let sensitivity = config["sensitivity"].as_f64().unwrap_or(0.5);
        let threshold_multiplier = config["threshold_multiplier"].as_f64().unwrap_or(3.0);
        let min_peak_width = config["min_peak_width"].as_f64().unwrap_or(0.1);

        let mut peaks = Vec::new();
        let max_intensity: f64 = curve.y_values.iter().fold(0.0, |a, &b| a.max(b));
        let threshold = curve.baseline_intensity + threshold_multiplier * curve.intensity_std;
        let dynamic_threshold = max_intensity * sensitivity;

        // 使用滑动窗口检测峰值
        let window_size = (min_peak_width * curve.point_count as f64 / (curve.x_max - curve.x_min)) as usize;
        let window_size = window_size.max(3);

        for i in window_size..(curve.y_values.len() - window_size) {
            let current = curve.y_values[i];
            
            // 检查是否超过阈值
            if current < threshold && current < dynamic_threshold {
                continue;
            }

            // 检查是否为局部最大值
            let mut is_peak = true;
            for j in (i - window_size)..i {
                if curve.y_values[j] >= current {
                    is_peak = false;
                    break;
                }
            }
            
            if !is_peak {
                continue;
            }
            
            for j in (i + 1)..(i + window_size + 1) {
                if j < curve.y_values.len() && curve.y_values[j] >= current {
                    is_peak = false;
                    break;
                }
            }

            if is_peak {
                let mut peak = Peak::new(
                    format!("peak_{}", Uuid::new_v4()),
                    curve.id.clone(),
                    curve.x_values[i],
                    current,
                    PeakType::Gaussian, // 默认类型，后续会通过拟合确定
                );
                
                peak.set_detection_parameters(
                    DetectionAlgorithm::Simple,
                    threshold.max(dynamic_threshold),
                    0.8
                );
                
                // 计算基本参数
                peak.fwhm = self.calculate_fwhm_simple(curve, i);
                peak.area = self.calculate_peak_area_simple(curve, i);
                
                peaks.push(peak);
            }
        }

        Ok(peaks)
    }
}

impl SimpleDetector {
    /// 计算半峰全宽（简化版本）
    fn calculate_fwhm_simple(&self, curve: &Curve, peak_index: usize) -> f64 {
        let peak_intensity = curve.y_values[peak_index];
        let half_max = peak_intensity / 2.0;

        let mut left_index = peak_index;
        let mut right_index = peak_index;

        // 寻找左半峰点
        for i in (0..peak_index).rev() {
            if curve.y_values[i] <= half_max {
                left_index = i;
                break;
            }
        }

        // 寻找右半峰点
        for i in (peak_index + 1)..curve.y_values.len() {
            if curve.y_values[i] <= half_max {
                right_index = i;
                break;
            }
        }

        curve.x_values[right_index] - curve.x_values[left_index]
    }

    /// 计算峰值面积（简化版本）
    fn calculate_peak_area_simple(&self, curve: &Curve, peak_index: usize) -> f64 {
        let peak_intensity = curve.y_values[peak_index];
        let threshold = peak_intensity * 0.1; // 10%阈值

        let mut start_index = peak_index;
        let mut end_index = peak_index;

        // 寻找峰值边界
        for i in (0..peak_index).rev() {
            if curve.y_values[i] <= threshold {
                start_index = i;
                break;
            }
        }

        for i in (peak_index + 1)..curve.y_values.len() {
            if curve.y_values[i] <= threshold {
                end_index = i;
                break;
            }
        }

        // 梯形积分
        let mut area = 0.0;
        for i in start_index..end_index {
            if i + 1 < curve.y_values.len() {
                let dx = curve.x_values[i + 1] - curve.x_values[i];
                let avg_y = (curve.y_values[i] + curve.y_values[i + 1]) / 2.0;
                area += dx * avg_y;
            }
        }

        area
    }
}
