//! EMG-NLLS (Non-Linear Least Squares) 重叠峰拟合器
//! 
//! 实现基于EMG的非线性最小二乘重叠峰拟合算法

use crate::core::data::{Curve, Peak, ProcessingError, PeakType};
use crate::core::processors::overlapping_peaks::OverlappingPeakProcessor;
use serde_json::Value;

/// EMG-NLLS拟合器
#[derive(Debug)]
pub struct EMGNLLSFitter {
    /// 最大迭代次数
    max_iterations: usize,
    /// 收敛阈值
    convergence_threshold: f64,
    /// 正则化参数
    regularization: f64,
}

impl OverlappingPeakProcessor for EMGNLLSFitter {
    fn name(&self) -> &str {
        "emg_nlls_fitter"
    }

    fn process_overlapping_peaks(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        _config: &Value,
    ) -> Result<Vec<Peak>, ProcessingError> {
        if peaks.len() < 2 {
            return Ok(peaks.to_vec());
        }
        
        // 提取重叠区域数据
        let (x_data, y_data) = self.extract_overlapping_region(peaks, curve);
        
        if x_data.len() < peaks.len() * 4 {
            return Err(ProcessingError::process_error(
                "重叠区域数据点不足"
            ));
        }
        
        // 初始化EMG参数
        let mut emg_params = self.initialize_emg_parameters(peaks);
        
        // 执行NLLS优化
        for _iteration in 0..self.max_iterations {
            // 计算残差和雅可比矩阵
            let (residuals, jacobian) = self.compute_residuals_and_jacobian(&x_data, &y_data, &emg_params)?;
            
            // 计算参数更新
            let parameter_update = self.compute_parameter_update(&residuals, &jacobian)?;
            
            // 更新参数
            let new_params = self.update_parameters(&emg_params, &parameter_update);
            
            // 检查收敛
            if self.check_convergence(&emg_params, &new_params) {
                emg_params = new_params;
                break;
            }
            
            emg_params = new_params;
        }
        
        // 生成拟合后的峰
        self.generate_fitted_peaks(&emg_params, peaks)
    }
}

impl EMGNLLSFitter {
    /// 创建新的EMG-NLLS拟合器
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
    
    /// 提取重叠区域数据
    fn extract_overlapping_region(&self, peaks: &[Peak], curve: &Curve) -> (Vec<f64>, Vec<f64>) {
        let mut x_data = Vec::new();
        let mut y_data = Vec::new();
        
        // 计算重叠区域范围
        let min_center = peaks.iter().map(|p| p.center).fold(f64::INFINITY, f64::min);
        let max_center = peaks.iter().map(|p| p.center).fold(f64::NEG_INFINITY, f64::max);
        let max_width = peaks.iter().map(|p| p.fwhm.max(p.peak_span)).fold(0.0, f64::max);
        
        let left_bound = min_center - max_width * 3.0;
        let right_bound = max_center + max_width * 3.0;
        
        for (i, &x) in curve.x_values.iter().enumerate() {
            if x >= left_bound && x <= right_bound {
                x_data.push(x);
                y_data.push(curve.y_values[i]);
            }
        }
        
        (x_data, y_data)
    }
    
    /// 初始化EMG参数
    fn initialize_emg_parameters(&self, peaks: &[Peak]) -> Vec<EMGParams> {
        let mut emg_params = Vec::new();
        
        for peak in peaks {
            emg_params.push(EMGParams {
                amplitude: peak.amplitude,
                center: peak.center,
                sigma: peak.sigma.max(0.1),
                tau: peak.sigma * 0.5, // 初始tau估计
            });
        }
        
        emg_params
    }
    
    /// 计算残差和雅可比矩阵
    fn compute_residuals_and_jacobian(
        &self,
        x_data: &[f64],
        y_data: &[f64],
        emg_params: &[EMGParams],
    ) -> Result<(Vec<f64>, Vec<Vec<f64>>), ProcessingError> {
        let n_points = x_data.len();
        let n_peaks = emg_params.len();
        let n_params = n_peaks * 4; // 每个EMG峰4个参数
        
        let mut residuals = vec![0.0; n_points];
        let mut jacobian = vec![vec![0.0; n_params]; n_points];
        
        for (i, &x) in x_data.iter().enumerate() {
            let mut predicted = 0.0;
            
            // 计算预测值和雅可比矩阵
            for (peak_idx, emg_param) in emg_params.iter().enumerate() {
                let (emg_value, emg_gradients) = self.emg_function_with_gradients(x, emg_param);
                predicted += emg_value;
                
                // 填充雅可比矩阵
                let param_start = peak_idx * 4;
                jacobian[i][param_start] = emg_gradients.amplitude;     // d/dA
                jacobian[i][param_start + 1] = emg_gradients.center;    // d/dμ
                jacobian[i][param_start + 2] = emg_gradients.sigma;     // d/dσ
                jacobian[i][param_start + 3] = emg_gradients.tau;       // d/dτ
            }
            
            residuals[i] = y_data[i] - predicted;
        }
        
        Ok((residuals, jacobian))
    }
    
    /// EMG函数及其梯度
    fn emg_function_with_gradients(&self, x: f64, params: &EMGParams) -> (f64, EMGGradients) {
        let z = (x - params.center) / params.sigma - params.sigma / params.tau;
        let erfc_arg = z / (2.0_f64.sqrt());
        
        // 使用近似erfc函数
        let erfc_value = self.approximate_erfc(erfc_arg);
        
        // EMG函数值
        let emg_value = params.amplitude * (params.sigma / params.tau) * 
                       (params.sigma / (2.0 * params.tau) - (x - params.center) / params.tau).exp() * 
                       erfc_value;
        
        // 计算梯度（简化版本）
        let gradients = EMGGradients {
            amplitude: emg_value / params.amplitude,
            center: emg_value * (1.0 / params.sigma + 1.0 / params.tau),
            sigma: emg_value * (1.0 / params.sigma - params.sigma / (params.tau * params.tau)),
            tau: emg_value * (params.sigma / (params.tau * params.tau) + (x - params.center) / (params.tau * params.tau)),
        };
        
        (emg_value, gradients)
    }
    
    /// 近似erfc函数
    fn approximate_erfc(&self, x: f64) -> f64 {
        // 使用Abramowitz和Stegun的近似公式
        let a1 = 0.254829592;
        let a2 = -0.284496736;
        let a3 = 1.421413741;
        let a4 = -1.453152027;
        let a5 = 1.061405429;
        let p = 0.3275911;
        
        let sign = if x >= 0.0 { 1.0 } else { -1.0 };
        let x = x.abs();
        
        let t = 1.0 / (1.0 + p * x);
        let y = 1.0 - (((((a5 * t + a4) * t) + a3) * t + a2) * t + a1) * t * (-x * x).exp();
        
        sign * y
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
        
        // 求解线性方程组（使用简化的高斯消元法）
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
    fn update_parameters(&self, old_params: &[EMGParams], update: &[f64]) -> Vec<EMGParams> {
        let mut new_params = Vec::new();
        
        for (i, old_param) in old_params.iter().enumerate() {
            let param_start = i * 4;
            new_params.push(EMGParams {
                amplitude: (old_param.amplitude + update[param_start]).max(0.0),
                center: old_param.center + update[param_start + 1],
                sigma: (old_param.sigma + update[param_start + 2]).max(0.01),
                tau: (old_param.tau + update[param_start + 3]).max(0.01),
            });
        }
        
        new_params
    }
    
    /// 检查收敛
    fn check_convergence(&self, old_params: &[EMGParams], new_params: &[EMGParams]) -> bool {
        for (old_param, new_param) in old_params.iter().zip(new_params.iter()) {
            let amplitude_diff = (old_param.amplitude - new_param.amplitude).abs() / old_param.amplitude.max(1e-6);
            let center_diff = (old_param.center - new_param.center).abs();
            let sigma_diff = (old_param.sigma - new_param.sigma).abs() / old_param.sigma.max(1e-6);
            let tau_diff = (old_param.tau - new_param.tau).abs() / old_param.tau.max(1e-6);
            
            if amplitude_diff > self.convergence_threshold ||
               center_diff > self.convergence_threshold ||
               sigma_diff > self.convergence_threshold ||
               tau_diff > self.convergence_threshold {
                return false;
            }
        }
        true
    }
    
    /// 生成拟合后的峰
    fn generate_fitted_peaks(&self, emg_params: &[EMGParams], original_peaks: &[Peak]) -> Result<Vec<Peak>, ProcessingError> {
        let mut fitted_peaks = Vec::new();
        
        for (i, emg_param) in emg_params.iter().enumerate() {
            if i < original_peaks.len() {
                let mut fitted_peak = original_peaks[i].clone();
                fitted_peak.peak_type = PeakType::EMG;
                fitted_peak.amplitude = emg_param.amplitude;
                fitted_peak.center = emg_param.center;
                fitted_peak.sigma = emg_param.sigma;
                fitted_peak.tau = emg_param.tau;
                
                // 计算EMG的FWHM
                let gaussian_fwhm = 2.355 * emg_param.sigma;
                let exponential_contribution = emg_param.tau * 2.0;
                fitted_peak.fwhm = (gaussian_fwhm * gaussian_fwhm + exponential_contribution * exponential_contribution).sqrt();
                fitted_peak.hwhm = fitted_peak.fwhm / 2.0;
                
                // 设置拟合参数
                let parameters = vec![
                    emg_param.amplitude,
                    emg_param.center,
                    emg_param.sigma,
                    emg_param.tau,
                ];
                let parameter_errors = vec![0.0; 4]; // 简化，实际应计算参数误差
                fitted_peak.set_fit_parameters(parameters, parameter_errors, None);
                
                // 计算峰面积
                fitted_peak.calculate_area_from_fit();
                
                // 添加EMG-NLLS特定元数据
                fitted_peak.add_metadata("emg_nlls_fitted".to_string(), serde_json::json!(true));
                fitted_peak.add_metadata("tau".to_string(), serde_json::json!(emg_param.tau));
                fitted_peak.add_metadata("asymmetry_ratio".to_string(), serde_json::json!(emg_param.tau / emg_param.sigma));
                
                fitted_peaks.push(fitted_peak);
            }
        }
        
        Ok(fitted_peaks)
    }
}

/// EMG参数
#[derive(Debug, Clone)]
struct EMGParams {
    amplitude: f64,
    center: f64,
    sigma: f64,
    tau: f64,
}

/// EMG梯度
#[derive(Debug)]
struct EMGGradients {
    amplitude: f64,
    center: f64,
    sigma: f64,
    tau: f64,
}
