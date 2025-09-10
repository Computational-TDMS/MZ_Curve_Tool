//! Pearson-IV拟合器
//! 
//! 实现Pearson-IV分布的峰拟合算法，适用于非对称峰的分析

use crate::core::data::{Curve, Peak, ProcessingError, PeakType};
use crate::core::processors::peak_fitting::PeakFitter;
use serde_json::Value;

/// Pearson-IV拟合器
#[derive(Debug)]
pub struct PearsonIVFitter {
    /// 最大迭代次数
    max_iterations: usize,
    /// 收敛阈值
    convergence_threshold: f64,
    /// 正则化参数
    regularization: f64,
}

impl PeakFitter for PearsonIVFitter {
    fn name(&self) -> &str {
        "pearson_iv_fitter"
    }

    fn fit_peak(
        &self,
        peak: &Peak,
        curve: &Curve,
        config: &Value,
    ) -> Result<Peak, ProcessingError> {
        // 提取拟合窗口
        let window_size = config["fit_window_size"].as_f64().unwrap_or(3.0);
        let (x_data, y_data) = self.extract_fit_data(curve, peak.center, window_size);
        
        if x_data.len() < 5 {
            return Err(ProcessingError::process_error(
                "Pearson-IV拟合需要至少5个数据点"
            ));
        }

        // 执行Pearson-IV拟合
        let fit_result = self.fit_pearson_iv(&x_data, &y_data, peak)?;
        
        // 创建拟合后的峰
        let mut fitted_peak = peak.clone();
        fitted_peak.peak_type = PeakType::PearsonIV;
        fitted_peak.amplitude = fit_result.amplitude;
        fitted_peak.center = fit_result.center;
        fitted_peak.sigma = fit_result.sigma;
        
        // 计算Pearson-IV的FWHM
        fitted_peak.fwhm = self.calculate_pearson_iv_fwhm(&fit_result);
        fitted_peak.hwhm = fitted_peak.fwhm / 2.0;
        
        // 设置拟合参数
        let parameters = vec![
            fit_result.amplitude,
            fit_result.center,
            fit_result.sigma,
            fit_result.a,
            fit_result.b,
            fit_result.c,
        ];
        let parameter_errors = vec![0.0; 6]; // 简化，实际应计算参数误差
        fitted_peak.set_fit_parameters(parameters, parameter_errors, None);
        
        // 计算峰面积
        fitted_peak.calculate_area_from_fit();
        
        // 添加Pearson-IV特定元数据
        fitted_peak.add_metadata("pearson_iv_fitted".to_string(), serde_json::json!(true));
        fitted_peak.add_metadata("pearson_a".to_string(), serde_json::json!(fit_result.a));
        fitted_peak.add_metadata("pearson_b".to_string(), serde_json::json!(fit_result.b));
        fitted_peak.add_metadata("pearson_c".to_string(), serde_json::json!(fit_result.c));
        fitted_peak.add_metadata("skewness".to_string(), serde_json::json!(self.calculate_skewness(&fit_result)));
        fitted_peak.add_metadata("kurtosis".to_string(), serde_json::json!(self.calculate_kurtosis(&fit_result)));
        
        Ok(fitted_peak)
    }
}

impl PearsonIVFitter {
    /// 创建新的Pearson-IV拟合器
    pub fn new() -> Self {
        Self {
            max_iterations: 100,
            convergence_threshold: 1e-6,
            regularization: 0.01,
        }
    }
    
    /// 设置参数
    pub fn with_parameters(
        mut self,
        max_iterations: usize,
        convergence_threshold: f64,
        regularization: f64,
    ) -> Self {
        self.max_iterations = max_iterations;
        self.convergence_threshold = convergence_threshold;
        self.regularization = regularization;
        self
    }
    
    /// 提取拟合数据
    fn extract_fit_data(&self, curve: &Curve, center: f64, window_size: f64) -> (Vec<f64>, Vec<f64>) {
        let mut x_data = Vec::new();
        let mut y_data = Vec::new();
        
        let left_bound = center - window_size;
        let right_bound = center + window_size;
        
        for (i, &x) in curve.x_values.iter().enumerate() {
            if x >= left_bound && x <= right_bound {
                x_data.push(x);
                y_data.push(curve.y_values[i]);
            }
        }
        
        (x_data, y_data)
    }
    
    /// 执行Pearson-IV拟合
    fn fit_pearson_iv(
        &self,
        x_data: &[f64],
        y_data: &[f64],
        initial_peak: &Peak,
    ) -> Result<PearsonIVParams, ProcessingError> {
        // 初始化参数
        let initial_amplitude = initial_peak.amplitude;
        let initial_center = initial_peak.center;
        let initial_sigma = initial_peak.sigma.max(0.1);
        
        // Pearson-IV参数初始化
        let mut params = PearsonIVParams {
            amplitude: initial_amplitude,
            center: initial_center,
            sigma: initial_sigma,
            a: 0.0,  // 形状参数
            b: 1.0,  // 形状参数
            c: 0.0,  // 形状参数
        };
        
        // 使用Levenberg-Marquardt算法进行非线性最小二乘拟合
        for _iteration in 0..self.max_iterations {
            // 计算残差和雅可比矩阵
            let (residuals, jacobian) = self.compute_residuals_and_jacobian(x_data, y_data, &params)?;
            
            // 计算参数更新
            let parameter_update = self.compute_parameter_update(&residuals, &jacobian)?;
            
            // 更新参数
            let new_params = self.update_parameters(&params, &parameter_update);
            
            // 检查收敛
            if self.check_convergence(&params, &new_params) {
                return Ok(new_params);
            }
            
            params = new_params;
        }
        
        Ok(params)
    }
    
    /// 计算残差和雅可比矩阵
    fn compute_residuals_and_jacobian(
        &self,
        x_data: &[f64],
        y_data: &[f64],
        params: &PearsonIVParams,
    ) -> Result<(Vec<f64>, Vec<Vec<f64>>), ProcessingError> {
        let n_points = x_data.len();
        let n_params = 6; // Pearson-IV有6个参数
        
        let mut residuals = vec![0.0; n_points];
        let mut jacobian = vec![vec![0.0; n_params]; n_points];
        
        for (i, &x) in x_data.iter().enumerate() {
            let (pearson_value, gradients) = self.pearson_iv_function_with_gradients(x, params);
            residuals[i] = y_data[i] - pearson_value;
            
            // 填充雅可比矩阵
            jacobian[i][0] = gradients.amplitude;
            jacobian[i][1] = gradients.center;
            jacobian[i][2] = gradients.sigma;
            jacobian[i][3] = gradients.a;
            jacobian[i][4] = gradients.b;
            jacobian[i][5] = gradients.c;
        }
        
        Ok((residuals, jacobian))
    }
    
    /// Pearson-IV函数及其梯度
    fn pearson_iv_function_with_gradients(&self, x: f64, params: &PearsonIVParams) -> (f64, PearsonIVGradients) {
        let z = (x - params.center) / params.sigma;
        let z_squared = z * z;
        
        // Pearson-IV函数值（简化版本）
        let denominator = 1.0 + params.a * z + params.b * z_squared + params.c * z_squared * z;
        let pearson_value = params.amplitude / denominator.powf(params.b / 2.0);
        
        // 计算梯度（简化版本）
        let gradients = PearsonIVGradients {
            amplitude: pearson_value / params.amplitude,
            center: pearson_value * params.b * z / (params.sigma * denominator),
            sigma: pearson_value * params.b * z_squared / (params.sigma * denominator),
            a: -pearson_value * params.b * z / (2.0 * denominator),
            b: -pearson_value * z_squared / (2.0 * denominator),
            c: -pearson_value * z_squared * z / (2.0 * denominator),
        };
        
        (pearson_value, gradients)
    }
    
    /// 计算参数更新
    fn compute_parameter_update(
        &self,
        residuals: &[f64],
        jacobian: &[Vec<f64>],
    ) -> Result<Vec<f64>, ProcessingError> {
        let n_points = residuals.len();
        let n_params = jacobian[0].len();
        
        // 计算正规方程: (J^T * J + λI) * Δp = J^T * r
        let mut jtj = vec![vec![0.0; n_params]; n_params];
        let mut jtr = vec![0.0; n_params];
        
        // 计算J^T * J
        for i in 0..n_params {
            for j in 0..n_params {
                for k in 0..n_points {
                    jtj[i][j] += jacobian[k][i] * jacobian[k][j];
                }
                // 添加正则化项
                if i == j {
                    jtj[i][j] += self.regularization;
                }
            }
        }
        
        // 计算J^T * r
        for i in 0..n_params {
            for k in 0..n_points {
                jtr[i] += jacobian[k][i] * residuals[k];
            }
        }
        
        // 求解线性方程组
        self.solve_linear_system(&jtj, &jtr)
    }
    
    /// 求解线性方程组
    fn solve_linear_system(&self, matrix: &[Vec<f64>], rhs: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        let n = matrix.len();
        let mut a = matrix.to_vec();
        let mut b = rhs.to_vec();
        
        // 高斯消元法
        for i in 0..n {
            // 寻找主元
            let mut max_row = i;
            for k in (i + 1)..n {
                if a[k][i].abs() > a[max_row][i].abs() {
                    max_row = k;
                }
            }
            
            // 交换行
            if max_row != i {
                a.swap(i, max_row);
                b.swap(i, max_row);
            }
            
            // 检查奇异矩阵
            if a[i][i].abs() < 1e-12 {
                return Err(ProcessingError::process_error(
                    "雅可比矩阵奇异，无法求解"
                ));
            }
            
            // 消元
            for k in (i + 1)..n {
                let factor = a[k][i] / a[i][i];
                for j in i..n {
                    a[k][j] -= factor * a[i][j];
                }
                b[k] -= factor * b[i];
            }
        }
        
        // 回代
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            x[i] = b[i];
            for j in (i + 1)..n {
                x[i] -= a[i][j] * x[j];
            }
            x[i] /= a[i][i];
        }
        
        Ok(x)
    }
    
    /// 更新参数
    fn update_parameters(&self, old_params: &PearsonIVParams, update: &[f64]) -> PearsonIVParams {
        PearsonIVParams {
            amplitude: (old_params.amplitude + update[0]).max(0.0),
            center: old_params.center + update[1],
            sigma: (old_params.sigma + update[2]).max(0.01),
            a: old_params.a + update[3],
            b: (old_params.b + update[4]).max(0.01),
            c: old_params.c + update[5],
        }
    }
    
    /// 检查收敛
    fn check_convergence(&self, old_params: &PearsonIVParams, new_params: &PearsonIVParams) -> bool {
        let amplitude_diff = (old_params.amplitude - new_params.amplitude).abs() / old_params.amplitude.max(1e-6);
        let center_diff = (old_params.center - new_params.center).abs();
        let sigma_diff = (old_params.sigma - new_params.sigma).abs() / old_params.sigma.max(1e-6);
        let a_diff = (old_params.a - new_params.a).abs();
        let b_diff = (old_params.b - new_params.b).abs() / old_params.b.max(1e-6);
        let c_diff = (old_params.c - new_params.c).abs();
        
        amplitude_diff < self.convergence_threshold &&
        center_diff < self.convergence_threshold &&
        sigma_diff < self.convergence_threshold &&
        a_diff < self.convergence_threshold &&
        b_diff < self.convergence_threshold &&
        c_diff < self.convergence_threshold
    }
    
    /// 计算Pearson-IV的FWHM
    fn calculate_pearson_iv_fwhm(&self, params: &PearsonIVParams) -> f64 {
        // 简化的FWHM计算
        let base_fwhm = 2.355 * params.sigma;
        let asymmetry_factor = 1.0 + params.a.abs() * 0.1;
        base_fwhm * asymmetry_factor
    }
    
    /// 计算偏度
    fn calculate_skewness(&self, params: &PearsonIVParams) -> f64 {
        // 基于Pearson-IV参数的偏度估计
        params.a * 0.5
    }
    
    /// 计算峰度
    fn calculate_kurtosis(&self, params: &PearsonIVParams) -> f64 {
        // 基于Pearson-IV参数的峰度估计
        3.0 + params.b * 0.3
    }
}

/// Pearson-IV参数
#[derive(Debug, Clone)]
struct PearsonIVParams {
    amplitude: f64,
    center: f64,
    sigma: f64,
    a: f64,  // 形状参数
    b: f64,  // 形状参数
    c: f64,  // 形状参数
}

/// Pearson-IV梯度
#[derive(Debug)]
struct PearsonIVGradients {
    amplitude: f64,
    center: f64,
    sigma: f64,
    a: f64,
    b: f64,
    c: f64,
}
