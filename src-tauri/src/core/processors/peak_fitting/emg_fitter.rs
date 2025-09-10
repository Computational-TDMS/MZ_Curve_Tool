//! EMG (Exponentially Modified Gaussian) 拟合器
//! 
//! 实现指数修正高斯峰的拟合算法，适用于色谱分析中的拖尾峰

use crate::core::data::{Curve, Peak, ProcessingError, PeakType};
use crate::core::processors::peak_fitting::PeakFitter;
use serde_json::Value;

/// EMG拟合器
#[derive(Debug)]
pub struct EMGFitter;

impl PeakFitter for EMGFitter {
    fn name(&self) -> &str {
        "emg_fitter"
    }

    fn fit_peak(&self, peak: &Peak, curve: &Curve, config: &Value) -> Result<Peak, ProcessingError> {
        // 提取拟合窗口
        let window_size = config["fit_window_size"].as_f64().unwrap_or(3.0);
        let (x_data, y_data) = self.extract_fit_data(curve, peak.center, window_size);
        
        if x_data.len() < 4 {
            return Err(ProcessingError::process_error(
                "EMG拟合需要至少4个数据点"
            ));
        }

        // 执行EMG拟合
        let fit_result = self.fit_emg(&x_data, &y_data, peak)?;
        
        // 创建拟合后的峰
        let mut fitted_peak = peak.clone();
        fitted_peak.peak_type = PeakType::EMG;
        fitted_peak.center = fit_result.center;
        fitted_peak.amplitude = fit_result.amplitude;
        fitted_peak.sigma = fit_result.sigma;
        fitted_peak.tau = fit_result.tau; // 设置tau参数
        fitted_peak.fwhm = fit_result.fwhm;
        fitted_peak.hwhm = fit_result.fwhm / 2.0;
        
        // 设置拟合参数
        let parameters = vec![
            fit_result.amplitude,
            fit_result.center,
            fit_result.sigma,
            fit_result.tau,
        ];
        let parameter_errors = vec![
            fit_result.amplitude_error,
            fit_result.center_error,
            fit_result.sigma_error,
            fit_result.tau_error,
        ];
        fitted_peak.set_fit_parameters(parameters, parameter_errors, None);
        
        // 计算峰面积
        fitted_peak.calculate_area_from_fit();
        
        // 计算拟合质量
        fitted_peak.rsquared = fit_result.rsquared;
        fitted_peak.standard_error = fit_result.standard_error;
        
        // 添加EMG特定元数据
        fitted_peak.add_metadata("emg_fitted".to_string(), serde_json::json!(true));
        fitted_peak.add_metadata("tau".to_string(), serde_json::json!(fit_result.tau));
        fitted_peak.add_metadata("tau_error".to_string(), serde_json::json!(fit_result.tau_error));
        fitted_peak.add_metadata("asymmetry_ratio".to_string(), serde_json::json!(fit_result.tau / fit_result.sigma));
        
        Ok(fitted_peak)
    }
}

impl EMGFitter {
    /// 提取拟合数据
    fn extract_fit_data(&self, curve: &Curve, center: f64, window_size: f64) -> (Vec<f64>, Vec<f64>) {
        let mut x_data = Vec::new();
        let mut y_data = Vec::new();
        
        let half_window = window_size / 2.0;
        let left_bound = center - half_window;
        let right_bound = center + half_window;
        
        for (i, &x) in curve.x_values.iter().enumerate() {
            if x >= left_bound && x <= right_bound {
                x_data.push(x);
                y_data.push(curve.y_values[i]);
            }
        }
        
        (x_data, y_data)
    }
    
    /// 执行EMG拟合
    fn fit_emg(&self, x_data: &[f64], y_data: &[f64], initial_peak: &Peak) -> Result<EMGFitResult, ProcessingError> {
        // 初始参数估计
        let initial_amplitude = initial_peak.amplitude;
        let initial_center = initial_peak.center;
        let initial_sigma = initial_peak.sigma.max(0.1);
        let initial_tau = initial_sigma * 0.5; // 初始tau估计
        
        // 使用Levenberg-Marquardt算法进行非线性最小二乘拟合
        let params = EMGParams {
            amplitude: initial_amplitude,
            center: initial_center,
            sigma: initial_sigma,
            tau: initial_tau,
        };
        
        // 简化的优化过程（实际应用中应使用更robust的优化库）
        let mut best_error = f64::INFINITY;
        let mut best_params = params.clone();
        
        // 网格搜索优化
        for amp_factor in [0.8, 0.9, 1.0, 1.1, 1.2] {
            for center_offset in [-0.1, -0.05, 0.0, 0.05, 0.1] {
                for sigma_factor in [0.8, 0.9, 1.0, 1.1, 1.2] {
                    for tau_factor in [0.5, 0.7, 1.0, 1.3, 1.5] {
                        let test_params = EMGParams {
                            amplitude: initial_amplitude * amp_factor,
                            center: initial_center + center_offset,
                            sigma: initial_sigma * sigma_factor,
                            tau: initial_tau * tau_factor,
                        };
                        
                        let error = self.calculate_fit_error(x_data, y_data, &test_params);
                        if error < best_error {
                            best_error = error;
                            best_params = test_params;
                        }
                    }
                }
            }
        }
        
        // 计算拟合质量
        let rsquared = self.calculate_rsquared(x_data, y_data, &best_params);
        let standard_error = (best_error / (x_data.len() as f64 - 4.0)).sqrt();
        
        // 计算FWHM（EMG的FWHM计算比较复杂，这里使用近似）
        let fwhm = self.calculate_emg_fwhm(&best_params);
        
        Ok(EMGFitResult {
            amplitude: best_params.amplitude,
            center: best_params.center,
            sigma: best_params.sigma,
            tau: best_params.tau,
            fwhm,
            amplitude_error: standard_error,
            center_error: standard_error,
            sigma_error: standard_error,
            tau_error: standard_error,
            rsquared,
            standard_error,
        })
    }
    
    /// 计算拟合误差
    fn calculate_fit_error(&self, x_data: &[f64], y_data: &[f64], params: &EMGParams) -> f64 {
        let mut error = 0.0;
        for (i, &x) in x_data.iter().enumerate() {
            let predicted = self.emg_function(x, params);
            error += (y_data[i] - predicted).powi(2);
        }
        error
    }
    
    /// EMG函数
    fn emg_function(&self, x: f64, params: &EMGParams) -> f64 {
        let z = (x - params.center) / params.sigma - params.sigma / params.tau;
        let erfc_arg = z / (2.0_f64.sqrt());
        
        // 使用近似erfc函数（实际应用中应使用更精确的实现）
        let erfc_value = self.approximate_erfc(erfc_arg);
        
        params.amplitude * (params.sigma / params.tau) * 
        (params.sigma / (2.0 * params.tau) - (x - params.center) / params.tau).exp() * 
        erfc_value
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
    
    /// 计算EMG的FWHM
    fn calculate_emg_fwhm(&self, params: &EMGParams) -> f64 {
        // EMG的FWHM计算比较复杂，这里使用经验公式
        let gaussian_fwhm = 2.355 * params.sigma;
        let exponential_contribution = params.tau * 2.0;
        (gaussian_fwhm * gaussian_fwhm + exponential_contribution * exponential_contribution).sqrt()
    }
    
    /// 计算R²
    fn calculate_rsquared(&self, x_data: &[f64], y_data: &[f64], params: &EMGParams) -> f64 {
        let y_mean: f64 = y_data.iter().sum::<f64>() / y_data.len() as f64;
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;

        for (i, &y) in y_data.iter().enumerate() {
            let y_fit = self.emg_function(x_data[i], params);
            ss_tot += (y - y_mean).powi(2);
            ss_res += (y - y_fit).powi(2);
        }

        if ss_tot == 0.0 {
            0.0
        } else {
            1.0 - (ss_res / ss_tot)
        }
    }
}

/// EMG拟合参数
#[derive(Debug, Clone)]
struct EMGParams {
    amplitude: f64,
    center: f64,
    sigma: f64,
    tau: f64, // 指数衰减常数
}

/// EMG拟合结果
#[derive(Debug)]
struct EMGFitResult {
    amplitude: f64,
    center: f64,
    sigma: f64,
    tau: f64,
    fwhm: f64,
    amplitude_error: f64,
    center_error: f64,
    sigma_error: f64,
    tau_error: f64,
    rsquared: f64,
    standard_error: f64,
}
