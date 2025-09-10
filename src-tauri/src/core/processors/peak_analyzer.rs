//! 峰分析整合器
//! 
//! 整合峰检测和峰拟合功能，提供完整的峰分析解决方案

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::core::data::{DataContainer, ProcessingError, ProcessingResult};
use crate::core::processors::base::Processor;
use crate::core::processors::peak_detection::{create_detector, PeakDetectorEnum, PeakDetector};
use crate::core::processors::peak_fitting::{create_fitter, PeakFitterEnum, PeakFitter};
use crate::core::processors::overlapping_peaks::{create_overlapping_processor, OverlappingPeakProcessorEnum, OverlappingPeakProcessor, OverlappingPeakStrategy};

/// 峰分析整合器
#[derive(Debug)]
pub struct PeakAnalyzer {
    detector: PeakDetectorEnum,
    fitter: PeakFitterEnum,
    overlapping_processor: Option<OverlappingPeakProcessorEnum>,
}

impl PeakAnalyzer {
    /// 创建新的峰分析器
    pub fn new(detection_method: &str, fitting_method: &str) -> Result<Self, ProcessingError> {
        let detector = create_detector(detection_method)?;
        let fitter = create_fitter(fitting_method)?;
        
        Ok(Self { 
            detector, 
            fitter,
            overlapping_processor: None,
        })
    }
    
    /// 创建带重叠峰处理的峰分析器
    pub fn new_with_overlapping_processing(
        detection_method: &str, 
        fitting_method: &str,
        overlapping_method: Option<&str>
    ) -> Result<Self, ProcessingError> {
        let detector = create_detector(detection_method)?;
        let fitter = create_fitter(fitting_method)?;
        let overlapping_processor = if let Some(method) = overlapping_method {
            Some(create_overlapping_processor(method)?)
        } else {
            None
        };
        
        Ok(Self { 
            detector, 
            fitter,
            overlapping_processor,
        })
    }
}

#[async_trait]
impl Processor for PeakAnalyzer {
    fn name(&self) -> &str {
        "peak_analyzer"
    }

    fn description(&self) -> &str {
        "峰分析整合器，结合多种检测算法和拟合方法，提供完整的峰分析解决方案"
    }

    fn config_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "detection_method": {
                    "type": "string",
                    "enum": ["cwt", "simple", "peak_finder"],
                    "default": "simple",
                    "description": "峰检测方法"
                },
                "fitting_method": {
                    "type": "string", 
                    "enum": ["gaussian", "lorentzian", "pseudo_voigt", "multi_peak", "emg", "bi_gaussian", "voigt_exponential_tail", "pearson_iv", "nlc", "gmg_bayesian"],
                    "default": "gaussian",
                    "description": "峰拟合方法"
                },
                "overlapping_processing": {
                    "type": "string",
                    "enum": ["none", "fbf", "sharpen_cwt", "emg_nlls", "extreme_overlap", "auto"],
                    "default": "auto",
                    "description": "重叠峰处理方法"
                },
                "sensitivity": {
                    "type": "number",
                    "minimum": 0.0,
                    "maximum": 1.0,
                    "default": 0.5,
                    "description": "峰检测敏感度"
                },
                "threshold_multiplier": {
                    "type": "number",
                    "minimum": 1.0,
                    "default": 3.0,
                    "description": "阈值倍数（基于标准差）"
                },
                "min_peak_width": {
                    "type": "number",
                    "minimum": 0.0,
                    "default": 0.1,
                    "description": "最小峰宽"
                },
                "max_peak_width": {
                    "type": "number",
                    "minimum": 0.0,
                    "default": 10.0,
                    "description": "最大峰宽"
                },
                "cwt_min_width": {
                    "type": "integer",
                    "minimum": 1,
                    "default": 1,
                    "description": "CWT最小宽度"
                },
                "cwt_max_width": {
                    "type": "integer",
                    "minimum": 1,
                    "default": 10,
                    "description": "CWT最大宽度"
                }
            }
        })
    }

    async fn process(
        &self,
        mut input: DataContainer,
        config: Value,
    ) -> Result<ProcessingResult, ProcessingError> {
        let detection_method = config["detection_method"]
            .as_str()
            .unwrap_or("simple");
        let fitting_method = config["fitting_method"]
            .as_str()
            .unwrap_or("gaussian");
        let overlapping_processing = config["overlapping_processing"]
            .as_str()
            .unwrap_or("auto");
        let sensitivity = config["sensitivity"].as_f64().unwrap_or(0.5);
        let threshold_multiplier = config["threshold_multiplier"].as_f64().unwrap_or(3.0);

        let mut all_peaks = Vec::new();
        let mut processed_curves = Vec::new();

        // 对每条曲线进行峰分析
        for curve in input.curves.iter() {
            // 1. 峰检测（如果配置要求）
            let peaks_to_fit = if detection_method != "none" {
                let detected_peaks = self.detector.detect_peaks(curve, &config)?;
                
                // 2. 重叠峰处理（如果需要）
                if detected_peaks.len() > 1 && overlapping_processing != "none" {
                    self.process_overlapping_peaks(&detected_peaks, curve, &config, overlapping_processing)?
                } else {
                    detected_peaks
                }
            } else {
                // 如果跳过检测，使用现有的峰
                input.peaks.iter()
                    .filter(|peak| peak.curve_id == curve.id)
                    .cloned()
                    .collect()
            };
            
            // 3. 峰拟合（如果配置要求）
            let mut fitted_peaks = Vec::new();
            if fitting_method != "none" {
                for peak in &peaks_to_fit {
                    let fitted_peak = self.fitter.fit_peak(peak, curve, &config)?;
                    fitted_peaks.push(fitted_peak);
                }
            } else {
                fitted_peaks = peaks_to_fit;
            }
            
            // 4. 峰信息增强
            let enhanced_peaks = self.enhance_peak_information(&fitted_peaks, curve)?;
            
            all_peaks.extend(enhanced_peaks);
            processed_curves.push(curve.clone());
        }

        let total_peaks = all_peaks.len();

        // 更新数据容器
        input.peaks.extend(all_peaks.clone());

        Ok(ProcessingResult {
            curves: processed_curves,
            peaks: all_peaks,
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("detection_method".to_string(), serde_json::json!(detection_method));
                meta.insert("fitting_method".to_string(), serde_json::json!(fitting_method));
                meta.insert("sensitivity".to_string(), serde_json::json!(sensitivity));
                meta.insert("threshold_multiplier".to_string(), serde_json::json!(threshold_multiplier));
                meta.insert("total_peaks".to_string(), serde_json::json!(total_peaks));
                meta
            },
        })
    }
}

impl PeakAnalyzer {
    /// 处理重叠峰
    fn process_overlapping_peaks(
        &self,
        peaks: &[crate::core::data::Peak],
        curve: &crate::core::data::Curve,
        config: &Value,
        overlapping_method: &str,
    ) -> Result<Vec<crate::core::data::Peak>, ProcessingError> {
        let processor_method = if overlapping_method == "auto" {
            // 自动选择处理策略
            let strategy = OverlappingPeakStrategy::auto_select(peaks, curve);
            strategy.get_processor_method()
        } else {
            overlapping_method
        };
        
        if processor_method == "none" {
            return Ok(peaks.to_vec());
        }
        
        // 使用现有的重叠峰处理器或创建新的
        let processor = if let Some(ref existing_processor) = self.overlapping_processor {
            existing_processor
        } else {
            // 动态创建处理器
            let _new_processor = create_overlapping_processor(processor_method)?;
            // 注意：这里我们不能直接使用new_processor，因为self是不可变的
            // 在实际应用中，应该重构代码结构以支持动态处理器
            return Err(ProcessingError::process_error(
                "动态重叠峰处理器需要重构代码结构"
            ));
        };
        
        processor.process_overlapping_peaks(peaks, curve, config)
    }
    
    /// 增强峰信息
    fn enhance_peak_information(&self, peaks: &[crate::core::data::Peak], curve: &crate::core::data::Curve) -> Result<Vec<crate::core::data::Peak>, ProcessingError> {
        let mut enhanced_peaks = Vec::new();
        
        for peak in peaks {
            let mut enhanced_peak = peak.clone();
            
            // 计算左右边界
            self.calculate_peak_boundaries(&mut enhanced_peak, curve)?;
            
            // 计算拖尾信息
            self.calculate_peak_tailing(&mut enhanced_peak, curve)?;
            
            // 计算与邻近峰的分离度
            self.calculate_peak_separation(&mut enhanced_peak, peaks)?;
            
            // 计算峰质量评分
            self.calculate_peak_quality_score(&mut enhanced_peak)?;
            
            enhanced_peaks.push(enhanced_peak);
        }
        
        Ok(enhanced_peaks)
    }
    
    /// 计算峰边界
    fn calculate_peak_boundaries(&self, peak: &mut crate::core::data::Peak, curve: &crate::core::data::Curve) -> Result<(), ProcessingError> {
        let threshold = peak.amplitude * 0.1; // 10%阈值
        
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
    
    /// 计算峰拖尾
    fn calculate_peak_tailing(&self, peak: &mut crate::core::data::Peak, curve: &crate::core::data::Curve) -> Result<(), ProcessingError> {
        // 计算左右半峰宽
        let half_max = peak.amplitude / 2.0;
        
        // 寻找左半峰点
        let mut left_hwhm = 0.0;
        for (i, &x) in curve.x_values.iter().enumerate() {
            if x < peak.center && curve.y_values[i] <= half_max {
                left_hwhm = peak.center - x;
                break;
            }
        }
        
        // 寻找右半峰点
        let mut right_hwhm = 0.0;
        for (i, &x) in curve.x_values.iter().enumerate() {
            if x > peak.center && curve.y_values[i] <= half_max {
                right_hwhm = x - peak.center;
                break;
            }
        }
        
        peak.left_hwhm = left_hwhm;
        peak.right_hwhm = right_hwhm;
        peak.calculate_asymmetry_factor();
        
        Ok(())
    }
    
    /// 计算峰分离度
    fn calculate_peak_separation(&self, peak: &mut crate::core::data::Peak, all_peaks: &[crate::core::data::Peak]) -> Result<(), ProcessingError> {
        let mut min_separation = f64::INFINITY;
        
        for other_peak in all_peaks {
            if other_peak.id != peak.id {
                let distance = (other_peak.center - peak.center).abs();
                let combined_width = (peak.fwhm + other_peak.fwhm) / 2.0;
                let separation = distance / combined_width;
                
                if separation < min_separation {
                    min_separation = separation;
                }
            }
        }
        
        // 添加分离度信息到元数据
        peak.add_metadata("min_separation".to_string(), serde_json::json!(min_separation));
        peak.add_metadata("is_resolved".to_string(), serde_json::json!(min_separation > 1.0));
        
        Ok(())
    }
    
    /// 计算峰质量评分
    fn calculate_peak_quality_score(&self, peak: &mut crate::core::data::Peak) -> Result<(), ProcessingError> {
        let quality_score = peak.get_quality_score();
        
        // 添加质量评分到元数据
        peak.add_metadata("quality_score".to_string(), serde_json::json!(quality_score));
        peak.add_metadata("quality_grade".to_string(), serde_json::json!(
            if quality_score > 0.8 { "A" }
            else if quality_score > 0.6 { "B" }
            else if quality_score > 0.4 { "C" }
            else { "D" }
        ));
        
        Ok(())
    }
}
