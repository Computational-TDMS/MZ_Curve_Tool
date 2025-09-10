//! FBF (Fast Bayesian Fitting) 重叠峰预处理方法
//! 
//! 实现快速贝叶斯拟合预处理，用于重叠峰的初始估计和分离

use crate::core::data::{Curve, Peak, ProcessingError};
use crate::core::processors::overlapping_peaks::OverlappingPeakProcessor;
use serde_json::Value;

/// FBF预处理器
#[derive(Debug)]
pub struct FBFPreprocessor {
    /// 最大迭代次数
    max_iterations: usize,
    /// 收敛阈值
    convergence_threshold: f64,
    /// 正则化参数
    regularization: f64,
}

impl FBFPreprocessor {
    /// 创建新的FBF预处理器
    pub fn new() -> Self {
        Self {
            max_iterations: 100,
            convergence_threshold: 1e-6,
            regularization: 0.01,
        }
    }
    
    /// 设置参数
    pub fn with_parameters(mut self, max_iterations: usize, convergence_threshold: f64, regularization: f64) -> Self {
        self.max_iterations = max_iterations;
        self.convergence_threshold = convergence_threshold;
        self.regularization = regularization;
        self
    }
    
    /// 对重叠峰进行FBF预处理
    pub fn preprocess_overlapping_peaks(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        config: &Value,
    ) -> Result<Vec<Peak>, ProcessingError> {
        if peaks.len() < 2 {
            return Ok(peaks.to_vec());
        }
        
        // 识别重叠峰组
        let overlapping_groups = self.identify_overlapping_groups(peaks, curve)?;
        
        let mut processed_peaks = Vec::new();
        
        for group in overlapping_groups {
            if group.len() > 1 {
                // 对重叠峰组进行FBF处理
                let separated_peaks = self.fbf_separate_peaks(&group, curve, config)?;
                processed_peaks.extend(separated_peaks);
            } else {
                processed_peaks.extend(group);
            }
        }
        
        Ok(processed_peaks)
    }
    
    /// 识别重叠峰组
    fn identify_overlapping_groups(&self, peaks: &[Peak], curve: &Curve) -> Result<Vec<Vec<Peak>>, ProcessingError> {
        let mut groups = Vec::new();
        let mut used = vec![false; peaks.len()];
        
        for i in 0..peaks.len() {
            if used[i] {
                continue;
            }
            
            let mut group = vec![peaks[i].clone()];
            used[i] = true;
            
            // 寻找与当前峰重叠的其他峰
            for j in (i + 1)..peaks.len() {
                if used[j] {
                    continue;
                }
                
                if self.peaks_overlap(&peaks[i], &peaks[j], curve) {
                    group.push(peaks[j].clone());
                    used[j] = true;
                }
            }
            
            groups.push(group);
        }
        
        Ok(groups)
    }
    
    /// 检查两个峰是否重叠
    fn peaks_overlap(&self, peak1: &Peak, peak2: &Peak, _curve: &Curve) -> bool {
        // 计算峰间距离
        let distance = (peak1.center - peak2.center).abs();
        
        // 计算峰宽
        let width1 = peak1.fwhm.max(peak1.peak_span);
        let width2 = peak2.fwhm.max(peak2.peak_span);
        
        // 如果峰间距离小于峰宽之和的一半，则认为重叠
        distance < (width1 + width2) * 0.5
    }
    
    /// 使用FBF方法分离重叠峰
    fn fbf_separate_peaks(
        &self,
        overlapping_peaks: &[Peak],
        curve: &Curve,
        _config: &Value,
    ) -> Result<Vec<Peak>, ProcessingError> {
        // 提取重叠区域的数据
        let (x_data, y_data) = self.extract_overlapping_region(overlapping_peaks, curve);
        
        if x_data.len() < overlapping_peaks.len() * 3 {
            return Err(ProcessingError::process_error(
                "重叠区域数据点不足"
            ));
        }
        
        // 初始化贝叶斯参数
        let mut bayesian_params = self.initialize_bayesian_parameters(overlapping_peaks);
        
        // 执行FBF迭代
        for _iteration in 0..self.max_iterations {
            // E步骤：计算期望
            let expectations = self.expectation_step(&x_data, &y_data, &bayesian_params)?;
            
            // M步骤：最大化
            let new_params = self.maximization_step(&x_data, &y_data, &expectations, &bayesian_params)?;
            
            // 检查收敛
            if self.check_convergence(&bayesian_params, &new_params) {
                break;
            }
            
            bayesian_params = new_params;
        }
        
        // 从贝叶斯参数生成分离的峰
        self.generate_separated_peaks(&bayesian_params, overlapping_peaks)
    }
    
    /// 提取重叠区域的数据
    fn extract_overlapping_region(&self, peaks: &[Peak], curve: &Curve) -> (Vec<f64>, Vec<f64>) {
        let mut x_data = Vec::new();
        let mut y_data = Vec::new();
        
        // 计算重叠区域的范围
        let min_center = peaks.iter().map(|p| p.center).fold(f64::INFINITY, f64::min);
        let max_center = peaks.iter().map(|p| p.center).fold(f64::NEG_INFINITY, f64::max);
        let max_width = peaks.iter().map(|p| p.fwhm.max(p.peak_span)).fold(0.0, f64::max);
        
        let left_bound = min_center - max_width * 2.0;
        let right_bound = max_center + max_width * 2.0;
        
        for (i, &x) in curve.x_values.iter().enumerate() {
            if x >= left_bound && x <= right_bound {
                x_data.push(x);
                y_data.push(curve.y_values[i]);
            }
        }
        
        (x_data, y_data)
    }
    
    /// 初始化贝叶斯参数
    fn initialize_bayesian_parameters(&self, peaks: &[Peak]) -> BayesianParameters {
        let mut peak_params = Vec::new();
        
        for peak in peaks {
            peak_params.push(PeakBayesianParams {
                amplitude_mean: peak.amplitude,
                amplitude_var: peak.amplitude * 0.1,
                center_mean: peak.center,
                center_var: peak.fwhm * 0.1,
                sigma_mean: peak.sigma.max(0.1),
                sigma_var: peak.sigma * 0.1,
                weight: 1.0 / peaks.len() as f64,
            });
        }
        
        BayesianParameters {
            peak_params,
            noise_var: 1.0,
            regularization: self.regularization,
        }
    }
    
    /// E步骤：计算期望
    fn expectation_step(
        &self,
        x_data: &[f64],
        y_data: &[f64],
        params: &BayesianParameters,
    ) -> Result<Vec<Vec<f64>>, ProcessingError> {
        let mut expectations = Vec::new();
        
        for (i, &x) in x_data.iter().enumerate() {
            let mut point_expectations = Vec::new();
            let mut total_prob = 0.0;
            
            for peak_param in &params.peak_params {
                let prob = self.calculate_peak_probability(x, y_data[i], peak_param, params.noise_var);
                point_expectations.push(prob);
                total_prob += prob;
            }
            
            // 归一化
            if total_prob > 0.0 {
                for prob in &mut point_expectations {
                    *prob /= total_prob;
                }
            }
            
            expectations.push(point_expectations);
        }
        
        Ok(expectations)
    }
    
    /// M步骤：最大化
    fn maximization_step(
        &self,
        x_data: &[f64],
        y_data: &[f64],
        expectations: &[Vec<f64>],
        old_params: &BayesianParameters,
    ) -> Result<BayesianParameters, ProcessingError> {
        let mut new_peak_params = Vec::new();
        
        for (peak_idx, old_peak_param) in old_params.peak_params.iter().enumerate() {
            let mut amplitude_sum = 0.0;
            let mut center_sum = 0.0;
            let mut sigma_sum = 0.0;
            let mut weight_sum = 0.0;
            
            for (i, &x) in x_data.iter().enumerate() {
                let expectation = expectations[i][peak_idx];
                amplitude_sum += expectation * y_data[i];
                center_sum += expectation * x;
                sigma_sum += expectation * (x - old_peak_param.center_mean).powi(2);
                weight_sum += expectation;
            }
            
            if weight_sum > 0.0 {
                let new_amplitude = amplitude_sum / weight_sum;
                let new_center = center_sum / weight_sum;
                let new_sigma = (sigma_sum / weight_sum).sqrt().max(0.01);
                
                new_peak_params.push(PeakBayesianParams {
                    amplitude_mean: new_amplitude,
                    amplitude_var: old_peak_param.amplitude_var,
                    center_mean: new_center,
                    center_var: old_peak_param.center_var,
                    sigma_mean: new_sigma,
                    sigma_var: old_peak_param.sigma_var,
                    weight: weight_sum / x_data.len() as f64,
                });
            } else {
                new_peak_params.push(old_peak_param.clone());
            }
        }
        
        Ok(BayesianParameters {
            peak_params: new_peak_params,
            noise_var: old_params.noise_var,
            regularization: old_params.regularization,
        })
    }
    
    /// 计算峰概率
    fn calculate_peak_probability(&self, x: f64, y: f64, peak_param: &PeakBayesianParams, noise_var: f64) -> f64 {
        // 简化的高斯概率计算
        let predicted_y = self.gaussian_function(x, peak_param);
        let residual = y - predicted_y;
        let prob = (-residual.powi(2) / (2.0 * noise_var)).exp();
        prob * peak_param.weight
    }
    
    /// 高斯函数
    fn gaussian_function(&self, x: f64, peak_param: &PeakBayesianParams) -> f64 {
        let exponent = -((x - peak_param.center_mean).powi(2)) / (2.0 * peak_param.sigma_mean.powi(2));
        peak_param.amplitude_mean * exponent.exp()
    }
    
    /// 检查收敛
    fn check_convergence(&self, old_params: &BayesianParameters, new_params: &BayesianParameters) -> bool {
        for (old_peak, new_peak) in old_params.peak_params.iter().zip(new_params.peak_params.iter()) {
            let amplitude_diff = (old_peak.amplitude_mean - new_peak.amplitude_mean).abs();
            let center_diff = (old_peak.center_mean - new_peak.center_mean).abs();
            let sigma_diff = (old_peak.sigma_mean - new_peak.sigma_mean).abs();
            
            if amplitude_diff > self.convergence_threshold ||
               center_diff > self.convergence_threshold ||
               sigma_diff > self.convergence_threshold {
                return false;
            }
        }
        true
    }
    
    /// 从贝叶斯参数生成分离的峰
    fn generate_separated_peaks(
        &self,
        params: &BayesianParameters,
        original_peaks: &[Peak],
    ) -> Result<Vec<Peak>, ProcessingError> {
        let mut separated_peaks = Vec::new();
        
        for (i, peak_param) in params.peak_params.iter().enumerate() {
            if i < original_peaks.len() {
                let mut separated_peak = original_peaks[i].clone();
                separated_peak.amplitude = peak_param.amplitude_mean;
                separated_peak.center = peak_param.center_mean;
                separated_peak.sigma = peak_param.sigma_mean;
                separated_peak.fwhm = peak_param.sigma_mean * 2.355;
                separated_peak.hwhm = peak_param.sigma_mean * 1.177;
                
                // 添加FBF处理元数据
                separated_peak.add_metadata("fbf_processed".to_string(), serde_json::json!(true));
                separated_peak.add_metadata("fbf_weight".to_string(), serde_json::json!(peak_param.weight));
                separated_peak.add_metadata("fbf_amplitude_var".to_string(), serde_json::json!(peak_param.amplitude_var));
                separated_peak.add_metadata("fbf_center_var".to_string(), serde_json::json!(peak_param.center_var));
                separated_peak.add_metadata("fbf_sigma_var".to_string(), serde_json::json!(peak_param.sigma_var));
                
                separated_peaks.push(separated_peak);
            }
        }
        
        Ok(separated_peaks)
    }
}

impl OverlappingPeakProcessor for FBFPreprocessor {
    fn name(&self) -> &str {
        "fbf_preprocessor"
    }

    fn process_overlapping_peaks(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        config: &Value,
    ) -> Result<Vec<Peak>, ProcessingError> {
        self.preprocess_overlapping_peaks(peaks, curve, config)
    }
}

/// 贝叶斯参数
#[derive(Debug, Clone)]
struct BayesianParameters {
    peak_params: Vec<PeakBayesianParams>,
    noise_var: f64,
    regularization: f64,
}

/// 单个峰的贝叶斯参数
#[derive(Debug, Clone)]
struct PeakBayesianParams {
    amplitude_mean: f64,
    amplitude_var: f64,
    center_mean: f64,
    center_var: f64,
    sigma_mean: f64,
    sigma_var: f64,
    weight: f64,
}
