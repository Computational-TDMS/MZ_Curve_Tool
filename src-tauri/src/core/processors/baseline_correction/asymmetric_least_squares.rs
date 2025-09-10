use crate::core::data::Curve;
use super::{BaselineAlgorithm, BaselineConfig, BaselineResult, BaselineStatistics, BaselineError, BaselineUtils};

/// 非对称最小二乘法基线校准算法
pub struct AsymmetricLeastSquaresCorrector;

impl AsymmetricLeastSquaresCorrector {
    pub fn new() -> Self {
        Self
    }
    
    /// 非对称最小二乘法基线估计
    fn asymmetric_least_squares(
        &self,
        curve: &Curve,
        lambda: f64,
        p: f64,
        max_iterations: usize,
    ) -> Result<Vec<f64>, BaselineError> {
        if curve.point_count < 3 {
            return Err(BaselineError::InsufficientData {
                required: 3,
                actual: curve.point_count,
            });
        }
        
        let n = curve.point_count;
        let mut baseline = curve.y_values.clone();
        
        // 初始化权重矩阵
        let mut weights = vec![1.0; n];
        
        for _iteration in 0..max_iterations {
            // 计算新的权重
            let mut new_weights = vec![0.0; n];
            for i in 0..n {
                let residual = curve.y_values[i] - baseline[i];
                if residual > 0.0 {
                    new_weights[i] = p;
                } else {
                    new_weights[i] = 1.0 - p;
                }
            }
            
            // 检查收敛性
            let weight_change: f64 = new_weights.iter()
                .zip(weights.iter())
                .map(|(new, old)| (new - old).abs())
                .sum();
            
            if weight_change < 1e-6 {
                break;
            }
            
            weights = new_weights;
            
            // 使用加权最小二乘法拟合基线
            baseline = self.weighted_least_squares_smoothing(
                curve,
                &weights,
                lambda,
            )?;
        }
        
        Ok(baseline)
    }
    
    /// 加权最小二乘法平滑
    fn weighted_least_squares_smoothing(
        &self,
        curve: &Curve,
        weights: &[f64],
        lambda: f64,
    ) -> Result<Vec<f64>, BaselineError> {
        let n = curve.point_count;
        
        // 构建差分矩阵 D (二阶差分)
        let mut d_matrix = vec![vec![0.0; n]; n - 2];
        for i in 0..n - 2 {
            d_matrix[i][i] = 1.0;
            d_matrix[i][i + 1] = -2.0;
            d_matrix[i][i + 2] = 1.0;
        }
        
        // 构建权重矩阵 W
        let mut w_matrix = vec![vec![0.0; n]; n];
        for i in 0..n {
            w_matrix[i][i] = weights[i];
        }
        
        // 计算 (W + λD^T D)^(-1) W y
        let result = self.solve_weighted_system(
            &w_matrix,
            &d_matrix,
            &curve.y_values,
            lambda,
        )?;
        
        Ok(result)
    }
    
    /// 求解加权系统
    fn solve_weighted_system(
        &self,
        w_matrix: &[Vec<f64>],
        d_matrix: &[Vec<f64>],
        y_values: &[f64],
        lambda: f64,
    ) -> Result<Vec<f64>, BaselineError> {
        let n = y_values.len();
        let m = d_matrix.len();
        
        // 构建系统矩阵 A = W + λD^T D
        let mut a_matrix = vec![vec![0.0; n]; n];
        
        // 添加 W 部分
        for i in 0..n {
            for j in 0..n {
                a_matrix[i][j] = w_matrix[i][j];
            }
        }
        
        // 添加 λD^T D 部分
        for i in 0..n {
            for j in 0..n {
                for k in 0..m {
                    a_matrix[i][j] += lambda * d_matrix[k][i] * d_matrix[k][j];
                }
            }
        }
        
        // 构建右端向量 b = W y
        let mut b_vector = vec![0.0; n];
        for i in 0..n {
            for j in 0..n {
                b_vector[i] += w_matrix[i][j] * y_values[j];
            }
        }
        
        // 求解线性方程组
        self.solve_linear_system(&a_matrix, &b_vector)
    }
    
    /// 求解线性方程组（使用LU分解）
    fn solve_linear_system(
        &self,
        matrix: &[Vec<f64>],
        rhs: &[f64],
    ) -> Result<Vec<f64>, BaselineError> {
        let n = matrix.len();
        let mut lu = matrix.to_vec();
        let mut p = (0..n).collect::<Vec<usize>>();
        let mut b = rhs.to_vec();
        
        // LU分解
        for k in 0..n - 1 {
            // 寻找主元
            let mut max_row = k;
            for i in k + 1..n {
                if lu[i][k].abs() > lu[max_row][k].abs() {
                    max_row = i;
                }
            }
            
            if lu[max_row][k].abs() < 1e-12 {
                return Err(BaselineError::MathError("Singular matrix".to_string()));
            }
            
            // 交换行
            if max_row != k {
                lu.swap(k, max_row);
                b.swap(k, max_row);
                p.swap(k, max_row);
            }
            
            // 消元
            for i in k + 1..n {
                let factor = lu[i][k] / lu[k][k];
                lu[i][k] = factor;
                for j in k + 1..n {
                    lu[i][j] -= factor * lu[k][j];
                }
            }
        }
        
        // 前向替换
        for i in 1..n {
            for j in 0..i {
                b[i] -= lu[i][j] * b[j];
            }
        }
        
        // 后向替换
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            x[i] = b[i];
            for j in i + 1..n {
                x[i] -= lu[i][j] * x[j];
            }
            x[i] /= lu[i][i];
        }
        
        Ok(x)
    }
    
    /// 自适应参数选择
    fn select_adaptive_parameters(&self, curve: &Curve) -> (f64, f64) {
        // 基于数据特征自动选择参数
        let data_range = curve.y_max - curve.y_min;
        let noise_level = curve.intensity_std;
        
        // lambda: 控制平滑度，噪声越大，lambda越大
        let lambda = if noise_level > 0.0 {
            (data_range / noise_level).powi(2) * 0.1
        } else {
            1.0
        };
        
        // p: 控制非对称性，通常设为0.001-0.1
        let p = if noise_level > 0.0 {
            (noise_level / data_range).min(0.1).max(0.001)
        } else {
            0.01
        };
        
        (lambda, p)
    }
}

impl BaselineAlgorithm for AsymmetricLeastSquaresCorrector {
    fn name(&self) -> &str {
        "Asymmetric Least Squares Baseline Correction"
    }
    
    fn description(&self) -> &str {
        "Uses asymmetric least squares algorithm to estimate baseline, giving different weights to positive and negative residuals"
    }
    
    fn validate_config(&self, config: &BaselineConfig) -> Result<(), BaselineError> {
        match &config.method {
            super::BaselineMethod::AsymmetricLeastSquares { lambda, p, max_iterations } => {
                if *lambda <= 0.0 {
                    return Err(BaselineError::InvalidConfig("Lambda must be positive".to_string()));
                }
                if *p < 0.0 || *p > 1.0 {
                    return Err(BaselineError::InvalidConfig("p must be between 0 and 1".to_string()));
                }
                if *max_iterations == 0 {
                    return Err(BaselineError::InvalidConfig("Max iterations must be positive".to_string()));
                }
                Ok(())
            }
            _ => Err(BaselineError::InvalidConfig(
                "Asymmetric least squares corrector only supports AsymmetricLeastSquares method".to_string()
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
        
        // 获取参数
        let (lambda, p, max_iterations) = match &config.method {
            super::BaselineMethod::AsymmetricLeastSquares { lambda, p, max_iterations } => {
                (*lambda, *p, *max_iterations)
            }
            _ => return Err(BaselineError::InvalidConfig("Invalid method".to_string())),
        };
        
        // 如果参数为默认值，使用自适应选择
        let (final_lambda, final_p) = if lambda == 0.0 || p == 0.0 {
            self.select_adaptive_parameters(curve)
        } else {
            (lambda, p)
        };
        
        // 计算基线
        let baseline_values = self.asymmetric_least_squares(
            curve,
            final_lambda,
            final_p,
            max_iterations,
        )?;
        
        // 计算校准后的数据
        let corrected_y_values: Vec<f64> = curve.y_values.iter()
            .zip(baseline_values.iter())
            .map(|(original, baseline)| (original - baseline).max(0.0))
            .collect();
        
        // 创建校准后的曲线
        let mut corrected_curve = curve.clone();
        corrected_curve.y_values = corrected_y_values.clone();
        corrected_curve.baseline_correction = Some(format!(
            "Asymmetric Least Squares (λ={:.3}, p={:.3})",
            final_lambda, final_p
        ));
        
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
            method_used: format!(
                "Asymmetric Least Squares (λ={:.3}, p={:.3})",
                final_lambda, final_p
            ),
            processing_time_ms: processing_time,
        };
        
        Ok(BaselineResult {
            corrected_curve,
            baseline_curve,
            statistics,
        })
    }
}

impl Default for AsymmetricLeastSquaresCorrector {
    fn default() -> Self {
        Self::new()
    }
}
