//! NLC (Non-Linear Curve) 拟合器
//! 
//! 实现非线性曲线峰的拟合算法，适用于复杂峰形的分析

use crate::core::data::{Curve, Peak, ProcessingError, PeakType};
use crate::core::processors::peak_fitting::PeakFitter;
use serde_json::Value;

/// NLC拟合器
#[derive(Debug)]
pub struct NLCFitter {
    /// 最大迭代次数
    max_iterations: usize,
    /// 收敛阈值
    convergence_threshold: f64,
    /// 正则化参数
    regularization: f64,
    /// 非线性参数数量
    nonlinear_params_count: usize,
}

impl PeakFitter for NLCFitter {
    fn name(&self) -> &str {
        "nlc_fitter"
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
        
        if x_data.len() < 4 {
            return Err(ProcessingError::process_error(
                "NLC拟合需要至少4个数据点"
            ));
        }

        // 执行NLC拟合
        let fit_result = self.fit_nlc(&x_data, &y_data, peak)?;
        
        // 创建拟合后的峰
        let mut fitted_peak = peak.clone();
        fitted_peak.peak_type = PeakType::NLC;
        fitted_peak.amplitude = fit_result.amplitude;
        fitted_peak.center = fit_result.center;
        fitted_peak.sigma = fit_result.sigma;
        
        // 计算NLC的FWHM
        fitted_peak.fwhm = self.calculate_nlc_fwhm(&fit_result);
        fitted_peak.hwhm = fitted_peak.fwhm / 2.0;
        
        // 设置拟合参数
        let mut parameters = vec![
            fit_result.amplitude,
            fit_result.center,
            fit_result.sigma,
        ];
        parameters.extend(&fit_result.nonlinear_params);
        let parameter_errors = vec![0.0; parameters.len()]; // 简化，实际应计算参数误差
        fitted_peak.set_fit_parameters(parameters, parameter_errors, None);
        
        // 计算峰面积
        fitted_peak.calculate_area_from_fit();
        
        // 添加NLC特定元数据
        fitted_peak.add_metadata("nlc_fitted".to_string(), serde_json::json!(true));
        fitted_peak.add_metadata("nonlinear_params_count".to_string(), serde_json::json!(fit_result.nonlinear_params.len()));
        fitted_peak.add_metadata("nonlinear_params".to_string(), serde_json::json!(fit_result.nonlinear_params));
        fitted_peak.add_metadata("curve_complexity".to_string(), serde_json::json!(self.calculate_curve_complexity(&fit_result)));
        fitted_peak.add_metadata("fit_quality".to_string(), serde_json::json!(self.calculate_fit_quality(&fit_result)));
        
        Ok(fitted_peak)
    }
}

impl NLCFitter {
    /// 创建新的NLC拟合器
    pub fn new() -> Self {
        Self {
            max_iterations: 100,
            convergence_threshold: 1e-6,
            regularization: 0.01,
            nonlinear_params_count: 3, // 默认3个非线性参数
        }
    }
    
    /// 设置参数
    pub fn with_parameters(
        mut self,
        max_iterations: usize,
        convergence_threshold: f64,
        regularization: f64,
        nonlinear_params_count: usize,
    ) -> Self {
        self.max_iterations = max_iterations;
        self.convergence_threshold = convergence_threshold;
        self.regularization = regularization;
        self.nonlinear_params_count = nonlinear_params_count;
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
    
    /// 执行NLC拟合
    fn fit_nlc(
        &self,
        x_data: &[f64],
        y_data: &[f64],
        initial_peak: &Peak,
    ) -> Result<NLCParams, ProcessingError> {
        // 初始化参数
        let initial_amplitude = initial_peak.amplitude;
        let initial_center = initial_peak.center;
        let initial_sigma = initial_peak.sigma.max(0.1);
        
        // NLC参数初始化
        let mut params = NLCParams {
            amplitude: initial_amplitude,
            center: initial_center,
            sigma: initial_sigma,
            nonlinear_params: vec![0.1; self.nonlinear_params_count], // 初始非线性参数
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
        params: &NLCParams,
    ) -> Result<(Vec<f64>, Vec<Vec<f64>>), ProcessingError> {
        let n_points = x_data.len();
        let n_params = 3 + params.nonlinear_params.len(); // 基础参数 + 非线性参数
        
        let mut residuals = vec![0.0; n_points];
        let mut jacobian = vec![vec![0.0; n_params]; n_points];
        
        for (i, &x) in x_data.iter().enumerate() {
            let (nlc_value, gradients) = self.nlc_function_with_gradients(x, params);
            residuals[i] = y_data[i] - nlc_value;
            
            // 填充雅可比矩阵
            jacobian[i][0] = gradients.amplitude;
            jacobian[i][1] = gradients.center;
            jacobian[i][2] = gradients.sigma;
            
            // 非线性参数的梯度
            for (j, &grad) in gradients.nonlinear_gradients.iter().enumerate() {
                jacobian[i][3 + j] = grad;
            }
        }
        
        Ok((residuals, jacobian))
    }
    
    /// NLC函数及其梯度
    fn nlc_function_with_gradients(&self, x: f64, params: &NLCParams) -> (f64, NLCGradients) {
        let z = (x - params.center) / params.sigma;
        let z_squared = z * z;
        
        // 基础高斯函数
        let gaussian_base = (-z_squared / 2.0).exp();
        
        // 非线性修正项
        let mut nonlinear_correction = 1.0;
        let mut nonlinear_gradients = vec![0.0; params.nonlinear_params.len()];
        
        for (i, &param) in params.nonlinear_params.iter().enumerate() {
            // 使用多项式修正
            let correction_term = 1.0 + param * z.powi(i as i32 + 1);
            nonlinear_correction *= correction_term;
            
            // 计算非线性参数的梯度
            nonlinear_gradients[i] = params.amplitude * gaussian_base * z.powi(i as i32 + 1);
        }
        
        // NLC函数值
        let nlc_value = params.amplitude * gaussian_base * nonlinear_correction;
        
        // 计算梯度
        let gradients = NLCGradients {
            amplitude: gaussian_base * nonlinear_correction,
            center: nlc_value * z / params.sigma,
            sigma: nlc_value * z_squared / params.sigma,
            nonlinear_gradients,
        };
        
        (nlc_value, gradients)
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
    fn update_parameters(&self, old_params: &NLCParams, update: &[f64]) -> NLCParams {
        let mut new_nonlinear_params = Vec::new();
        for (i, &old_param) in old_params.nonlinear_params.iter().enumerate() {
            new_nonlinear_params.push(old_param + update[3 + i]);
        }
        
        NLCParams {
            amplitude: (old_params.amplitude + update[0]).max(0.0),
            center: old_params.center + update[1],
            sigma: (old_params.sigma + update[2]).max(0.01),
            nonlinear_params: new_nonlinear_params,
        }
    }
    
    /// 检查收敛
    fn check_convergence(&self, old_params: &NLCParams, new_params: &NLCParams) -> bool {
        let amplitude_diff = (old_params.amplitude - new_params.amplitude).abs() / old_params.amplitude.max(1e-6);
        let center_diff = (old_params.center - new_params.center).abs();
        let sigma_diff = (old_params.sigma - new_params.sigma).abs() / old_params.sigma.max(1e-6);
        
        if amplitude_diff > self.convergence_threshold ||
           center_diff > self.convergence_threshold ||
           sigma_diff > self.convergence_threshold {
            return false;
        }
        
        // 检查非线性参数的收敛
        for (old_param, new_param) in old_params.nonlinear_params.iter().zip(new_params.nonlinear_params.iter()) {
            let param_diff = (old_param - new_param).abs() / old_param.abs().max(1e-6);
            if param_diff > self.convergence_threshold {
                return false;
            }
        }
        
        true
    }
    
    /// 计算NLC的FWHM
    fn calculate_nlc_fwhm(&self, params: &NLCParams) -> f64 {
        // 基础FWHM
        let base_fwhm = 2.355 * params.sigma;
        
        // 非线性修正
        let nonlinear_factor = 1.0 + params.nonlinear_params.iter().map(|&p| p.abs()).sum::<f64>() * 0.1;
        
        base_fwhm * nonlinear_factor
    }
    
    /// 计算曲线复杂度
    fn calculate_curve_complexity(&self, params: &NLCParams) -> f64 {
        // 基于非线性参数的数量和大小计算复杂度
        let param_magnitude = params.nonlinear_params.iter().map(|&p| p.abs()).sum::<f64>();
        let complexity = params.nonlinear_params.len() as f64 * 0.1 + param_magnitude * 0.5;
        complexity.min(1.0) // 限制在0-1范围内
    }
    
    /// 计算拟合质量
    fn calculate_fit_quality(&self, params: &NLCParams) -> f64 {
        // 基于参数稳定性计算拟合质量
        let param_stability = 1.0 / (1.0 + params.nonlinear_params.iter().map(|&p| p.abs()).sum::<f64>());
        param_stability.min(1.0)
    }
}

/// NLC参数
#[derive(Debug, Clone)]
struct NLCParams {
    amplitude: f64,
    center: f64,
    sigma: f64,
    nonlinear_params: Vec<f64>,
}

/// NLC梯度
#[derive(Debug)]
struct NLCGradients {
    amplitude: f64,
    center: f64,
    sigma: f64,
    nonlinear_gradients: Vec<f64>,
}
