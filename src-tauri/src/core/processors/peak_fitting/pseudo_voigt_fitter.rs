//! 伪Voigt峰拟合器
//! 
//! 实现伪Voigt峰拟合算法（高斯和洛伦兹的混合）

use crate::core::data::{Curve, Peak, ProcessingError, PeakType};
use crate::core::processors::peak_fitting::PeakFitter;
use serde_json::Value;

/// 伪Voigt峰拟合器
#[derive(Debug)]
pub struct PseudoVoigtFitter;

impl PeakFitter for PseudoVoigtFitter {
    fn name(&self) -> &str {
        "pseudo_voigt_fitter"
    }

    fn fit_peak(&self, peak: &Peak, curve: &Curve, config: &Value) -> Result<Peak, ProcessingError> {
        let min_peak_width = config["min_peak_width"].as_f64().unwrap_or(0.1);
        let max_peak_width = config["max_peak_width"].as_f64().unwrap_or(10.0);

        // 确定拟合窗口
        let window_size = self.calculate_fit_window(peak, curve, min_peak_width, max_peak_width);
        
        // 提取拟合数据
        let (x_data, y_data) = self.extract_fit_data(curve, peak.center, window_size);
        
        if x_data.len() < 4 {
            // 数据点太少，返回原始峰
            return Ok(peak.clone());
        }

        // 进行伪Voigt拟合
        self.fit_pseudo_voigt(peak, &x_data, &y_data)
    }
}

impl PseudoVoigtFitter {
    /// 计算拟合窗口大小
    fn calculate_fit_window(&self, peak: &Peak, curve: &Curve, min_width: f64, max_width: f64) -> f64 {
        // 基于峰高和曲线特征计算窗口大小
        let peak_height = peak.amplitude;
        let curve_std = curve.intensity_std;
        
        // 动态计算窗口大小
        let estimated_width = (peak_height / curve_std).sqrt() * 0.5;
        estimated_width.max(min_width).min(max_width)
    }

    /// 提取拟合数据
    fn extract_fit_data(&self, curve: &Curve, center: f64, window_size: f64) -> (Vec<f64>, Vec<f64>) {
        let mut x_data = Vec::new();
        let mut y_data = Vec::new();

        for (i, &x) in curve.x_values.iter().enumerate() {
            if (x - center).abs() <= window_size {
                x_data.push(x);
                y_data.push(curve.y_values[i]);
            }
        }

        (x_data, y_data)
    }

    /// 伪Voigt拟合实现
    fn fit_pseudo_voigt(&self, peak: &Peak, x_data: &[f64], y_data: &[f64]) -> Result<Peak, ProcessingError> {
        // 简化的伪Voigt拟合实现
        let result = self.least_squares_pseudo_voigt_fit(x_data, y_data)?;
        
        let mut fitted_peak = peak.clone();
        
        // 更新拟合参数
        fitted_peak.center = result.center;
        fitted_peak.amplitude = result.amplitude;
        fitted_peak.sigma = result.sigma;
        fitted_peak.gamma = result.gamma;
        fitted_peak.mixing_parameter = result.mixing_parameter;
        fitted_peak.peak_type = PeakType::PseudoVoigt;
        
        // 计算FWHM（伪Voigt的FWHM是高斯和洛伦兹的加权平均）
        let gaussian_fwhm = result.sigma * 2.355;
        let lorentzian_fwhm = 2.0 * result.gamma;
        fitted_peak.fwhm = result.mixing_parameter * lorentzian_fwhm + (1.0 - result.mixing_parameter) * gaussian_fwhm;
        fitted_peak.hwhm = fitted_peak.fwhm / 2.0;
        
        // 设置拟合参数
        let parameters = vec![result.amplitude, result.center, result.sigma, result.gamma, result.mixing_parameter];
        let parameter_errors = vec![result.amplitude_error, result.center_error, result.sigma_error, result.gamma_error, result.mixing_error];
        fitted_peak.set_fit_parameters(parameters, parameter_errors, None);
        
        // 计算峰面积
        let gaussian_area = result.amplitude * result.sigma * (std::f64::consts::PI * 2.0).sqrt();
        let lorentzian_area = result.amplitude * result.gamma * std::f64::consts::PI;
        fitted_peak.area = result.mixing_parameter * lorentzian_area + (1.0 - result.mixing_parameter) * gaussian_area;
        
        // 计算拟合质量
        fitted_peak.rsquared = result.rsquared;
        fitted_peak.standard_error = result.standard_error;
        
        Ok(fitted_peak)
    }

    /// 最小二乘法伪Voigt拟合
    fn least_squares_pseudo_voigt_fit(&self, x_data: &[f64], y_data: &[f64]) -> Result<PseudoVoigtFitResult, ProcessingError> {
        if x_data.len() != y_data.len() || x_data.len() < 4 {
            return Err(ProcessingError::DataError("数据点不足".to_string()));
        }

        // 初始参数估计
        let max_idx = y_data.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
        let initial_amplitude = y_data[max_idx];
        let initial_center = x_data[max_idx];
        
        // 估计sigma和gamma
        let mut width_sum = 0.0;
        let mut width_count = 0;
        for (i, &y) in y_data.iter().enumerate() {
            if y > initial_amplitude / 2.0 {
                let dx = (x_data[i] - initial_center).abs();
                width_sum += dx;
                width_count += 1;
            }
        }
        let initial_width = if width_count > 0 { width_sum / width_count as f64 } else { 1.0 };
        let initial_sigma = initial_width / 2.355; // 转换为sigma
        let initial_gamma = initial_width / 2.0; // 转换为gamma

        // 简化的拟合过程
        let mut best_params = PseudoVoigtParams {
            amplitude: initial_amplitude,
            center: initial_center,
            sigma: initial_sigma,
            gamma: initial_gamma,
            mixing_parameter: 0.5, // 50% 洛伦兹，50% 高斯
        };

        let mut best_error = f64::INFINITY;
        
        // 简单的网格搜索优化
        for amp_factor in [0.8, 0.9, 1.0, 1.1, 1.2] {
            for center_offset in [-0.1, -0.05, 0.0, 0.05, 0.1] {
                for sigma_factor in [0.8, 0.9, 1.0, 1.1, 1.2] {
                    for gamma_factor in [0.8, 0.9, 1.0, 1.1, 1.2] {
                        for mixing in [0.0, 0.25, 0.5, 0.75, 1.0] {
                            let params = PseudoVoigtParams {
                                amplitude: initial_amplitude * amp_factor,
                                center: initial_center + center_offset,
                                sigma: initial_sigma * sigma_factor,
                                gamma: initial_gamma * gamma_factor,
                                mixing_parameter: mixing,
                            };
                            
                            let error = self.calculate_fit_error(x_data, y_data, &params);
                            if error < best_error {
                                best_error = error;
                                best_params = params;
                            }
                        }
                    }
                }
            }
        }

        // 计算拟合质量
        let rsquared = self.calculate_rsquared(x_data, y_data, &best_params);
        let standard_error = (best_error / (x_data.len() as f64 - 5.0)).sqrt();

        Ok(PseudoVoigtFitResult {
            amplitude: best_params.amplitude,
            center: best_params.center,
            sigma: best_params.sigma,
            gamma: best_params.gamma,
            mixing_parameter: best_params.mixing_parameter,
            amplitude_error: standard_error,
            center_error: standard_error,
            sigma_error: standard_error,
            gamma_error: standard_error,
            mixing_error: standard_error,
            rsquared,
            standard_error,
        })
    }

    /// 计算拟合误差
    fn calculate_fit_error(&self, x_data: &[f64], y_data: &[f64], params: &PseudoVoigtParams) -> f64 {
        let mut error = 0.0;
        for (i, &x) in x_data.iter().enumerate() {
            let predicted = self.pseudo_voigt_function(x, params);
            error += (y_data[i] - predicted).powi(2);
        }
        error
    }

    /// 伪Voigt函数
    fn pseudo_voigt_function(&self, x: f64, params: &PseudoVoigtParams) -> f64 {
        // 高斯部分
        let gaussian_exponent = -((x - params.center).powi(2)) / (2.0 * params.sigma.powi(2));
        let gaussian = params.amplitude * gaussian_exponent.exp();
        
        // 洛伦兹部分
        let lorentzian_denominator = 1.0 + ((x - params.center) / params.gamma).powi(2);
        let lorentzian = params.amplitude / lorentzian_denominator;
        
        // 混合
        params.mixing_parameter * lorentzian + (1.0 - params.mixing_parameter) * gaussian
    }

    /// 计算R²
    fn calculate_rsquared(&self, x_data: &[f64], y_data: &[f64], params: &PseudoVoigtParams) -> f64 {
        let y_mean: f64 = y_data.iter().sum::<f64>() / y_data.len() as f64;
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;

        for (i, &y) in y_data.iter().enumerate() {
            let y_fit = self.pseudo_voigt_function(x_data[i], params);
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

/// 伪Voigt拟合参数
#[derive(Debug, Clone)]
struct PseudoVoigtParams {
    amplitude: f64,
    center: f64,
    sigma: f64,
    gamma: f64,
    mixing_parameter: f64,
}

/// 伪Voigt拟合结果
#[derive(Debug)]
struct PseudoVoigtFitResult {
    amplitude: f64,
    center: f64,
    sigma: f64,
    gamma: f64,
    mixing_parameter: f64,
    amplitude_error: f64,
    center_error: f64,
    sigma_error: f64,
    gamma_error: f64,
    mixing_error: f64,
    rsquared: f64,
    standard_error: f64,
}
