//! 复杂峰形算法
//! 
//! 针对特殊峰形的专门算法实现

use crate::core::data::ProcessingError;
use crate::core::processors::peak_fitting::peak_shapes::{PeakShapeType, PeakShapeParams};

/// 复杂峰形算法trait
pub trait AdvancedPeakAlgorithm {
    fn name(&self) -> &str;
    fn supported_shape_types(&self) -> Vec<PeakShapeType>;
    fn fit_peak(&self, x_data: &[f64], y_data: &[f64], initial_params: &PeakShapeParams) -> Result<PeakShapeParams, ProcessingError>;
    fn requires_special_initialization(&self) -> bool;
}

/// EMG (指数修正高斯) 专门算法
pub struct EMGAlgorithm;

impl AdvancedPeakAlgorithm for EMGAlgorithm {
    fn name(&self) -> &str {
        "emg_algorithm"
    }
    
    fn supported_shape_types(&self) -> Vec<PeakShapeType> {
        vec![PeakShapeType::ExponentiallyModifiedGaussian]
    }
    
    fn fit_peak(&self, x_data: &[f64], y_data: &[f64], initial_params: &PeakShapeParams) -> Result<PeakShapeParams, ProcessingError> {
        let mut params = initial_params.clone();
        
        // EMG特殊初始化
        if self.requires_special_initialization() {
            self.initialize_emg_parameters(&mut params, x_data, y_data);
        }
        
        // 使用EMG特定的优化算法
        self.emg_optimization(x_data, y_data, &mut params)?;
        
        Ok(params)
    }
    
    fn requires_special_initialization(&self) -> bool {
        true
    }
}

impl EMGAlgorithm {
    /// EMG参数初始化
    fn initialize_emg_parameters(&self, params: &mut PeakShapeParams, x_data: &[f64], y_data: &[f64]) {
        // 找到峰中心
        let max_idx = y_data.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap().0;
        
        let center = x_data[max_idx];
        let amplitude = y_data[max_idx];
        
        // 估计sigma（高斯部分）
        let mut sigma_sum = 0.0;
        let mut sigma_count = 0;
        for (i, &y) in y_data.iter().enumerate() {
            if y > amplitude / 2.0 {
                let dx = (x_data[i] - center).abs();
                sigma_sum += dx;
                sigma_count += 1;
            }
        }
        let sigma = if sigma_count > 0 { sigma_sum / sigma_count as f64 } else { 1.0 };
        
        // 估计tau（指数衰减参数）
        let tau = self.estimate_tau_parameter(x_data, y_data, center, amplitude);
        
        // 设置参数
        if let Some(amp_idx) = params.parameter_names.iter().position(|n| n == "amplitude") {
            params.parameters[amp_idx] = amplitude;
        }
        if let Some(center_idx) = params.parameter_names.iter().position(|n| n == "center") {
            params.parameters[center_idx] = center;
        }
        if let Some(sigma_idx) = params.parameter_names.iter().position(|n| n == "sigma") {
            params.parameters[sigma_idx] = sigma;
        }
        if let Some(tau_idx) = params.parameter_names.iter().position(|n| n == "tau") {
            params.parameters[tau_idx] = tau;
        }
    }
    
    /// 估计tau参数
    fn estimate_tau_parameter(&self, x_data: &[f64], y_data: &[f64], center: f64, amplitude: f64) -> f64 {
        // 分析峰右侧的拖尾衰减
        let mut tau_estimates = Vec::new();
        
        for (i, &x) in x_data.iter().enumerate() {
            if x > center && y_data[i] > amplitude * 0.1 {
                let distance = x - center;
                let intensity_ratio = y_data[i] / amplitude;
                
                // 使用指数衰减模型估计tau
                if intensity_ratio > 0.0 {
                    let tau_est = -distance / intensity_ratio.ln();
                    if tau_est > 0.0 && tau_est < 10.0 {
                        tau_estimates.push(tau_est);
                    }
                }
            }
        }
        
        if tau_estimates.is_empty() {
            0.5 // 默认值
        } else {
            // 使用中位数作为tau的估计
            tau_estimates.sort_by(|a, b| a.partial_cmp(b).unwrap());
            tau_estimates[tau_estimates.len() / 2]
        }
    }
    
    /// EMG优化算法
    fn emg_optimization(&self, x_data: &[f64], y_data: &[f64], params: &mut PeakShapeParams) -> Result<(), ProcessingError> {
        let max_iterations = 50;
        let learning_rate = 0.01;
        let convergence_threshold = 1e-6;
        
        let mut previous_error = f64::INFINITY;
        
        for _iteration in 0..max_iterations {
            // 计算梯度
            let gradient = self.compute_emg_gradient(x_data, y_data, params);
            
            // 更新参数
            for (i, param) in params.parameters.iter_mut().enumerate() {
                *param -= learning_rate * gradient[i];
            }
            
            // 应用边界约束
            params.clamp_parameters();
            
            // 计算误差
            let current_error = self.calculate_emg_error(x_data, y_data, params);
            
            // 检查收敛
            if (previous_error - current_error).abs() < convergence_threshold {
                break;
            }
            
            previous_error = current_error;
        }
        
        Ok(())
    }
    
    /// 计算EMG梯度
    fn compute_emg_gradient(&self, x_data: &[f64], y_data: &[f64], params: &PeakShapeParams) -> Vec<f64> {
        let h = 1e-6;
        let mut gradient = Vec::new();
        
        for i in 0..params.parameters.len() {
            let mut params_plus = params.clone();
            let mut params_minus = params.clone();
            
            params_plus.parameters[i] += h;
            params_minus.parameters[i] -= h;
            
            let f_plus = self.calculate_emg_error(x_data, y_data, &params_plus);
            let f_minus = self.calculate_emg_error(x_data, y_data, &params_minus);
            
            gradient.push((f_plus - f_minus) / (2.0 * h));
        }
        
        gradient
    }
    
    /// 计算EMG误差
    fn calculate_emg_error(&self, x_data: &[f64], y_data: &[f64], params: &PeakShapeParams) -> f64 {
        let mut error = 0.0;
        
        for (i, &x) in x_data.iter().enumerate() {
            let predicted = self.emg_function(x, params);
            error += (y_data[i] - predicted).powi(2);
        }
        
        error
    }
    
    /// EMG函数
    fn emg_function(&self, x: f64, params: &PeakShapeParams) -> f64 {
        let amplitude = params.get_parameter("amplitude").unwrap_or(0.0);
        let center = params.get_parameter("center").unwrap_or(0.0);
        let sigma = params.get_parameter("sigma").unwrap_or(1.0);
        let tau = params.get_parameter("tau").unwrap_or(1.0);
        
        // EMG函数实现
        let z = (x - center) / sigma - sigma / tau;
        let erfc_term = 1.0 - self.erfc(-z / 2.0_f64.sqrt());
        let exp_term = ((x - center) / tau + sigma.powi(2) / (2.0 * tau.powi(2))).exp();
        
        amplitude * erfc_term * exp_term / 2.0
    }
    
    /// 互补误差函数近似
    fn erfc(&self, x: f64) -> f64 {
        // 使用Abramowitz和Stegun的近似
        let a1 = -1.26551223;
        let a2 = 1.00002368;
        let a3 = 0.37409196;
        let a4 = 0.09678418;
        let a5 = -0.18628806;
        let a6 = 0.27886807;
        let a7 = -1.13520398;
        let a8 = 1.48851587;
        let a9 = -0.82215223;
        let a10 = 0.17087277;
        
        let t = 1.0 / (1.0 + 0.5 * x.abs());
        let erf_approx = 1.0 - t * (a1 + t * (a2 + t * (a3 + t * (a4 + t * (a5 + t * (a6 + t * (a7 + t * (a8 + t * (a9 + t * a10))))))))) * (-x.powi(2)).exp();
        
        if x >= 0.0 {
            1.0 - erf_approx
        } else {
            1.0 + erf_approx
        }
    }
}

/// 双高斯峰专门算法
pub struct BiGaussianAlgorithm;

impl AdvancedPeakAlgorithm for BiGaussianAlgorithm {
    fn name(&self) -> &str {
        "bi_gaussian_algorithm"
    }
    
    fn supported_shape_types(&self) -> Vec<PeakShapeType> {
        vec![PeakShapeType::BiGaussian]
    }
    
    fn fit_peak(&self, x_data: &[f64], y_data: &[f64], initial_params: &PeakShapeParams) -> Result<PeakShapeParams, ProcessingError> {
        let mut params = initial_params.clone();
        
        // 双高斯特殊初始化
        if self.requires_special_initialization() {
            self.initialize_bi_gaussian_parameters(&mut params, x_data, y_data);
        }
        
        // 使用双高斯特定的优化算法
        self.bi_gaussian_optimization(x_data, y_data, &mut params)?;
        
        Ok(params)
    }
    
    fn requires_special_initialization(&self) -> bool {
        true
    }
}

impl BiGaussianAlgorithm {
    /// 双高斯参数初始化
    fn initialize_bi_gaussian_parameters(&self, params: &mut PeakShapeParams, x_data: &[f64], y_data: &[f64]) {
        // 找到峰中心
        let max_idx = y_data.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap().0;
        
        let center = x_data[max_idx];
        let amplitude = y_data[max_idx];
        
        // 分析峰的不对称性
        let asymmetry = self.calculate_asymmetry(x_data, y_data, center, amplitude);
        
        // 估计左右sigma
        let (sigma_left, sigma_right) = self.estimate_left_right_sigma(x_data, y_data, center, amplitude);
        
        // 设置参数
        if let Some(amp_idx) = params.parameter_names.iter().position(|n| n == "amplitude") {
            params.parameters[amp_idx] = amplitude;
        }
        if let Some(center_idx) = params.parameter_names.iter().position(|n| n == "center") {
            params.parameters[center_idx] = center;
        }
        if let Some(sigma_left_idx) = params.parameter_names.iter().position(|n| n == "sigma_left") {
            params.parameters[sigma_left_idx] = sigma_left;
        }
        if let Some(sigma_right_idx) = params.parameter_names.iter().position(|n| n == "sigma_right") {
            params.parameters[sigma_right_idx] = sigma_right;
        }
        if let Some(asymmetry_idx) = params.parameter_names.iter().position(|n| n == "asymmetry") {
            params.parameters[asymmetry_idx] = asymmetry;
        }
    }
    
    /// 计算不对称性
    fn calculate_asymmetry(&self, x_data: &[f64], y_data: &[f64], center: f64, amplitude: f64) -> f64 {
        let half_height = amplitude / 2.0;
        
        // 找到左半高宽
        let mut left_hwhm = 0.0;
        for (i, &x) in x_data.iter().enumerate() {
            if x < center && y_data[i] <= half_height {
                left_hwhm = center - x;
                break;
            }
        }
        
        // 找到右半高宽
        let mut right_hwhm = 0.0;
        for (i, &x) in x_data.iter().enumerate() {
            if x > center && y_data[i] <= half_height {
                right_hwhm = x - center;
                break;
            }
        }
        
        if left_hwhm > 0.0 && right_hwhm > 0.0 {
            (right_hwhm - left_hwhm) / (left_hwhm + right_hwhm)
        } else {
            0.0
        }
    }
    
    /// 估计左右sigma
    fn estimate_left_right_sigma(&self, x_data: &[f64], y_data: &[f64], center: f64, amplitude: f64) -> (f64, f64) {
        let mut left_sigma_sum = 0.0;
        let mut left_count = 0;
        let mut right_sigma_sum = 0.0;
        let mut right_count = 0;
        
        for (i, &x) in x_data.iter().enumerate() {
            if y_data[i] > amplitude * 0.1 {
                let distance = (x - center).abs();
                if x < center {
                    left_sigma_sum += distance;
                    left_count += 1;
                } else if x > center {
                    right_sigma_sum += distance;
                    right_count += 1;
                }
            }
        }
        
        let sigma_left = if left_count > 0 { left_sigma_sum / left_count as f64 } else { 1.0 };
        let sigma_right = if right_count > 0 { right_sigma_sum / right_count as f64 } else { 1.0 };
        
        (sigma_left, sigma_right)
    }
    
    /// 双高斯优化算法
    fn bi_gaussian_optimization(&self, x_data: &[f64], y_data: &[f64], params: &mut PeakShapeParams) -> Result<(), ProcessingError> {
        let max_iterations = 50;
        let learning_rate = 0.01;
        let convergence_threshold = 1e-6;
        
        let mut previous_error = f64::INFINITY;
        
        for _iteration in 0..max_iterations {
            // 计算梯度
            let gradient = self.compute_bi_gaussian_gradient(x_data, y_data, params);
            
            // 更新参数
            for (i, param) in params.parameters.iter_mut().enumerate() {
                *param -= learning_rate * gradient[i];
            }
            
            // 应用边界约束
            params.clamp_parameters();
            
            // 计算误差
            let current_error = self.calculate_bi_gaussian_error(x_data, y_data, params);
            
            // 检查收敛
            if (previous_error - current_error).abs() < convergence_threshold {
                break;
            }
            
            previous_error = current_error;
        }
        
        Ok(())
    }
    
    /// 计算双高斯梯度
    fn compute_bi_gaussian_gradient(&self, x_data: &[f64], y_data: &[f64], params: &PeakShapeParams) -> Vec<f64> {
        let h = 1e-6;
        let mut gradient = Vec::new();
        
        for i in 0..params.parameters.len() {
            let mut params_plus = params.clone();
            let mut params_minus = params.clone();
            
            params_plus.parameters[i] += h;
            params_minus.parameters[i] -= h;
            
            let f_plus = self.calculate_bi_gaussian_error(x_data, y_data, &params_plus);
            let f_minus = self.calculate_bi_gaussian_error(x_data, y_data, &params_minus);
            
            gradient.push((f_plus - f_minus) / (2.0 * h));
        }
        
        gradient
    }
    
    /// 计算双高斯误差
    fn calculate_bi_gaussian_error(&self, x_data: &[f64], y_data: &[f64], params: &PeakShapeParams) -> f64 {
        let mut error = 0.0;
        
        for (i, &x) in x_data.iter().enumerate() {
            let predicted = self.bi_gaussian_function(x, params);
            error += (y_data[i] - predicted).powi(2);
        }
        
        error
    }
    
    /// 双高斯函数
    fn bi_gaussian_function(&self, x: f64, params: &PeakShapeParams) -> f64 {
        let amplitude = params.get_parameter("amplitude").unwrap_or(0.0);
        let center = params.get_parameter("center").unwrap_or(0.0);
        let sigma_left = params.get_parameter("sigma_left").unwrap_or(1.0);
        let sigma_right = params.get_parameter("sigma_right").unwrap_or(1.0);
        let asymmetry = params.get_parameter("asymmetry").unwrap_or(0.0);
        
        let sigma = if x < center { sigma_left } else { sigma_right };
        let exponent = -((x - center).powi(2)) / (2.0 * sigma.powi(2));
        
        amplitude * exponent.exp() * (1.0 + asymmetry * (x - center) / center.abs().max(1.0))
    }
}

/// 复杂峰形算法工厂
pub struct AdvancedAlgorithmFactory;

impl AdvancedAlgorithmFactory {
    pub fn create_algorithm(shape_type: &PeakShapeType) -> Option<Box<dyn AdvancedPeakAlgorithm>> {
        match shape_type {
            PeakShapeType::ExponentiallyModifiedGaussian => Some(Box::new(EMGAlgorithm)),
            PeakShapeType::BiGaussian => Some(Box::new(BiGaussianAlgorithm)),
            _ => None,
        }
    }
    
    pub fn get_available_algorithms() -> Vec<Box<dyn AdvancedPeakAlgorithm>> {
        vec![
            Box::new(EMGAlgorithm),
            Box::new(BiGaussianAlgorithm),
        ]
    }
}
