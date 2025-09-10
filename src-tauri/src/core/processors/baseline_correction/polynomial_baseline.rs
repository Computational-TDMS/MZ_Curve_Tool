use crate::core::data::Curve;
use super::{BaselineAlgorithm, BaselineConfig, BaselineResult, BaselineStatistics, BaselineError, BaselineUtils};

/// 多项式基线校准算法
pub struct PolynomialBaselineCorrector;

impl PolynomialBaselineCorrector {
    pub fn new() -> Self {
        Self
    }
    
    /// 计算多项式基线
    fn calculate_polynomial_baseline(
        &self,
        curve: &Curve,
        degree: u32,
    ) -> Result<Vec<f64>, BaselineError> {
        if curve.point_count < (degree + 1) as usize {
            return Err(BaselineError::InsufficientData {
                required: (degree + 1) as usize,
                actual: curve.point_count,
            });
        }
        
        if degree == 0 {
            // 零次多项式（常数）
            let baseline_value = curve.y_values.iter().sum::<f64>() / curve.point_count as f64;
            return Ok(vec![baseline_value; curve.point_count]);
        }
        
        if degree == 1 {
            // 一次多项式（线性）
            return self.calculate_linear_baseline(curve);
        }
        
        // 高次多项式使用最小二乘法
        self.fit_polynomial_least_squares(curve, degree)
    }
    
    /// 线性基线计算（一次多项式）
    fn calculate_linear_baseline(&self, curve: &Curve) -> Result<Vec<f64>, BaselineError> {
        let n = curve.point_count as f64;
        let sum_x: f64 = curve.x_values.iter().sum();
        let sum_y: f64 = curve.y_values.iter().sum();
        let sum_xy: f64 = curve.x_values.iter()
            .zip(curve.y_values.iter())
            .map(|(x, y)| x * y)
            .sum();
        let sum_x2: f64 = curve.x_values.iter().map(|x| x * x).sum();
        
        let denominator = n * sum_x2 - sum_x * sum_x;
        if denominator.abs() < 1e-10 {
            let baseline_value = sum_y / n;
            return Ok(vec![baseline_value; curve.point_count]);
        }
        
        let slope = (n * sum_xy - sum_x * sum_y) / denominator;
        let intercept = (sum_y - slope * sum_x) / n;
        
        let baseline: Vec<f64> = curve.x_values.iter()
            .map(|&x| slope * x + intercept)
            .collect();
        
        Ok(baseline)
    }
    
    /// 使用最小二乘法拟合多项式
    fn fit_polynomial_least_squares(
        &self,
        curve: &Curve,
        degree: u32,
    ) -> Result<Vec<f64>, BaselineError> {
        let n = curve.point_count;
        let m = (degree + 1) as usize;
        
        // 构建范德蒙德矩阵
        let mut vandermonde = vec![vec![0.0; m]; n];
        for i in 0..n {
            for j in 0..m {
                vandermonde[i][j] = curve.x_values[i].powi(j as i32);
            }
        }
        
        // 构建正规方程 A^T * A * x = A^T * b
        let mut ata = vec![vec![0.0; m]; m];
        let mut atb = vec![0.0; m];
        
        for i in 0..m {
            for j in 0..m {
                for k in 0..n {
                    ata[i][j] += vandermonde[k][i] * vandermonde[k][j];
                }
            }
        }
        
        for i in 0..m {
            for k in 0..n {
                atb[i] += vandermonde[k][i] * curve.y_values[k];
            }
        }
        
        // 求解线性方程组（使用高斯消元法）
        let coefficients = self.solve_linear_system(&ata, &atb)?;
        
        // 计算基线值
        let baseline: Vec<f64> = curve.x_values.iter()
            .map(|&x| {
                let mut value = 0.0;
                for (j, &coeff) in coefficients.iter().enumerate() {
                    value += coeff * x.powi(j as i32);
                }
                value
            })
            .collect();
        
        Ok(baseline)
    }
    
    /// 求解线性方程组
    fn solve_linear_system(
        &self,
        matrix: &[Vec<f64>],
        rhs: &[f64],
    ) -> Result<Vec<f64>, BaselineError> {
        let n = matrix.len();
        let mut augmented = vec![vec![0.0; n + 1]; n];
        
        // 构建增广矩阵
        for i in 0..n {
            for j in 0..n {
                augmented[i][j] = matrix[i][j];
            }
            augmented[i][n] = rhs[i];
        }
        
        // 高斯消元法
        for i in 0..n {
            // 寻找主元
            let mut max_row = i;
            for k in i + 1..n {
                if augmented[k][i].abs() > augmented[max_row][i].abs() {
                    max_row = k;
                }
            }
            
            if augmented[max_row][i].abs() < 1e-10 {
                return Err(BaselineError::MathError("Singular matrix".to_string()));
            }
            
            // 交换行
            if max_row != i {
                augmented.swap(i, max_row);
            }
            
            // 消元
            for k in i + 1..n {
                let factor = augmented[k][i] / augmented[i][i];
                for j in i..=n {
                    augmented[k][j] -= factor * augmented[i][j];
                }
            }
        }
        
        // 回代
        let mut solution = vec![0.0; n];
        for i in (0..n).rev() {
            solution[i] = augmented[i][n];
            for j in i + 1..n {
                solution[i] -= augmented[i][j] * solution[j];
            }
            solution[i] /= augmented[i][i];
        }
        
        Ok(solution)
    }
}

impl BaselineAlgorithm for PolynomialBaselineCorrector {
    fn name(&self) -> &str {
        "Polynomial Baseline Correction"
    }
    
    fn description(&self) -> &str {
        "Fits a polynomial baseline using least squares regression and subtracts it from the signal"
    }
    
    fn validate_config(&self, config: &BaselineConfig) -> Result<(), BaselineError> {
        match &config.method {
            super::BaselineMethod::Polynomial { degree } => {
                if *degree > 10 {
                    Err(BaselineError::InvalidPolynomialDegree { degree: *degree })
                } else {
                    Ok(())
                }
            }
            _ => Err(BaselineError::InvalidConfig(
                "Polynomial baseline corrector only supports Polynomial method".to_string()
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
        
        // 获取多项式次数
        let degree = match &config.method {
            super::BaselineMethod::Polynomial { degree } => *degree,
            _ => return Err(BaselineError::InvalidConfig("Invalid method".to_string())),
        };
        
        // 计算基线
        let baseline_values = self.calculate_polynomial_baseline(curve, degree)?;
        
        // 计算校准后的数据
        let corrected_y_values: Vec<f64> = curve.y_values.iter()
            .zip(baseline_values.iter())
            .map(|(original, baseline)| (original - baseline).max(0.0))
            .collect();
        
        // 创建校准后的曲线
        let mut corrected_curve = curve.clone();
        corrected_curve.y_values = corrected_y_values.clone();
        corrected_curve.baseline_correction = Some(format!("Polynomial (degree {})", degree));
        
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
            method_used: format!("Polynomial (degree {})", degree),
            processing_time_ms: processing_time,
        };
        
        Ok(BaselineResult {
            corrected_curve,
            baseline_curve,
            statistics,
        })
    }
}

impl Default for PolynomialBaselineCorrector {
    fn default() -> Self {
        Self::new()
    }
}
