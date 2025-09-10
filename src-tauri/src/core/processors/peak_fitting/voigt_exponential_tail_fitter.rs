//! Voigt + 指数尾拟合器
//! 
//! 实现Voigt峰加指数尾的拟合算法，适用于XPS/XRD分析

use crate::core::data::{Curve, Peak, ProcessingError, PeakType};
use crate::core::processors::peak_fitting::PeakFitter;
use serde_json::Value;

/// Voigt + 指数尾拟合器
#[derive(Debug)]
pub struct VoigtExponentialTailFitter;

impl PeakFitter for VoigtExponentialTailFitter {
    fn name(&self) -> &str {
        "voigt_exponential_tail_fitter"
    }

    fn fit_peak(&self, peak: &Peak, curve: &Curve, config: &Value) -> Result<Peak, ProcessingError> {
        // 提取拟合窗口
        let window_size = config["fit_window_size"].as_f64().unwrap_or(3.0);
        let (x_data, y_data) = self.extract_fit_data(curve, peak.center, window_size);
        
        if x_data.len() < 5 {
            return Err(ProcessingError::process_error(
                "Voigt+指数尾拟合需要至少5个数据点"
            ));
        }

        // 执行Voigt+指数尾拟合
        let fit_result = self.fit_voigt_exponential_tail(&x_data, &y_data, peak)?;
        
        // 创建拟合后的峰
        let mut fitted_peak = peak.clone();
        fitted_peak.peak_type = PeakType::VoigtExponentialTail;
        fitted_peak.center = fit_result.center;
        fitted_peak.amplitude = fit_result.amplitude;
        fitted_peak.sigma = fit_result.sigma;
        fitted_peak.gamma = fit_result.gamma;
        fitted_peak.fwhm = fit_result.fwhm;
        fitted_peak.hwhm = fit_result.fwhm / 2.0;
        
        // 设置拟合参数
        let parameters = vec![
            fit_result.amplitude,
            fit_result.center,
            fit_result.sigma,
            fit_result.gamma,
            fit_result.tau,
        ];
        let parameter_errors = vec![
            fit_result.amplitude_error,
            fit_result.center_error,
            fit_result.sigma_error,
            fit_result.gamma_error,
            fit_result.tau_error,
        ];
        fitted_peak.set_fit_parameters(parameters, parameter_errors, None);
        
        // 计算峰面积
        fitted_peak.calculate_area_from_fit();
        
        // 计算拟合质量
        fitted_peak.rsquared = fit_result.rsquared;
        fitted_peak.standard_error = fit_result.standard_error;
        
        // 添加Voigt+指数尾特定元数据
        fitted_peak.add_metadata("gamma".to_string(), serde_json::json!(fit_result.gamma));
        fitted_peak.add_metadata("tau".to_string(), serde_json::json!(fit_result.tau));
        fitted_peak.add_metadata("voigt_mixing".to_string(), serde_json::json!(fit_result.voigt_mixing));
        fitted_peak.add_metadata("tail_contribution".to_string(), serde_json::json!(fit_result.tail_contribution));
        
        Ok(fitted_peak)
    }
}

impl VoigtExponentialTailFitter {
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
    
    /// 执行Voigt+指数尾拟合
    fn fit_voigt_exponential_tail(&self, x_data: &[f64], y_data: &[f64], initial_peak: &Peak) -> Result<VoigtExponentialTailFitResult, ProcessingError> {
        // 初始参数估计
        let initial_amplitude = initial_peak.amplitude;
        let initial_center = initial_peak.center;
        let initial_sigma = initial_peak.sigma.max(0.1);
        let initial_gamma = initial_sigma * 0.5; // 初始gamma估计
        let initial_tau = initial_sigma * 0.3; // 初始tau估计
        
        // 使用网格搜索优化
        let mut best_error = f64::INFINITY;
        let mut best_params = VoigtExponentialTailParams {
            amplitude: initial_amplitude,
            center: initial_center,
            sigma: initial_sigma,
            gamma: initial_gamma,
            tau: initial_tau,
        };
        
        // 网格搜索优化
        for amp_factor in [0.8, 0.9, 1.0, 1.1, 1.2] {
            for center_offset in [-0.1, -0.05, 0.0, 0.05, 0.1] {
                for sigma_factor in [0.8, 0.9, 1.0, 1.1, 1.2] {
                    for gamma_factor in [0.5, 0.7, 1.0, 1.3, 1.5] {
                        for tau_factor in [0.2, 0.4, 0.6, 0.8, 1.0] {
                            let test_params = VoigtExponentialTailParams {
                                amplitude: initial_amplitude * amp_factor,
                                center: initial_center + center_offset,
                                sigma: initial_sigma * sigma_factor,
                                gamma: initial_gamma * gamma_factor,
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
        }
        
        // 计算拟合质量
        let rsquared = self.calculate_rsquared(x_data, y_data, &best_params);
        let standard_error = (best_error / (x_data.len() as f64 - 5.0)).sqrt();
        
        // 计算FWHM
        let fwhm = self.calculate_voigt_fwhm(&best_params);
        
        // 计算Voigt混合参数和尾贡献
        let voigt_mixing = best_params.gamma / (best_params.sigma + best_params.gamma);
        let tail_contribution = best_params.tau / (best_params.sigma + best_params.tau);
        
        Ok(VoigtExponentialTailFitResult {
            amplitude: best_params.amplitude,
            center: best_params.center,
            sigma: best_params.sigma,
            gamma: best_params.gamma,
            tau: best_params.tau,
            fwhm,
            voigt_mixing,
            tail_contribution,
            amplitude_error: standard_error,
            center_error: standard_error,
            sigma_error: standard_error,
            gamma_error: standard_error,
            tau_error: standard_error,
            rsquared,
            standard_error,
        })
    }
    
    /// 计算拟合误差
    fn calculate_fit_error(&self, x_data: &[f64], y_data: &[f64], params: &VoigtExponentialTailParams) -> f64 {
        let mut error = 0.0;
        for (i, &x) in x_data.iter().enumerate() {
            let predicted = self.voigt_exponential_tail_function(x, params);
            error += (y_data[i] - predicted).powi(2);
        }
        error
    }
    
    /// Voigt + 指数尾函数
    fn voigt_exponential_tail_function(&self, x: f64, params: &VoigtExponentialTailParams) -> f64 {
        // Voigt函数部分
        let voigt_part = self.voigt_function(x, params);
        
        // 指数尾部分
        let tail_part = if x > params.center {
            params.amplitude * 0.1 * (-(x - params.center) / params.tau).exp()
        } else {
            0.0
        };
        
        voigt_part + tail_part
    }
    
    /// Voigt函数
    fn voigt_function(&self, x: f64, params: &VoigtExponentialTailParams) -> f64 {
        // 简化的Voigt函数实现
        // 实际应用中应使用更精确的Voigt函数实现
        
        // 高斯部分
        let gaussian_exponent = -((x - params.center).powi(2)) / (2.0 * params.sigma.powi(2));
        let gaussian = params.amplitude * gaussian_exponent.exp();
        
        // 洛伦兹部分
        let lorentzian_denominator = 1.0 + ((x - params.center) / params.gamma).powi(2);
        let lorentzian = params.amplitude / lorentzian_denominator;
        
        // Voigt混合（简化版本）
        let mixing = params.gamma / (params.sigma + params.gamma);
        mixing * lorentzian + (1.0 - mixing) * gaussian
    }
    
    /// 计算Voigt的FWHM
    fn calculate_voigt_fwhm(&self, params: &VoigtExponentialTailParams) -> f64 {
        // Voigt的FWHM近似计算
        let gaussian_fwhm = 2.355 * params.sigma;
        let lorentzian_fwhm = 2.0 * params.gamma;
        
        // 经验公式
        let fwhm_squared = 0.5346 * lorentzian_fwhm + (0.2166 * lorentzian_fwhm.powi(2) + gaussian_fwhm.powi(2)).sqrt();
        fwhm_squared
    }
    
    /// 计算R²
    fn calculate_rsquared(&self, x_data: &[f64], y_data: &[f64], params: &VoigtExponentialTailParams) -> f64 {
        let y_mean: f64 = y_data.iter().sum::<f64>() / y_data.len() as f64;
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;

        for (i, &y) in y_data.iter().enumerate() {
            let y_fit = self.voigt_exponential_tail_function(x_data[i], params);
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

/// Voigt + 指数尾拟合参数
#[derive(Debug, Clone)]
struct VoigtExponentialTailParams {
    amplitude: f64,
    center: f64,
    sigma: f64,    // 高斯宽度参数
    gamma: f64,    // 洛伦兹宽度参数
    tau: f64,      // 指数尾衰减常数
}

/// Voigt + 指数尾拟合结果
#[derive(Debug)]
struct VoigtExponentialTailFitResult {
    amplitude: f64,
    center: f64,
    sigma: f64,
    gamma: f64,
    tau: f64,
    fwhm: f64,
    voigt_mixing: f64,      // Voigt混合参数
    tail_contribution: f64, // 指数尾贡献
    amplitude_error: f64,
    center_error: f64,
    sigma_error: f64,
    gamma_error: f64,
    tau_error: f64,
    rsquared: f64,
    standard_error: f64,
}
