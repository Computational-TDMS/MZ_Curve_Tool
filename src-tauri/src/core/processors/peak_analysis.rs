//! 峰分析模块
//! 
//! 使用新的统一架构实现峰分析功能

use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::core::data::{DataContainer, ProcessingError, ProcessingResult};
use crate::core::processors::core::{Processor, ProcessorType, ProcessorConfig};

/// 峰分析器
#[derive(Debug)]
pub struct PeakAnalyzer {
    name: String,
}

impl PeakAnalyzer {
    pub fn new() -> Self {
        Self {
            name: "peak_analyzer".to_string(),
        }
    }
    
    /// 创建峰分析器实例
    pub fn create(method: &str) -> Result<std::sync::Arc<dyn Processor>, ProcessingError> {
        match method {
            "smart" => Ok(std::sync::Arc::new(Self::new())),
            "basic" => Ok(std::sync::Arc::new(Self::new())),
            _ => Err(ProcessingError::ConfigError(format!("不支持的峰分析方法: {}", method))),
        }
    }
}

#[async_trait]
impl Processor for PeakAnalyzer {
    fn name(&self) -> &str {
        &self.name
    }
    
    fn description(&self) -> &str {
        "智能峰分析器，自动选择最佳的检测和拟合策略"
    }
    
    fn processor_type(&self) -> ProcessorType {
        ProcessorType::PeakAnalysis
    }
    
    fn supported_methods(&self) -> Vec<String> {
        vec![
            "smart".to_string(),
            "basic".to_string(),
        ]
    }
    
    fn config_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "detection_method": {
                    "type": "string",
                    "enum": ["auto", "simple", "cwt", "peak_finder"],
                    "default": "auto",
                    "description": "峰检测方法"
                },
                "fitting_method": {
                    "type": "string",
                    "enum": ["auto", "gaussian", "lorentzian", "emg", "adaptive_hybrid"],
                    "default": "auto",
                    "description": "峰拟合方法"
                },
                "overlapping_processing": {
                    "type": "string",
                    "enum": ["auto", "none", "fbf", "sharpen_cwt", "extreme_overlap"],
                    "default": "auto",
                    "description": "重叠峰处理方法"
                },
                "sensitivity": {
                    "type": "number",
                    "minimum": 0.0,
                    "maximum": 1.0,
                    "default": 0.5,
                    "description": "检测敏感度"
                },
                "quality_threshold": {
                    "type": "number",
                    "minimum": 0.0,
                    "maximum": 1.0,
                    "default": 0.7,
                    "description": "峰质量阈值"
                }
            }
        })
    }
    
    async fn process(
        &self,
        input: DataContainer,
        config: serde_json::Value,
    ) -> Result<ProcessingResult, ProcessingError> {
        let detection_method = config.get("detection_method")
            .and_then(|v| v.as_str())
            .unwrap_or("auto")
            .to_string();
        let fitting_method = config.get("fitting_method")
            .and_then(|v| v.as_str())
            .unwrap_or("auto")
            .to_string();
        let overlapping_processing = config.get("overlapping_processing")
            .and_then(|v| v.as_str())
            .unwrap_or("auto")
            .to_string();
        let sensitivity = config.get("sensitivity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.5);
        let quality_threshold = config.get("quality_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.7);
        
        let mut result_curves = Vec::new();
        let mut result_peaks = Vec::new();
        let mut metadata = HashMap::new();
        
        // 对每条曲线进行峰分析
        for curve in input.curves.iter() {
            // 1. 峰检测
            let detected_peaks = self.detect_peaks(curve, &detection_method, sensitivity).await?;
            
            // 2. 重叠峰处理
            let processed_peaks = if detected_peaks.len() > 1 && overlapping_processing != "none" {
                self.process_overlapping_peaks(&detected_peaks, curve, &overlapping_processing).await?
            } else {
                detected_peaks
            };
            
            // 3. 峰拟合
            let fitted_peaks = self.fit_peaks(&processed_peaks, curve, &fitting_method).await?;
            
            // 4. 质量过滤
            let quality_peaks: Vec<_> = fitted_peaks.into_iter()
                .filter(|peak| peak.get_quality_score() >= quality_threshold)
                .collect();
            
            // 5. 增强峰信息
            let enhanced_peaks = self.enhance_peak_information(&quality_peaks, curve).await?;
            
            result_peaks.extend(enhanced_peaks);
            result_curves.push(curve.clone());
        }
        
        // 更新元数据
        metadata.insert("total_peaks".to_string(), Value::Number(serde_json::Number::from(result_peaks.len())));
        metadata.insert("detection_method".to_string(), Value::String(detection_method));
        metadata.insert("fitting_method".to_string(), Value::String(fitting_method));
        metadata.insert("quality_threshold".to_string(), Value::Number(serde_json::Number::from_f64(quality_threshold).unwrap()));
        
        Ok(ProcessingResult {
            curves: result_curves,
            peaks: result_peaks,
            metadata,
        })
    }
}

impl PeakAnalyzer {
    /// 峰检测
    async fn detect_peaks(
        &self,
        curve: &crate::core::data::Curve,
        method: &str,
        sensitivity: f64,
    ) -> Result<Vec<crate::core::data::Peak>, ProcessingError> {
        let actual_method = if method == "auto" {
            self.select_detection_method(curve)
        } else {
            method.to_string()
        };
        
        // 创建检测器配置
        let config = ProcessorConfig::new(ProcessorType::PeakDetection, actual_method)
            .with_parameter("sensitivity".to_string(), Value::Number(serde_json::Number::from_f64(sensitivity).unwrap()));
        
        // 创建检测器
        let detector = crate::core::processors::core::ProcessorFactory::create_processor(config.clone())?;
        
        // 创建输入数据容器
        let input = DataContainer {
            curves: vec![curve.clone()],
            metadata: HashMap::new(),
            spectra: vec![],
        };
        
        // 执行检测
        let result = detector.process(input, serde_json::to_value(&config)?).await?;
        if let Some(curve) = result.curves.first() {
            Ok(curve.peaks.clone())
        } else {
            Ok(vec![])
        }
    }
    
    /// 选择检测方法
    fn select_detection_method(&self, curve: &crate::core::data::Curve) -> String {
        // 分析曲线特征
        let noise_level = self.estimate_noise_level(curve);
        let signal_strength = self.estimate_signal_strength(curve);
        
        if noise_level > 0.1 {
            "cwt".to_string()
        } else if signal_strength > 0.8 {
            "peak_finder".to_string()
        } else {
            "simple".to_string()
        }
    }
    
    /// 处理重叠峰
    async fn process_overlapping_peaks(
        &self,
        peaks: &[crate::core::data::Peak],
        curve: &crate::core::data::Curve,
        method: &str,
    ) -> Result<Vec<crate::core::data::Peak>, ProcessingError> {
        if method == "none" {
            return Ok(peaks.to_vec());
        }
        
        let actual_method = if method == "auto" {
            self.select_overlapping_method(peaks, curve)
        } else {
            method.to_string()
        };
        
        // 创建重叠峰处理器配置
        let config = ProcessorConfig::new(ProcessorType::OverlappingPeaks, actual_method);
        
        // 创建处理器
        let processor = crate::core::processors::core::ProcessorFactory::create_processor(config.clone())?;
        
        // 创建输入数据容器
        let mut curve_with_peaks = curve.clone();
        curve_with_peaks.peaks = peaks.to_vec();
        let input = DataContainer {
            curves: vec![curve_with_peaks],
            metadata: HashMap::new(),
            spectra: vec![],
        };
        
        // 执行处理
        let result = processor.process(input, serde_json::to_value(&config)?).await?;
        if let Some(curve) = result.curves.first() {
            Ok(curve.peaks.clone())
        } else {
            Ok(vec![])
        }
    }
    
    /// 选择重叠峰处理方法
    fn select_overlapping_method(&self, peaks: &[crate::core::data::Peak], curve: &crate::core::data::Curve) -> String {
        let overlap_level = self.estimate_overlap_level(peaks);
        let snr = self.estimate_snr(curve);
        
        if overlap_level > 0.8 && snr < 10.0 {
            "extreme_overlap".to_string()
        } else if overlap_level > 0.5 {
            "sharpen_cwt".to_string()
        } else if overlap_level > 0.2 {
            "fbf".to_string()
        } else {
            "none".to_string()
        }
    }
    
    /// 峰拟合
    async fn fit_peaks(
        &self,
        peaks: &[crate::core::data::Peak],
        curve: &crate::core::data::Curve,
        method: &str,
    ) -> Result<Vec<crate::core::data::Peak>, ProcessingError> {
        let actual_method = if method == "auto" {
            self.select_fitting_method(peaks, curve)
        } else {
            method.to_string()
        };
        
        let mut fitted_peaks = Vec::new();
        
        for peak in peaks {
            // 创建拟合器配置
            let config = ProcessorConfig::new(ProcessorType::PeakFitting, actual_method.clone());
            
            // 创建拟合器
            let fitter = crate::core::processors::core::ProcessorFactory::create_processor(config.clone())?;
            
            // 创建输入数据容器
            let mut curve_with_peak = curve.clone();
            curve_with_peak.peaks = vec![peak.clone()];
            let input = DataContainer {
                curves: vec![curve_with_peak],
                metadata: HashMap::new(),
                spectra: vec![],
            };
            
            // 执行拟合
            let result = fitter.process(input, serde_json::to_value(&config)?).await?;
            if let Some(curve) = result.curves.first() {
                if let Some(fitted_peak) = curve.peaks.first() {
                    fitted_peaks.push(fitted_peak.clone());
                }
            }
        }
        
        Ok(fitted_peaks)
    }
    
    /// 选择拟合方法
    fn select_fitting_method(&self, peaks: &[crate::core::data::Peak], curve: &crate::core::data::Curve) -> String {
        let complexity = self.estimate_peak_complexity(peaks, curve);
        
        if complexity > 0.7 {
            "adaptive_hybrid".to_string()
        } else if complexity > 0.4 {
            "emg".to_string()
        } else {
            "gaussian".to_string()
        }
    }
    
    /// 增强峰信息
    async fn enhance_peak_information(
        &self,
        peaks: &[crate::core::data::Peak],
        curve: &crate::core::data::Curve,
    ) -> Result<Vec<crate::core::data::Peak>, ProcessingError> {
        let mut enhanced_peaks = Vec::new();
        
        for peak in peaks {
            let mut enhanced_peak = peak.clone();
            
            // 计算峰边界
            self.calculate_peak_boundaries(&mut enhanced_peak, curve)?;
            
            // 计算拖尾信息
            self.calculate_peak_tailing(&mut enhanced_peak, curve)?;
            
            // 计算分离度
            self.calculate_peak_separation(&mut enhanced_peak, peaks)?;
            
            // 计算质量评分
            self.calculate_peak_quality(&mut enhanced_peak)?;
            
            enhanced_peaks.push(enhanced_peak);
        }
        
        Ok(enhanced_peaks)
    }
    
    /// 估计噪声水平
    fn estimate_noise_level(&self, curve: &crate::core::data::Curve) -> f64 {
        if curve.y_values.is_empty() {
            return 0.0;
        }
        
        let mean: f64 = curve.y_values.iter().sum::<f64>() / curve.y_values.len() as f64;
        let variance: f64 = curve.y_values.iter()
            .map(|&y| (y - mean).powi(2))
            .sum::<f64>() / curve.y_values.len() as f64;
        
        variance.sqrt() / mean.max(1e-6)
    }
    
    /// 估计信号强度
    fn estimate_signal_strength(&self, curve: &crate::core::data::Curve) -> f64 {
        if curve.y_values.is_empty() {
            return 0.0;
        }
        
        let max_signal = curve.y_values.iter().fold(0.0_f64, |a, &b| a.max(b));
        let mean_signal: f64 = curve.y_values.iter().sum::<f64>() / curve.y_values.len() as f64;
        
        if mean_signal > 0.0 {
            max_signal / mean_signal
        } else {
            0.0
        }
    }
    
    /// 估计重叠水平
    fn estimate_overlap_level(&self, peaks: &[crate::core::data::Peak]) -> f64 {
        if peaks.len() < 2 {
            return 0.0;
        }
        
        let mut overlap_count = 0;
        let mut total_pairs = 0;
        
        for i in 0..peaks.len() {
            for j in (i + 1)..peaks.len() {
                let peak1 = &peaks[i];
                let peak2 = &peaks[j];
                
                let distance = (peak1.center - peak2.center).abs();
                let combined_width = (peak1.fwhm + peak2.fwhm) / 2.0;
                
                if distance < combined_width {
                    overlap_count += 1;
                }
                total_pairs += 1;
            }
        }
        
        if total_pairs > 0 {
            overlap_count as f64 / total_pairs as f64
        } else {
            0.0
        }
    }
    
    /// 估计信噪比
    fn estimate_snr(&self, curve: &crate::core::data::Curve) -> f64 {
        if curve.y_values.is_empty() {
            return 0.0;
        }
        
        let max_signal = curve.y_values.iter().fold(0.0_f64, |a, &b| a.max(b));
        let noise_level = self.estimate_noise_level(curve);
        
        if noise_level > 0.0 {
            max_signal / noise_level
        } else {
            0.0
        }
    }
    
    /// 估计峰复杂度
    fn estimate_peak_complexity(&self, peaks: &[crate::core::data::Peak], curve: &crate::core::data::Curve) -> f64 {
        if peaks.is_empty() {
            return 0.0;
        }
        
        let mut complexity_sum = 0.0;
        for peak in peaks {
            // 基于峰的不对称性和拖尾计算复杂度
            let asymmetry = (peak.left_hwhm - peak.right_hwhm).abs() / (peak.left_hwhm + peak.right_hwhm).max(1e-6);
            let tailing = if peak.left_hwhm > 0.0 && peak.right_hwhm > 0.0 {
                (peak.right_hwhm / peak.left_hwhm - 1.0).abs()
            } else {
                0.0
            };
            
            complexity_sum += asymmetry + tailing;
        }
        
        complexity_sum / peaks.len() as f64
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
        peak.add_metadata("min_separation".to_string(), Value::Number(serde_json::Number::from_f64(min_separation).unwrap()));
        peak.add_metadata("is_resolved".to_string(), Value::Bool(min_separation > 1.0));
        
        Ok(())
    }
    
    /// 计算峰质量
    fn calculate_peak_quality(&self, peak: &mut crate::core::data::Peak) -> Result<(), ProcessingError> {
        let quality_score = peak.get_quality_score();
        
        // 添加质量评分到元数据
        peak.add_metadata("quality_score".to_string(), Value::Number(serde_json::Number::from_f64(quality_score).unwrap()));
        peak.add_metadata("quality_grade".to_string(), Value::String(
            if quality_score > 0.8 { "A".to_string() }
            else if quality_score > 0.6 { "B".to_string() }
            else if quality_score > 0.4 { "C".to_string() }
            else { "D".to_string() }
        ));
        
        Ok(())
    }
}
