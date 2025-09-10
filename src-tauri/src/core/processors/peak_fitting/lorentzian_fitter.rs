//! 洛伦兹峰拟合器
//! 
//! 实现洛伦兹峰拟合算法

use crate::core::data::{Curve, Peak, ProcessingError, PeakType};
use crate::core::processors::peak_fitting::PeakFitter;
use serde_json::Value;

/// 洛伦兹峰拟合器
#[derive(Debug)]
pub struct LorentzianFitter;

impl PeakFitter for LorentzianFitter {
    fn name(&self) -> &str {
        "lorentzian_fitter"
    }

    fn fit_peak(&self, peak: &Peak, curve: &Curve, config: &Value) -> Result<Peak, ProcessingError> {
        let min_peak_width = config["min_peak_width"].as_f64().unwrap_or(0.1);
        let max_peak_width = config["max_peak_width"].as_f64().unwrap_or(10.0);

        // 确定拟合窗口
        let window_size = self.calculate_fit_window(peak, curve, min_peak_width, max_peak_width);
        
        // 提取拟合数据
        let (x_data, y_data) = self.extract_fit_data(curve, peak.center, window_size);
        
        if x_data.len() < 3 {
            // 数据点太少，返回原始峰
            return Ok(peak.clone());
        }

        // 进行洛伦兹拟合
        self.fit_lorentzian(peak, &x_data, &y_data)
    }
}

impl LorentzianFitter {
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

    /// 洛伦兹拟合实现
    fn fit_lorentzian(&self, peak: &Peak, x_data: &[f64], y_data: &[f64]) -> Result<Peak, ProcessingError> {
        // 简化的洛伦兹拟合实现
        let result = self.least_squares_lorentzian_fit(x_data, y_data)?;
        
        let mut fitted_peak = peak.clone();
        
        // 更新拟合参数
        fitted_peak.center = result.center;
        fitted_peak.amplitude = result.amplitude;
        fitted_peak.gamma = result.gamma;
        fitted_peak.fwhm = 2.0 * result.gamma; // 洛伦兹峰的FWHM = 2 * gamma
        fitted_peak.hwhm = result.gamma; // 洛伦兹峰的HWHM = gamma
        fitted_peak.peak_type = PeakType::Lorentzian;
        
        // 设置拟合参数
        let parameters = vec![result.amplitude, result.center, result.gamma];
        let parameter_errors = vec![result.amplitude_error, result.center_error, result.gamma_error];
        fitted_peak.set_fit_parameters(parameters, parameter_errors, None);
        
        // 计算峰面积
        fitted_peak.area = result.amplitude * result.gamma * std::f64::consts::PI;
        
        // 计算拟合质量
        fitted_peak.rsquared = result.rsquared;
        fitted_peak.standard_error = result.standard_error;
        
        Ok(fitted_peak)
    }

    /// 最小二乘法洛伦兹拟合
    fn least_squares_lorentzian_fit(&self, x_data: &[f64], y_data: &[f64]) -> Result<LorentzianFitResult, ProcessingError> {
        if x_data.len() != y_data.len() || x_data.len() < 3 {
            return Err(ProcessingError::DataError("数据点不足".to_string()));
        }

        // 初始参数估计
        let max_idx = y_data.iter().enumerate().max_by(|a, b| a.1.partial_cmp(b.1).unwrap()).unwrap().0;
        let initial_amplitude = y_data[max_idx];
        let initial_center = x_data[max_idx];
        
        // 估计gamma
        let mut gamma_sum = 0.0;
        let mut gamma_count = 0;
        for (i, &y) in y_data.iter().enumerate() {
            if y > initial_amplitude / 2.0 {
                let dx = (x_data[i] - initial_center).abs();
                gamma_sum += dx;
                gamma_count += 1;
            }
        }
        let initial_gamma = if gamma_count > 0 { gamma_sum / gamma_count as f64 } else { 1.0 };

        // 简化的拟合过程
        let mut best_params = LorentzianParams {
            amplitude: initial_amplitude,
            center: initial_center,
            gamma: initial_gamma,
        };

        let mut best_error = f64::INFINITY;
        
        // 简单的网格搜索优化
        for amp_factor in [0.8, 0.9, 1.0, 1.1, 1.2] {
            for center_offset in [-0.1, -0.05, 0.0, 0.05, 0.1] {
                for gamma_factor in [0.8, 0.9, 1.0, 1.1, 1.2] {
                    let params = LorentzianParams {
                        amplitude: initial_amplitude * amp_factor,
                        center: initial_center + center_offset,
                        gamma: initial_gamma * gamma_factor,
                    };
                    
                    let error = self.calculate_fit_error(x_data, y_data, &params);
                    if error < best_error {
                        best_error = error;
                        best_params = params;
                    }
                }
            }
        }

        // 计算拟合质量
        let rsquared = self.calculate_rsquared(x_data, y_data, &best_params);
        let standard_error = (best_error / (x_data.len() as f64 - 3.0)).sqrt();

        Ok(LorentzianFitResult {
            amplitude: best_params.amplitude,
            center: best_params.center,
            gamma: best_params.gamma,
            amplitude_error: standard_error,
            center_error: standard_error,
            gamma_error: standard_error,
            rsquared,
            standard_error,
        })
    }

    /// 计算拟合误差
    fn calculate_fit_error(&self, x_data: &[f64], y_data: &[f64], params: &LorentzianParams) -> f64 {
        let mut error = 0.0;
        for (i, &x) in x_data.iter().enumerate() {
            let predicted = self.lorentzian_function(x, params);
            error += (y_data[i] - predicted).powi(2);
        }
        error
    }

    /// 洛伦兹函数
    fn lorentzian_function(&self, x: f64, params: &LorentzianParams) -> f64 {
        let denominator = 1.0 + ((x - params.center) / params.gamma).powi(2);
        params.amplitude / denominator
    }

    /// 计算R²
    fn calculate_rsquared(&self, x_data: &[f64], y_data: &[f64], params: &LorentzianParams) -> f64 {
        let y_mean: f64 = y_data.iter().sum::<f64>() / y_data.len() as f64;
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;

        for (i, &y) in y_data.iter().enumerate() {
            let y_fit = self.lorentzian_function(x_data[i], params);
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

/// 洛伦兹拟合参数
#[derive(Debug, Clone)]
struct LorentzianParams {
    amplitude: f64,
    center: f64,
    gamma: f64,
}

/// 洛伦兹拟合结果
#[derive(Debug)]
struct LorentzianFitResult {
    amplitude: f64,
    center: f64,
    gamma: f64,
    amplitude_error: f64,
    center_error: f64,
    gamma_error: f64,
    rsquared: f64,
    standard_error: f64,
}
