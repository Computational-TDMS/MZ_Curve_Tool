//! GMG贝叶斯拟合器
//! 
//! 实现GMG (Gaussian Mixture with Gaussian) 贝叶斯拟合算法，适用于复杂峰形的贝叶斯分析

use crate::core::data::{Curve, Peak, ProcessingError, PeakType};
use crate::core::processors::peak_fitting::PeakFitter;
use serde_json::Value;

/// GMG贝叶斯拟合器
#[derive(Debug)]
pub struct GMGBayesianFitter {
    /// 最大迭代次数
    max_iterations: usize,
    /// 收敛阈值
    convergence_threshold: f64,
    /// 正则化参数
    regularization: f64,
    /// 贝叶斯先验强度
    prior_strength: f64,
    /// 混合成分数量
    mixture_components: usize,
}

impl PeakFitter for GMGBayesianFitter {
    fn name(&self) -> &str {
        "gmg_bayesian_fitter"
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
        
        if x_data.len() < 6 {
            return Err(ProcessingError::process_error(
                "GMG贝叶斯拟合需要至少6个数据点"
            ));
        }

        // 执行GMG贝叶斯拟合
        let fit_result = self.fit_gmg_bayesian(&x_data, &y_data, peak)?;
        
        // 创建拟合后的峰
        let mut fitted_peak = peak.clone();
        fitted_peak.peak_type = PeakType::GMGBayesian;
        fitted_peak.amplitude = fit_result.amplitude;
        fitted_peak.center = fit_result.center;
        fitted_peak.sigma = fit_result.sigma;
        
        // 计算GMG的FWHM
        fitted_peak.fwhm = self.calculate_gmg_fwhm(&fit_result);
        fitted_peak.hwhm = fitted_peak.fwhm / 2.0;
        
        // 设置拟合参数
        let mut parameters = vec![
            fit_result.amplitude,
            fit_result.center,
            fit_result.sigma,
        ];
        parameters.extend(&fit_result.mixture_weights);
        parameters.extend(&fit_result.mixture_means);
        parameters.extend(&fit_result.mixture_sigmas);
        let parameter_errors = vec![0.0; parameters.len()]; // 简化，实际应计算参数误差
        fitted_peak.set_fit_parameters(parameters, parameter_errors, None);
        
        // 计算峰面积
        fitted_peak.calculate_area_from_fit();
        
        // 添加GMG贝叶斯特定元数据
        fitted_peak.add_metadata("gmg_bayesian_fitted".to_string(), serde_json::json!(true));
        fitted_peak.add_metadata("mixture_components".to_string(), serde_json::json!(fit_result.mixture_weights.len()));
        fitted_peak.add_metadata("mixture_weights".to_string(), serde_json::json!(fit_result.mixture_weights));
        fitted_peak.add_metadata("bayesian_evidence".to_string(), serde_json::json!(fit_result.bayesian_evidence));
        fitted_peak.add_metadata("model_complexity".to_string(), serde_json::json!(self.calculate_model_complexity(&fit_result)));
        fitted_peak.add_metadata("bayesian_confidence".to_string(), serde_json::json!(self.calculate_bayesian_confidence(&fit_result)));
        
        Ok(fitted_peak)
    }
}

impl GMGBayesianFitter {
    /// 创建新的GMG贝叶斯拟合器
    pub fn new() -> Self {
        Self {
            max_iterations: 100,
            convergence_threshold: 1e-6,
            regularization: 0.01,
            prior_strength: 1.0,
            mixture_components: 2, // 默认2个混合成分
        }
    }
    
    /// 设置参数
    pub fn with_parameters(
        mut self,
        max_iterations: usize,
        convergence_threshold: f64,
        regularization: f64,
        prior_strength: f64,
        mixture_components: usize,
    ) -> Self {
        self.max_iterations = max_iterations;
        self.convergence_threshold = convergence_threshold;
        self.regularization = regularization;
        self.prior_strength = prior_strength;
        self.mixture_components = mixture_components;
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
    
    /// 执行GMG贝叶斯拟合
    fn fit_gmg_bayesian(
        &self,
        x_data: &[f64],
        y_data: &[f64],
        initial_peak: &Peak,
    ) -> Result<GMGBayesianParams, ProcessingError> {
        // 初始化参数
        let initial_amplitude = initial_peak.amplitude;
        let initial_center = initial_peak.center;
        let initial_sigma = initial_peak.sigma.max(0.1);
        
        // GMG贝叶斯参数初始化
        let mut params = GMGBayesianParams {
            amplitude: initial_amplitude,
            center: initial_center,
            sigma: initial_sigma,
            mixture_weights: vec![1.0 / self.mixture_components as f64; self.mixture_components],
            mixture_means: vec![initial_center; self.mixture_components],
            mixture_sigmas: vec![initial_sigma; self.mixture_components],
            bayesian_evidence: 0.0,
        };
        
        // 使用变分贝叶斯EM算法
        for _iteration in 0..self.max_iterations {
            // E步骤：计算后验分布
            let posterior = self.expectation_step(x_data, y_data, &params)?;
            
            // M步骤：最大化后验
            let new_params = self.maximization_step(x_data, y_data, &posterior, &params)?;
            
            // 计算贝叶斯证据
            let evidence = self.calculate_bayesian_evidence(x_data, y_data, &new_params);
            let mut updated_params = new_params;
            updated_params.bayesian_evidence = evidence;
            
            // 检查收敛
            if self.check_convergence(&params, &updated_params) {
                return Ok(updated_params);
            }
            
            params = updated_params;
        }
        
        Ok(params)
    }
    
    /// E步骤：计算后验分布
    fn expectation_step(
        &self,
        x_data: &[f64],
        _y_data: &[f64],
        params: &GMGBayesianParams,
    ) -> Result<GMGPosterior, ProcessingError> {
        let mut responsibilities = vec![vec![0.0; self.mixture_components]; x_data.len()];
        let mut _log_likelihood = 0.0;
        
        for (i, &x) in x_data.iter().enumerate() {
            let mut total_prob = 0.0;
            let mut component_probs = vec![0.0; self.mixture_components];
            
            for (j, &weight) in params.mixture_weights.iter().enumerate() {
                let mean = params.mixture_means[j];
                let sigma = params.mixture_sigmas[j];
                
                // 计算高斯概率密度
                let prob = weight * self.gaussian_pdf(x, mean, sigma);
                component_probs[j] = prob;
                total_prob += prob;
            }
            
            // 归一化责任
            if total_prob > 0.0 {
                for j in 0..self.mixture_components {
                    responsibilities[i][j] = component_probs[j] / total_prob;
                }
                _log_likelihood += total_prob.ln();
            }
        }
        
        Ok(GMGPosterior {
            responsibilities,
        })
    }
    
    /// M步骤：最大化后验
    fn maximization_step(
        &self,
        x_data: &[f64],
        y_data: &[f64],
        posterior: &GMGPosterior,
        old_params: &GMGBayesianParams,
    ) -> Result<GMGBayesianParams, ProcessingError> {
        let mut new_weights = vec![0.0; self.mixture_components];
        let mut new_means = vec![0.0; self.mixture_components];
        let mut new_sigmas = vec![0.0; self.mixture_components];
        
        // 更新混合权重
        for j in 0..self.mixture_components {
            let mut weight_sum = 0.0;
            for i in 0..x_data.len() {
                weight_sum += posterior.responsibilities[i][j];
            }
            new_weights[j] = (weight_sum + self.prior_strength) / (x_data.len() as f64 + self.prior_strength * self.mixture_components as f64);
        }
        
        // 更新混合均值和方差
        for j in 0..self.mixture_components {
            let mut mean_sum = 0.0;
            let mut var_sum = 0.0;
            let mut weight_sum = 0.0;
            
            for i in 0..x_data.len() {
                let resp = posterior.responsibilities[i][j];
                mean_sum += resp * x_data[i];
                weight_sum += resp;
            }
            
            if weight_sum > 0.0 {
                new_means[j] = mean_sum / weight_sum;
                
                for i in 0..x_data.len() {
                    let resp = posterior.responsibilities[i][j];
                    let diff = x_data[i] - new_means[j];
                    var_sum += resp * diff * diff;
                }
                new_sigmas[j] = (var_sum / weight_sum).sqrt().max(0.01);
            } else {
                new_means[j] = old_params.mixture_means[j];
                new_sigmas[j] = old_params.mixture_sigmas[j];
            }
        }
        
        // 计算整体峰参数
        let amplitude = y_data.iter().fold(0.0_f64, |a, &b| a.max(b));
        let center = new_means.iter().zip(&new_weights).map(|(m, w)| m * w).sum::<f64>();
        let sigma = new_sigmas.iter().zip(&new_weights).map(|(s, w)| s * w).sum::<f64>();
        
        Ok(GMGBayesianParams {
            amplitude,
            center,
            sigma,
            mixture_weights: new_weights,
            mixture_means: new_means,
            mixture_sigmas: new_sigmas,
            bayesian_evidence: 0.0, // 将在外部计算
        })
    }
    
    /// 计算贝叶斯证据
    fn calculate_bayesian_evidence(&self, x_data: &[f64], _y_data: &[f64], params: &GMGBayesianParams) -> f64 {
        let mut evidence = 0.0;
        
        for &x in x_data {
            let mut component_sum = 0.0;
            for (j, &weight) in params.mixture_weights.iter().enumerate() {
                let mean = params.mixture_means[j];
                let sigma = params.mixture_sigmas[j];
                component_sum += weight * self.gaussian_pdf(x, mean, sigma);
            }
            evidence += component_sum.ln();
        }
        
        // 添加先验项
        let prior_term = -self.prior_strength * params.mixture_weights.iter().map(|&w| w.ln()).sum::<f64>();
        evidence + prior_term
    }
    
    /// 高斯概率密度函数
    fn gaussian_pdf(&self, x: f64, mean: f64, sigma: f64) -> f64 {
        let diff = x - mean;
        let exponent = -diff * diff / (2.0 * sigma * sigma);
        (exponent).exp() / (sigma * (2.0 * std::f64::consts::PI).sqrt())
    }
    
    /// 检查收敛
    fn check_convergence(&self, old_params: &GMGBayesianParams, new_params: &GMGBayesianParams) -> bool {
        let amplitude_diff = (old_params.amplitude - new_params.amplitude).abs() / old_params.amplitude.max(1e-6);
        let center_diff = (old_params.center - new_params.center).abs();
        let sigma_diff = (old_params.sigma - new_params.sigma).abs() / old_params.sigma.max(1e-6);
        
        if amplitude_diff > self.convergence_threshold ||
           center_diff > self.convergence_threshold ||
           sigma_diff > self.convergence_threshold {
            return false;
        }
        
        // 检查混合参数的收敛
        for (old_weight, new_weight) in old_params.mixture_weights.iter().zip(new_params.mixture_weights.iter()) {
            let weight_diff = (old_weight - new_weight).abs();
            if weight_diff > self.convergence_threshold {
                return false;
            }
        }
        
        true
    }
    
    /// 计算GMG的FWHM
    fn calculate_gmg_fwhm(&self, params: &GMGBayesianParams) -> f64 {
        // 基于混合成分计算FWHM
        let mut weighted_fwhm = 0.0;
        for (j, &weight) in params.mixture_weights.iter().enumerate() {
            let component_fwhm = 2.355 * params.mixture_sigmas[j];
            weighted_fwhm += weight * component_fwhm;
        }
        weighted_fwhm
    }
    
    /// 计算模型复杂度
    fn calculate_model_complexity(&self, params: &GMGBayesianParams) -> f64 {
        // 基于混合成分数量和参数数量计算复杂度
        let param_count = 3 + params.mixture_weights.len() * 3; // 基础参数 + 混合参数
        let complexity = param_count as f64 / 10.0; // 归一化
        complexity.min(1.0)
    }
    
    /// 计算贝叶斯置信度
    fn calculate_bayesian_confidence(&self, params: &GMGBayesianParams) -> f64 {
        // 基于贝叶斯证据和模型复杂度计算置信度
        let evidence_confidence = (params.bayesian_evidence / 100.0).exp().min(1.0);
        let complexity_penalty = 1.0 - self.calculate_model_complexity(params) * 0.1;
        evidence_confidence * complexity_penalty
    }
}

/// GMG贝叶斯参数
#[derive(Debug, Clone)]
struct GMGBayesianParams {
    amplitude: f64,
    center: f64,
    sigma: f64,
    mixture_weights: Vec<f64>,
    mixture_means: Vec<f64>,
    mixture_sigmas: Vec<f64>,
    bayesian_evidence: f64,
}

/// GMG后验分布
#[derive(Debug)]
struct GMGPosterior {
    responsibilities: Vec<Vec<f64>>,
}
