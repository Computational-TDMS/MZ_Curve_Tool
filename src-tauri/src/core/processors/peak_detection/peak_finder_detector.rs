//! PeakFinder峰检测器
//! 
//! 基于peak_finder库的峰检测算法

use crate::core::data::{Curve, Peak, ProcessingError, PeakType, DetectionAlgorithm};
use crate::core::processors::peak_detection::PeakDetector;
use serde_json::Value;
use uuid::Uuid;

/// PeakFinder峰检测器
#[derive(Debug)]
pub struct PeakFinderDetector;

impl PeakDetector for PeakFinderDetector {
    fn name(&self) -> &str {
        "peak_finder_detector"
    }

    fn detect_peaks(&self, curve: &Curve, config: &Value) -> Result<Vec<Peak>, ProcessingError> {
        let threshold_multiplier = config["threshold_multiplier"].as_f64().unwrap_or(3.0);
        
        // 计算阈值
        let mean = curve.mean_intensity;
        let std = curve.intensity_std;
        let threshold = mean + threshold_multiplier * std;

        // 简化的peak_finder实现（不依赖外部库）
        let peak_indices = self.find_peaks_simple(&curve.y_values, threshold);

        let mut detected_peaks = Vec::new();
        for &peak_idx in &peak_indices {
            let mut detected_peak = Peak::new(
                format!("peak_{}", Uuid::new_v4()),
                curve.id.clone(),
                curve.x_values[peak_idx],
                curve.y_values[peak_idx],
                PeakType::Gaussian,
            );
            
            detected_peak.set_detection_parameters(
                DetectionAlgorithm::PeakFinder,
                threshold,
                0.9
            );
            
            // 计算基本参数
            detected_peak.fwhm = self.calculate_fwhm_simple(curve, peak_idx);
            detected_peak.area = self.calculate_peak_area_simple(curve, peak_idx);
            
            detected_peaks.push(detected_peak);
        }

        Ok(detected_peaks)
    }
}

impl PeakFinderDetector {
    /// 简化的峰值查找实现
    fn find_peaks_simple(&self, signal: &[f64], threshold: f64) -> Vec<usize> {
        let mut peaks = Vec::new();
        
        for i in 1..signal.len() - 1 {
            if signal[i] > threshold && 
               signal[i] > signal[i - 1] && 
               signal[i] > signal[i + 1] {
                peaks.push(i);
            }
        }
        
        peaks
    }

    /// 计算半峰全宽
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

    /// 计算峰值面积
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
