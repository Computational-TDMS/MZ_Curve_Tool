use crate::core::data::Curve;
use super::{BaselineAlgorithm, BaselineConfig, BaselineResult, BaselineStatistics, BaselineError, BaselineUtils};

/// 移动平均基线校准算法
pub struct MovingAverageBaselineCorrector;

impl MovingAverageBaselineCorrector {
    pub fn new() -> Self {
        Self
    }
    
    /// 计算移动平均基线
    fn calculate_moving_average_baseline(
        &self,
        curve: &Curve,
        window_size: usize,
    ) -> Result<Vec<f64>, BaselineError> {
        if window_size < 3 {
            return Err(BaselineError::InvalidWindowSize { window_size });
        }
        
        if window_size > curve.point_count {
            return Err(BaselineError::InvalidWindowSize { window_size });
        }
        
        let mut baseline = vec![0.0; curve.point_count];
        let half_window = window_size / 2;
        
        for i in 0..curve.point_count {
            let start = if i < half_window {
                0
            } else {
                i - half_window
            };
            
            let end = if i + half_window >= curve.point_count {
                curve.point_count
            } else {
                i + half_window + 1
            };
            
            // 计算窗口内的平均值
            let sum: f64 = curve.y_values[start..end].iter().sum();
            let count = end - start;
            baseline[i] = sum / count as f64;
        }
        
        Ok(baseline)
    }
    
    /// 计算加权移动平均基线（使用高斯权重）
    fn calculate_weighted_moving_average_baseline(
        &self,
        curve: &Curve,
        window_size: usize,
    ) -> Result<Vec<f64>, BaselineError> {
        if window_size < 3 {
            return Err(BaselineError::InvalidWindowSize { window_size });
        }
        
        if window_size > curve.point_count {
            return Err(BaselineError::InvalidWindowSize { window_size });
        }
        
        let mut baseline = vec![0.0; curve.point_count];
        let half_window = window_size / 2;
        let sigma = (window_size as f64) / 6.0; // 3-sigma rule
        
        for i in 0..curve.point_count {
            let mut weighted_sum = 0.0;
            let mut weight_sum = 0.0;
            
            let start = if i < half_window {
                0
            } else {
                i - half_window
            };
            
            let end = if i + half_window >= curve.point_count {
                curve.point_count
            } else {
                i + half_window + 1
            };
            
            for j in start..end {
                let distance = (j as f64 - i as f64).abs();
                let weight = (-distance * distance / (2.0 * sigma * sigma)).exp();
                
                weighted_sum += curve.y_values[j] * weight;
                weight_sum += weight;
            }
            
            baseline[i] = if weight_sum > 0.0 {
                weighted_sum / weight_sum
            } else {
                curve.y_values[i]
            };
        }
        
        Ok(baseline)
    }
    
    /// 计算自适应移动平均基线
    fn calculate_adaptive_moving_average_baseline(
        &self,
        curve: &Curve,
        base_window_size: usize,
    ) -> Result<Vec<f64>, BaselineError> {
        if base_window_size < 3 {
            return Err(BaselineError::InvalidWindowSize { window_size: base_window_size });
        }
        
        let mut baseline = vec![0.0; curve.point_count];
        
        for i in 0..curve.point_count {
            // 根据局部方差调整窗口大小
            let local_variance = self.calculate_local_variance(curve, i, base_window_size);
            let adaptive_window = if local_variance > curve.intensity_std {
                (base_window_size as f64 * 1.5) as usize
            } else {
                base_window_size
            };
            
            let half_window = adaptive_window / 2;
            let start = if i < half_window {
                0
            } else {
                i - half_window
            };
            
            let end = if i + half_window >= curve.point_count {
                curve.point_count
            } else {
                i + half_window + 1
            };
            
            // 计算窗口内的中位数（更鲁棒）
            let mut window_values: Vec<f64> = curve.y_values[start..end].to_vec();
            window_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let median = if window_values.len() % 2 == 0 {
                (window_values[window_values.len() / 2 - 1] + window_values[window_values.len() / 2]) / 2.0
            } else {
                window_values[window_values.len() / 2]
            };
            
            baseline[i] = median;
        }
        
        Ok(baseline)
    }
    
    /// 计算局部方差
    fn calculate_local_variance(&self, curve: &Curve, center: usize, window_size: usize) -> f64 {
        let half_window = window_size / 2;
        let start = if center < half_window {
            0
        } else {
            center - half_window
        };
        
        let end = if center + half_window >= curve.point_count {
            curve.point_count
        } else {
            center + half_window + 1
        };
        
        if end <= start {
            return 0.0;
        }
        
        let window_values = &curve.y_values[start..end];
        let mean: f64 = window_values.iter().sum::<f64>() / window_values.len() as f64;
        let variance: f64 = window_values.iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / window_values.len() as f64;
        
        variance
    }
}

impl BaselineAlgorithm for MovingAverageBaselineCorrector {
    fn name(&self) -> &str {
        "Moving Average Baseline Correction"
    }
    
    fn description(&self) -> &str {
        "Uses moving average to estimate baseline and subtracts it from the signal"
    }
    
    fn validate_config(&self, config: &BaselineConfig) -> Result<(), BaselineError> {
        match &config.method {
            super::BaselineMethod::MovingAverage { window_size } => {
                if *window_size < 3 {
                    Err(BaselineError::InvalidWindowSize { window_size: *window_size })
                } else {
                    Ok(())
                }
            }
            _ => Err(BaselineError::InvalidConfig(
                "Moving average baseline corrector only supports MovingAverage method".to_string()
            )),
        }
    }
    
    fn correct_baseline(
        &self,
        curve: &Curve,
        config: &BaselineConfig,
    ) -> Result<BaselineResult, BaselineError> {
        let start_time = std::time::Instant::now();
        
        // 验证配置
        self.validate_config(config)?;
        
        // 获取窗口大小
        let window_size = match &config.method {
            super::BaselineMethod::MovingAverage { window_size } => *window_size,
            _ => return Err(BaselineError::InvalidConfig("Invalid method".to_string())),
        };
        
        // 检查窗口大小是否合理
        if window_size > curve.point_count {
            return Err(BaselineError::InvalidWindowSize { window_size });
        }
        
        // 根据配置选择算法
        let baseline_values = if let Some(algorithm) = config.custom_params.get("algorithm") {
            match algorithm.as_str().unwrap_or("simple") {
                "weighted" => self.calculate_weighted_moving_average_baseline(curve, window_size)?,
                "adaptive" => self.calculate_adaptive_moving_average_baseline(curve, window_size)?,
                _ => self.calculate_moving_average_baseline(curve, window_size)?,
            }
        } else {
            self.calculate_moving_average_baseline(curve, window_size)?
        };
        
        // 计算校准后的数据
        let corrected_y_values: Vec<f64> = curve.y_values.iter()
            .zip(baseline_values.iter())
            .map(|(original, baseline)| (original - baseline).max(0.0))
            .collect();
        
        // 创建校准后的曲线
        let mut corrected_curve = curve.clone();
        corrected_curve.y_values = corrected_y_values.clone();
        corrected_curve.baseline_correction = Some(format!("Moving Average (window {})", window_size));
        
        // 重新计算统计信息
        corrected_curve.y_min = corrected_y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        corrected_curve.y_max = corrected_y_values.iter().fold(0.0, |a, &b| a.max(b));
        corrected_curve.mean_intensity = corrected_y_values.iter().sum::<f64>() / corrected_y_values.len() as f64;
        corrected_curve.baseline_intensity = corrected_curve.y_min;
        corrected_curve.calculate_signal_to_noise();
        
        // 创建基线曲线（如果需要）
        let baseline_curve = if config.output_baseline {
            let mut baseline_curve = curve.clone();
            baseline_curve.id = format!("{}_baseline", curve.id);
            baseline_curve.curve_type = "Baseline".to_string();
            baseline_curve.y_values = baseline_values;
            baseline_curve.y_label = "Baseline Intensity".to_string();
            Some(baseline_curve)
        } else {
            None
        };
        
        // 计算统计信息
        let original_baseline = curve.baseline_intensity;
        let corrected_baseline = corrected_curve.baseline_intensity;
        let baseline_offset = original_baseline - corrected_baseline;
        
        let rmse = BaselineUtils::calculate_rmse(&curve.y_values, &corrected_y_values);
        let quality_score = (1.0 / (1.0 + rmse / curve.mean_intensity)).min(1.0);
        
        let processing_time = start_time.elapsed().as_millis() as u64;
        
        let statistics = BaselineStatistics {
            original_baseline,
            corrected_baseline,
            baseline_offset,
            quality_score,
            method_used: format!("Moving Average (window {})", window_size),
            processing_time_ms: processing_time,
        };
        
        Ok(BaselineResult {
            corrected_curve,
            baseline_curve,
            statistics,
        })
    }
}

impl Default for MovingAverageBaselineCorrector {
    fn default() -> Self {
        Self::new()
    }
}
