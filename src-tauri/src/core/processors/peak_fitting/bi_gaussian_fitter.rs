//! Bi-Gaussian 拟合器
//! 
//! 实现双高斯峰的拟合算法，适用于不对称峰的分析

use crate::core::data::{Curve, Peak, ProcessingError, PeakType};
use crate::core::processors::peak_fitting::PeakFitter;
use serde_json::Value;

/// Bi-Gaussian拟合器
#[derive(Debug)]
pub struct BiGaussianFitter;

impl PeakFitter for BiGaussianFitter {
    fn name(&self) -> &str {
        "bi_gaussian_fitter"
    }

    fn fit_peak(&self, peak: &Peak, curve: &Curve, config: &Value) -> Result<Peak, ProcessingError> {
        // 提取拟合窗口
        let window_size = config["fit_window_size"].as_f64().unwrap_or(3.0);
        let (x_data, y_data) = self.extract_fit_data(curve, peak.center, window_size);
        
        if x_data.len() < 6 {
            return Err(ProcessingError::process_error(
                "Bi-Gaussian拟合需要至少6个数据点"
            ));
        }

        // 执行Bi-Gaussian拟合
        let fit_result = self.fit_bi_gaussian(&x_data, &y_data, peak)?;
        
        // 创建拟合后的峰
        let mut fitted_peak = peak.clone();
        fitted_peak.peak_type = PeakType::BiGaussian;
        fitted_peak.center = fit_result.center;
        fitted_peak.amplitude = fit_result.amplitude;
        fitted_peak.sigma = (fit_result.sigma_left + fit_result.sigma_right) / 2.0;
        fitted_peak.fwhm = fit_result.fwhm;
        fitted_peak.hwhm = fit_result.fwhm / 2.0;
        
        // 计算左右半峰宽
        fitted_peak.left_hwhm = fit_result.sigma_left * 1.177; // HWHM = 1.177 * sigma
        fitted_peak.right_hwhm = fit_result.sigma_right * 1.177;
        fitted_peak.calculate_asymmetry_factor();
        
        // 设置拟合参数
        let parameters = vec![
            fit_result.amplitude,
            fit_result.center,
            fit_result.sigma_left,
            fit_result.sigma_right,
            fit_result.mixing_parameter,
        ];
        let parameter_errors = vec![
            fit_result.amplitude_error,
            fit_result.center_error,
            fit_result.sigma_left_error,
            fit_result.sigma_right_error,
            fit_result.mixing_error,
        ];
        fitted_peak.set_fit_parameters(parameters, parameter_errors, None);
        
        // 计算峰面积
        fitted_peak.calculate_area_from_fit();
        
        // 计算拟合质量
        fitted_peak.rsquared = fit_result.rsquared;
        fitted_peak.standard_error = fit_result.standard_error;
        
        // 添加Bi-Gaussian特定元数据
        fitted_peak.add_metadata("sigma_left".to_string(), serde_json::json!(fit_result.sigma_left));
        fitted_peak.add_metadata("sigma_right".to_string(), serde_json::json!(fit_result.sigma_right));
        fitted_peak.add_metadata("mixing_parameter".to_string(), serde_json::json!(fit_result.mixing_parameter));
        fitted_peak.add_metadata("asymmetry_index".to_string(), serde_json::json!(fit_result.sigma_right / fit_result.sigma_left));
        
        Ok(fitted_peak)
    }
}

impl BiGaussianFitter {
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
    
    /// 执行Bi-Gaussian拟合
    fn fit_bi_gaussian(&self, x_data: &[f64], y_data: &[f64], initial_peak: &Peak) -> Result<BiGaussianFitResult, ProcessingError> {
        // 初始参数估计
        let initial_amplitude = initial_peak.amplitude;
        let initial_center = initial_peak.center;
        let initial_sigma = initial_peak.sigma.max(0.1);
        
        // 估计左右sigma（基于峰的不对称性）
        let asymmetry = if initial_peak.asymmetry_factor > 0.0 {
            initial_peak.asymmetry_factor
        } else {
            1.2 // 默认轻微不对称
        };
        
        let initial_sigma_left = initial_sigma / asymmetry.sqrt();
        let initial_sigma_right = initial_sigma * asymmetry.sqrt();
        let initial_mixing = 0.5; // 初始混合参数
        
        // 使用网格搜索优化
        let mut best_error = f64::INFINITY;
        let mut best_params = BiGaussianParams {
            amplitude: initial_amplitude,
            center: initial_center,
            sigma_left: initial_sigma_left,
            sigma_right: initial_sigma_right,
            mixing_parameter: initial_mixing,
        };
        
        // 网格搜索优化
        for amp_factor in [0.8, 0.9, 1.0, 1.1, 1.2] {
            for center_offset in [-0.1, -0.05, 0.0, 0.05, 0.1] {
                for sigma_factor in [0.8, 0.9, 1.0, 1.1, 1.2] {
                    for mixing in [0.3, 0.4, 0.5, 0.6, 0.7] {
                        let test_params = BiGaussianParams {
                            amplitude: initial_amplitude * amp_factor,
                            center: initial_center + center_offset,
                            sigma_left: initial_sigma_left * sigma_factor,
                            sigma_right: initial_sigma_right * sigma_factor,
                            mixing_parameter: mixing,
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
        let standard_error = (best_error / (x_data.len() as f64 - 5.0)).sqrt();
        
        // 计算FWHM
        let fwhm = self.calculate_bi_gaussian_fwhm(&best_params);
        
        Ok(BiGaussianFitResult {
            amplitude: best_params.amplitude,
            center: best_params.center,
            sigma_left: best_params.sigma_left,
            sigma_right: best_params.sigma_right,
            mixing_parameter: best_params.mixing_parameter,
            fwhm,
            amplitude_error: standard_error,
            center_error: standard_error,
            sigma_left_error: standard_error,
            sigma_right_error: standard_error,
            mixing_error: standard_error,
            rsquared,
            standard_error,
        })
    }
    
    /// 计算拟合误差
    fn calculate_fit_error(&self, x_data: &[f64], y_data: &[f64], params: &BiGaussianParams) -> f64 {
        let mut error = 0.0;
        for (i, &x) in x_data.iter().enumerate() {
            let predicted = self.bi_gaussian_function(x, params);
            error += (y_data[i] - predicted).powi(2);
        }
        error
    }
    
    /// Bi-Gaussian函数
    fn bi_gaussian_function(&self, x: f64, params: &BiGaussianParams) -> f64 {
        let left_gaussian = if x <= params.center {
            let exponent = -((x - params.center).powi(2)) / (2.0 * params.sigma_left.powi(2));
            params.amplitude * params.mixing_parameter * exponent.exp()
        } else {
            0.0
        };
        
        let right_gaussian = if x >= params.center {
            let exponent = -((x - params.center).powi(2)) / (2.0 * params.sigma_right.powi(2));
            params.amplitude * (1.0 - params.mixing_parameter) * exponent.exp()
        } else {
            0.0
        };
        
        left_gaussian + right_gaussian
    }
    
    /// 计算Bi-Gaussian的FWHM
    fn calculate_bi_gaussian_fwhm(&self, params: &BiGaussianParams) -> f64 {
        // Bi-Gaussian的FWHM是左右FWHM的加权平均
        let left_fwhm = 2.355 * params.sigma_left;
        let right_fwhm = 2.355 * params.sigma_right;
        params.mixing_parameter * left_fwhm + (1.0 - params.mixing_parameter) * right_fwhm
    }
    
    /// 计算R²
    fn calculate_rsquared(&self, x_data: &[f64], y_data: &[f64], params: &BiGaussianParams) -> f64 {
        let y_mean: f64 = y_data.iter().sum::<f64>() / y_data.len() as f64;
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;

        for (i, &y) in y_data.iter().enumerate() {
            let y_fit = self.bi_gaussian_function(x_data[i], params);
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

/// Bi-Gaussian拟合参数
#[derive(Debug, Clone)]
struct BiGaussianParams {
    amplitude: f64,
    center: f64,
    sigma_left: f64,
    sigma_right: f64,
    mixing_parameter: f64, // 左高斯峰的权重
}

/// Bi-Gaussian拟合结果
#[derive(Debug)]
struct BiGaussianFitResult {
    amplitude: f64,
    center: f64,
    sigma_left: f64,
    sigma_right: f64,
    mixing_parameter: f64,
    fwhm: f64,
    amplitude_error: f64,
    center_error: f64,
    sigma_left_error: f64,
    sigma_right_error: f64,
    mixing_error: f64,
    rsquared: f64,
    standard_error: f64,
}
