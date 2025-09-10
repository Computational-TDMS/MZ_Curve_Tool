use crate::core::data::Curve;
use super::{BaselineAlgorithm, BaselineConfig, BaselineResult, BaselineStatistics, BaselineError, BaselineUtils};

/// 线性基线校准算法
pub struct LinearBaselineCorrector;

impl LinearBaselineCorrector {
    pub fn new() -> Self {
        Self
    }
    
    /// 计算线性基线
    fn calculate_linear_baseline(&self, curve: &Curve) -> Result<Vec<f64>, BaselineError> {
        if curve.point_count < 2 {
            return Err(BaselineError::InsufficientData {
                required: 2,
                actual: curve.point_count,
            });
        }
        
        // 使用最小二乘法拟合线性基线
        let n = curve.point_count as f64;
        let sum_x: f64 = curve.x_values.iter().sum();
        let sum_y: f64 = curve.y_values.iter().sum();
        let sum_xy: f64 = curve.x_values.iter()
            .zip(curve.y_values.iter())
            .map(|(x, y)| x * y)
            .sum();
        let sum_x2: f64 = curve.x_values.iter().map(|x| x * x).sum();
        
        // 计算斜率和截距
        let denominator = n * sum_x2 - sum_x * sum_x;
        if denominator.abs() < 1e-10 {
            // 如果分母接近零，使用水平基线
            let baseline_value = sum_y / n;
            return Ok(vec![baseline_value; curve.point_count]);
        }
        
        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;
        
        // 生成基线数据
        let baseline: Vec<f64> = curve.x_values.iter()
            .map(|&x| slope * x + intercept)
            .collect();
        
        Ok(baseline)
    }
}

impl BaselineAlgorithm for LinearBaselineCorrector {
    fn name(&self) -> &str {
        "Linear Baseline Correction"
    }
    
    fn description(&self) -> &str {
        "Fits a linear baseline using least squares regression and subtracts it from the signal"
    }
    
    fn validate_config(&self, config: &BaselineConfig) -> Result<(), BaselineError> {
        match config.method {
            super::BaselineMethod::Linear => Ok(()),
            _ => Err(BaselineError::InvalidConfig(
                "Linear baseline corrector only supports Linear method".to_string()
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
        
        // 计算基线
        let baseline_values = self.calculate_linear_baseline(curve)?;
        
        // 计算校准后的数据
        let corrected_y_values: Vec<f64> = curve.y_values.iter()
            .zip(baseline_values.iter())
            .map(|(original, baseline)| (original - baseline).max(0.0))
            .collect();
        
        // 创建校准后的曲线
        let mut corrected_curve = curve.clone();
        corrected_curve.y_values = corrected_y_values.clone();
        corrected_curve.baseline_correction = Some("Linear".to_string());
        
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
            method_used: self.name().to_string(),
            processing_time_ms: processing_time,
        };
        
        Ok(BaselineResult {
            corrected_curve,
            baseline_curve,
            statistics,
        })
    }
}

impl Default for LinearBaselineCorrector {
    fn default() -> Self {
        Self::new()
    }
}
