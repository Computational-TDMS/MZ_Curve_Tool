//! 多峰拟合器
//! 
//! 实现多峰同时拟合和峰拆分算法

use crate::core::data::{Curve, Peak, ProcessingError, PeakType};
use crate::core::processors::peak_fitting::PeakFitter;
use crate::core::processors::peak_fitting::peak_shapes::{PeakShapeType, PeakShapeParams, PeakShapeAnalyzer, PeakShapeCalculatorFactory};
use crate::core::processors::peak_fitting::parameter_optimizer::{ParameterOptimizer, OptimizationAlgorithm};
use serde_json::Value;

/// 多峰拟合器
#[derive(Debug)]
pub struct MultiPeakFitter {
    peak_analyzer: PeakShapeAnalyzer,
    optimizer: ParameterOptimizer,
}

impl MultiPeakFitter {
    pub fn new() -> Self {
        Self {
            peak_analyzer: PeakShapeAnalyzer,
            optimizer: ParameterOptimizer::new(OptimizationAlgorithm::LevenbergMarquardt {
                max_iterations: 100,
                convergence_threshold: 1e-6,
                damping_factor: 0.1,
            }),
        }
    }
    
    /// 创建带自定义优化算法的多峰拟合器
    pub fn with_optimizer(algorithm: OptimizationAlgorithm) -> Self {
        Self {
            peak_analyzer: PeakShapeAnalyzer,
            optimizer: ParameterOptimizer::new(algorithm),
        }
    }
}

impl PeakFitter for MultiPeakFitter {
    fn name(&self) -> &str {
        "multi_peak_fitter"
    }

    fn fit_peak(&self, peak: &Peak, curve: &Curve, config: &Value) -> Result<Peak, ProcessingError> {
        // 多峰拟合需要分析整个峰区域，而不仅仅是单个峰
        let window_size = config["fit_window_size"].as_f64().unwrap_or(3.0);
        let (x_data, y_data) = self.extract_fit_data(curve, peak.center, window_size);
        
        if x_data.len() < 10 {
            return Ok(peak.clone());
        }
        
        // 检测峰区域内的所有峰
        let detected_peaks = self.detect_peaks_in_region(&x_data, &y_data, config)?;
        
        if detected_peaks.len() <= 1 {
            // 单峰情况，使用单峰拟合
            self.fit_single_peak(peak, &x_data, &y_data, config)
        } else {
            // 多峰情况，使用多峰拟合
            self.fit_multiple_peaks(&detected_peaks, &x_data, &y_data, config)
                .and_then(|fitted_peaks| {
                    // 找到与输入峰最接近的拟合峰
                    self.find_closest_peak(peak, &fitted_peaks)
                })
        }
    }
}

impl MultiPeakFitter {
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
    
    /// 在区域内检测峰
    fn detect_peaks_in_region(
        &self,
        x_data: &[f64],
        y_data: &[f64],
        config: &Value,
    ) -> Result<Vec<PeakCandidate>, ProcessingError> {
        let mut peaks = Vec::new();
        let threshold = config["peak_threshold"].as_f64().unwrap_or(0.1);
        let min_distance = config["min_peak_distance"].as_f64().unwrap_or(0.5);
        
        // 简单的峰检测算法
        for i in 1..(y_data.len() - 1) {
            if y_data[i] > y_data[i-1] && y_data[i] > y_data[i+1] && y_data[i] > threshold {
                // 检查与已有峰的距离
                let current_x = x_data[i];
                let too_close = peaks.iter().any(|peak: &PeakCandidate| (peak.center - current_x).abs() < min_distance);
                
                if !too_close {
                    peaks.push(PeakCandidate {
                        center: current_x,
                        amplitude: y_data[i],
                        width: self.estimate_peak_width(x_data, y_data, i),
                        shape_type: PeakShapeType::Gaussian, // 默认形状
                    });
                }
            }
        }
        
        // 按振幅排序
        peaks.sort_by(|a, b| b.amplitude.partial_cmp(&a.amplitude).unwrap());
        
        Ok(peaks)
    }
    
    /// 估计峰宽
    fn estimate_peak_width(&self, x_data: &[f64], y_data: &[f64], peak_index: usize) -> f64 {
        let peak_height = y_data[peak_index];
        let half_height = peak_height / 2.0;
        
        // 寻找左半高宽
        let mut left_width = 0.0;
        for i in (0..peak_index).rev() {
            if y_data[i] <= half_height {
                left_width = x_data[peak_index] - x_data[i];
                break;
            }
        }
        
        // 寻找右半高宽
        let mut right_width = 0.0;
        for i in (peak_index + 1)..y_data.len() {
            if y_data[i] <= half_height {
                right_width = x_data[i] - x_data[peak_index];
                break;
            }
        }
        
        (left_width + right_width) / 2.0
    }
    
    /// 拟合单个峰
    fn fit_single_peak(
        &self,
        peak: &Peak,
        x_data: &[f64],
        y_data: &[f64],
        _config: &Value,
    ) -> Result<Peak, ProcessingError> {
        // 分析峰形
        let shape_type = self.peak_analyzer.analyze_peak_shape(x_data, y_data);
        
        // 创建峰形参数
        let mut params = PeakShapeParams::new(shape_type);
        self.initialize_parameters(&mut params, x_data, y_data, peak);
        
        // 定义目标函数
        let objective_function = |x: &[f64], y: &[f64], p: &PeakShapeParams| -> f64 {
            self.calculate_fit_error(x, y, p)
        };
        
        // 执行优化
        let result = self.optimizer.optimize(objective_function, params, x_data, y_data)?;
        
        // 创建拟合后的峰
        self.create_fitted_peak(peak, &result.optimized_params, &result, x_data, y_data)
    }
    
    /// 拟合多个峰
    fn fit_multiple_peaks(
        &self,
        peak_candidates: &[PeakCandidate],
        x_data: &[f64],
        y_data: &[f64],
        _config: &Value,
    ) -> Result<Vec<Peak>, ProcessingError> {
        let mut fitted_peaks = Vec::new();
        
        // 为每个峰候选创建峰形参数
        let mut all_params = Vec::new();
        for candidate in peak_candidates {
            let shape_type = self.peak_analyzer.analyze_peak_shape(x_data, y_data);
            let mut params = PeakShapeParams::new(shape_type);
            self.initialize_parameters_for_candidate(&mut params, x_data, y_data, candidate);
            all_params.push(params);
        }
        
        // 多峰联合优化
        let result = self.optimize_multiple_peaks(&all_params, x_data, y_data)?;
        
        // 创建拟合后的峰
        for (i, optimized_params) in result.optimized_params.iter().enumerate() {
            if i < peak_candidates.len() {
                let candidate = &peak_candidates[i];
                let peak = self.create_peak_from_candidate(candidate, optimized_params, x_data, y_data);
                fitted_peaks.push(peak);
            }
        }
        
        Ok(fitted_peaks)
    }
    
    /// 多峰联合优化
    fn optimize_multiple_peaks(
        &self,
        initial_params: &[PeakShapeParams],
        x_data: &[f64],
        y_data: &[f64],
    ) -> Result<MultiPeakOptimizationResult, ProcessingError> {
        // 合并所有参数
        let mut combined_params = PeakShapeParams::new(PeakShapeType::Gaussian);
        combined_params.parameters.clear();
        combined_params.parameter_names.clear();
        combined_params.bounds.clear();
        
        for params in initial_params {
            combined_params.parameters.extend(params.parameters.clone());
            combined_params.parameter_names.extend(params.parameter_names.clone());
            combined_params.bounds.extend(params.bounds.clone());
        }
        
        // 定义多峰目标函数
        let objective_function = |x: &[f64], y: &[f64], p: &PeakShapeParams| -> f64 {
            self.calculate_multi_peak_fit_error(x, y, p, initial_params.len())
        };
        
        // 执行优化
        let result = self.optimizer.optimize(objective_function, combined_params, x_data, y_data)?;
        
        // 分离参数
        let mut separated_params = Vec::new();
        let mut param_index = 0;
        
        for params in initial_params {
            let param_count = params.parameters.len();
            let mut separated = params.clone();
            
            for i in 0..param_count {
                if param_index < result.optimized_params.parameters.len() {
                    separated.parameters[i] = result.optimized_params.parameters[param_index];
                    param_index += 1;
                }
            }
            
            separated_params.push(separated);
        }
        
        Ok(MultiPeakOptimizationResult {
            optimized_params: separated_params,
            final_error: result.final_error,
            iterations: result.iterations,
            converged: result.converged,
        })
    }
    
    /// 计算多峰拟合误差
    fn calculate_multi_peak_fit_error(
        &self,
        x_data: &[f64],
        y_data: &[f64],
        combined_params: &PeakShapeParams,
        peak_count: usize,
    ) -> f64 {
        let mut error = 0.0;
        
        for (i, &x) in x_data.iter().enumerate() {
            let mut predicted = 0.0;
            let mut param_index = 0;
            
            // 计算所有峰的贡献
            for _ in 0..peak_count {
                let mut peak_params = PeakShapeParams::new(PeakShapeType::Gaussian);
                let param_count = 3; // 假设每个峰有3个参数
                
                for j in 0..param_count {
                    if param_index < combined_params.parameters.len() {
                        peak_params.parameters[j] = combined_params.parameters[param_index];
                        param_index += 1;
                    }
                }
                
                predicted += self.predict_single_peak_value(x, &peak_params);
            }
            
            error += (y_data[i] - predicted).powi(2);
        }
        
        error
    }
    
    /// 预测单个峰的值
    fn predict_single_peak_value(&self, x: f64, params: &PeakShapeParams) -> f64 {
        let calculator = PeakShapeCalculatorFactory::create_calculator(&params.shape_type);
        calculator.calculate(x, params)
    }
    
    /// 计算拟合误差
    fn calculate_fit_error(&self, x_data: &[f64], y_data: &[f64], params: &PeakShapeParams) -> f64 {
        let mut error = 0.0;
        let calculator = PeakShapeCalculatorFactory::create_calculator(&params.shape_type);
        
        for (i, &x) in x_data.iter().enumerate() {
            let predicted = calculator.calculate(x, params);
            error += (y_data[i] - predicted).powi(2);
        }
        
        error
    }
    
    /// 初始化参数
    fn initialize_parameters(&self, params: &mut PeakShapeParams, _x_data: &[f64], _y_data: &[f64], peak: &Peak) {
        if let Some(amplitude) = params.parameter_names.iter().position(|n| n == "amplitude") {
            params.parameters[amplitude] = peak.amplitude;
        }
        
        if let Some(center) = params.parameter_names.iter().position(|n| n == "center") {
            params.parameters[center] = peak.center;
        }
        
        if let Some(sigma) = params.parameter_names.iter().position(|n| n == "sigma") {
            params.parameters[sigma] = peak.sigma.max(0.1);
        }
        
        if let Some(gamma) = params.parameter_names.iter().position(|n| n == "gamma") {
            params.parameters[gamma] = peak.fwhm / 2.0;
        }
    }
    
    /// 为峰候选初始化参数
    fn initialize_parameters_for_candidate(&self, params: &mut PeakShapeParams, _x_data: &[f64], _y_data: &[f64], candidate: &PeakCandidate) {
        if let Some(amplitude) = params.parameter_names.iter().position(|n| n == "amplitude") {
            params.parameters[amplitude] = candidate.amplitude;
        }
        
        if let Some(center) = params.parameter_names.iter().position(|n| n == "center") {
            params.parameters[center] = candidate.center;
        }
        
        if let Some(sigma) = params.parameter_names.iter().position(|n| n == "sigma") {
            params.parameters[sigma] = candidate.width / 2.355; // 转换为sigma
        }
        
        if let Some(gamma) = params.parameter_names.iter().position(|n| n == "gamma") {
            params.parameters[gamma] = candidate.width / 2.0;
        }
    }
    
    /// 创建拟合后的峰
    fn create_fitted_peak(
        &self,
        original_peak: &Peak,
        params: &PeakShapeParams,
        result: &crate::core::processors::peak_fitting::parameter_optimizer::OptimizationResult,
        x_data: &[f64],
        y_data: &[f64],
    ) -> Result<Peak, ProcessingError> {
        let mut fitted_peak = original_peak.clone();
        
        // 更新峰参数
        if let Some(amplitude) = params.get_parameter("amplitude") {
            fitted_peak.amplitude = amplitude;
        }
        
        if let Some(center) = params.get_parameter("center") {
            fitted_peak.center = center;
        }
        
        if let Some(sigma) = params.get_parameter("sigma") {
            fitted_peak.sigma = sigma;
            fitted_peak.fwhm = sigma * 2.355;
            fitted_peak.hwhm = sigma * 1.177;
        }
        
        if let Some(gamma) = params.get_parameter("gamma") {
            fitted_peak.gamma = gamma;
            fitted_peak.fwhm = 2.0 * gamma;
            fitted_peak.hwhm = gamma;
        }
        
        // 设置峰类型
        fitted_peak.peak_type = match params.shape_type {
            PeakShapeType::Gaussian => PeakType::Gaussian,
            PeakShapeType::Lorentzian => PeakType::Lorentzian,
            PeakShapeType::PseudoVoigt => PeakType::PseudoVoigt,
            PeakShapeType::ExponentiallyModifiedGaussian => PeakType::EMG,
            PeakShapeType::BiGaussian => PeakType::BiGaussian,
            _ => PeakType::Gaussian,
        };
        
        // 设置拟合参数
        fitted_peak.set_fit_parameters(params.parameters.clone(), result.parameter_errors.clone(), None);
        
        // 计算峰面积
        fitted_peak.calculate_area_from_fit();
        
        // 计算拟合质量
        let rsquared = self.calculate_rsquared(x_data, y_data, params);
        fitted_peak.rsquared = rsquared;
        fitted_peak.standard_error = result.final_error.sqrt();
        
        // 添加多峰拟合元数据
        fitted_peak.add_metadata("multi_peak_fitting".to_string(), Value::Bool(true));
        fitted_peak.add_metadata("fitting_method".to_string(), Value::String("multi_peak".to_string()));
        fitted_peak.add_metadata("shape_type".to_string(), Value::String(format!("{:?}", params.shape_type)));
        fitted_peak.add_metadata("iterations".to_string(), Value::Number(serde_json::Number::from(result.iterations)));
        fitted_peak.add_metadata("converged".to_string(), Value::Bool(result.converged));
        
        Ok(fitted_peak)
    }
    
    /// 从峰候选创建峰
    fn create_peak_from_candidate(
        &self,
        candidate: &PeakCandidate,
        params: &PeakShapeParams,
        x_data: &[f64],
        y_data: &[f64],
    ) -> Peak {
        let mut peak = Peak::new(
            format!("peak_{}", candidate.center),
            "unknown".to_string(),
            candidate.center,
            candidate.amplitude,
            PeakType::Gaussian,
        );
        
        // 设置基本参数
        if let Some(amplitude) = params.get_parameter("amplitude") {
            peak.amplitude = amplitude;
        }
        
        if let Some(center) = params.get_parameter("center") {
            peak.center = center;
        }
        
        if let Some(sigma) = params.get_parameter("sigma") {
            peak.sigma = sigma;
            peak.fwhm = sigma * 2.355;
            peak.hwhm = sigma * 1.177;
        }
        
        if let Some(gamma) = params.get_parameter("gamma") {
            peak.gamma = gamma;
            peak.fwhm = 2.0 * gamma;
            peak.hwhm = gamma;
        }
        
        // 设置峰类型
        peak.peak_type = match params.shape_type {
            PeakShapeType::Gaussian => PeakType::Gaussian,
            PeakShapeType::Lorentzian => PeakType::Lorentzian,
            PeakShapeType::PseudoVoigt => PeakType::PseudoVoigt,
            PeakShapeType::ExponentiallyModifiedGaussian => PeakType::EMG,
            PeakShapeType::BiGaussian => PeakType::BiGaussian,
            _ => PeakType::Gaussian,
        };
        
        // 计算峰面积
        peak.calculate_area_from_fit();
        
        // 计算拟合质量
        let rsquared = self.calculate_rsquared(x_data, y_data, params);
        peak.rsquared = rsquared;
        
        // 添加元数据
        peak.add_metadata("multi_peak_fitting".to_string(), Value::Bool(true));
        peak.add_metadata("shape_type".to_string(), Value::String(format!("{:?}", params.shape_type)));
        
        peak
    }
    
    /// 计算R²
    fn calculate_rsquared(&self, x_data: &[f64], y_data: &[f64], params: &PeakShapeParams) -> f64 {
        let y_mean: f64 = y_data.iter().sum::<f64>() / y_data.len() as f64;
        let mut ss_tot = 0.0;
        let mut ss_res = 0.0;
        
        let calculator = PeakShapeCalculatorFactory::create_calculator(&params.shape_type);
        
        for (i, &y) in y_data.iter().enumerate() {
            let y_fit = calculator.calculate(x_data[i], params);
            ss_tot += (y - y_mean).powi(2);
            ss_res += (y - y_fit).powi(2);
        }
        
        if ss_tot == 0.0 {
            0.0
        } else {
            1.0 - (ss_res / ss_tot)
        }
    }
    
    /// 找到最接近的峰
    fn find_closest_peak(&self, target_peak: &Peak, fitted_peaks: &[Peak]) -> Result<Peak, ProcessingError> {
        if fitted_peaks.is_empty() {
            return Ok(target_peak.clone());
        }
        
        let mut closest_peak = &fitted_peaks[0];
        let mut min_distance = (target_peak.center - closest_peak.center).abs();
        
        for peak in fitted_peaks.iter().skip(1) {
            let distance = (target_peak.center - peak.center).abs();
            if distance < min_distance {
                min_distance = distance;
                closest_peak = peak;
            }
        }
        
        Ok(closest_peak.clone())
    }
}

/// 峰候选
#[derive(Debug, Clone)]
struct PeakCandidate {
    center: f64,
    amplitude: f64,
    width: f64,
    shape_type: PeakShapeType,
}

/// 多峰优化结果
#[derive(Debug)]
struct MultiPeakOptimizationResult {
    optimized_params: Vec<PeakShapeParams>,
    final_error: f64,
    iterations: usize,
    converged: bool,
}