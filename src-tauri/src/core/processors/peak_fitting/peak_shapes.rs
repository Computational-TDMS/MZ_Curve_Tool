//! 峰形定义器
//! 
//! 定义各种基础峰形和复杂峰形


/// 峰形类型
#[derive(Debug, Clone, PartialEq)]
pub enum PeakShapeType {
    /// 高斯峰
    Gaussian,
    /// 洛伦兹峰
    Lorentzian,
    /// 伪Voigt峰（高斯和洛伦兹的混合）
    PseudoVoigt,
    /// 指数修正高斯峰（EMG）
    ExponentiallyModifiedGaussian,
    /// 双高斯峰
    BiGaussian,
    /// 不对称峰
    Asymmetric,
}

/// 峰形参数
#[derive(Debug, Clone)]
pub struct PeakShapeParams {
    pub shape_type: PeakShapeType,
    pub parameters: Vec<f64>,
    pub parameter_names: Vec<String>,
    pub bounds: Vec<(f64, f64)>, // (min, max) 边界
}

impl PeakShapeParams {
    pub fn new(shape_type: PeakShapeType) -> Self {
        match shape_type {
            PeakShapeType::Gaussian => Self {
                shape_type,
                parameters: vec![0.0; 3], // amplitude, center, sigma
                parameter_names: vec!["amplitude".to_string(), "center".to_string(), "sigma".to_string()],
                bounds: vec![(0.0, f64::INFINITY), (f64::NEG_INFINITY, f64::INFINITY), (0.01, 10.0)],
            },
            PeakShapeType::Lorentzian => Self {
                shape_type,
                parameters: vec![0.0; 3], // amplitude, center, gamma
                parameter_names: vec!["amplitude".to_string(), "center".to_string(), "gamma".to_string()],
                bounds: vec![(0.0, f64::INFINITY), (f64::NEG_INFINITY, f64::INFINITY), (0.01, 10.0)],
            },
            PeakShapeType::PseudoVoigt => Self {
                shape_type,
                parameters: vec![0.0; 4], // amplitude, center, sigma, mixing
                parameter_names: vec!["amplitude".to_string(), "center".to_string(), "sigma".to_string(), "mixing".to_string()],
                bounds: vec![(0.0, f64::INFINITY), (f64::NEG_INFINITY, f64::INFINITY), (0.01, 10.0), (0.0, 1.0)],
            },
            PeakShapeType::ExponentiallyModifiedGaussian => Self {
                shape_type,
                parameters: vec![0.0; 4], // amplitude, center, sigma, tau
                parameter_names: vec!["amplitude".to_string(), "center".to_string(), "sigma".to_string(), "tau".to_string()],
                bounds: vec![(0.0, f64::INFINITY), (f64::NEG_INFINITY, f64::INFINITY), (0.01, 10.0), (0.01, 5.0)],
            },
            PeakShapeType::BiGaussian => Self {
                shape_type,
                parameters: vec![0.0; 5], // amplitude, center, sigma_left, sigma_right, asymmetry
                parameter_names: vec!["amplitude".to_string(), "center".to_string(), "sigma_left".to_string(), "sigma_right".to_string(), "asymmetry".to_string()],
                bounds: vec![(0.0, f64::INFINITY), (f64::NEG_INFINITY, f64::INFINITY), (0.01, 10.0), (0.01, 10.0), (0.0, 1.0)],
            },
            PeakShapeType::Asymmetric => Self {
                shape_type,
                parameters: vec![0.0; 6], // amplitude, center, sigma, asymmetry, tail_left, tail_right
                parameter_names: vec!["amplitude".to_string(), "center".to_string(), "sigma".to_string(), "asymmetry".to_string(), "tail_left".to_string(), "tail_right".to_string()],
                bounds: vec![(0.0, f64::INFINITY), (f64::NEG_INFINITY, f64::INFINITY), (0.01, 10.0), (0.0, 1.0), (0.0, 2.0), (0.0, 2.0)],
            },
        }
    }
    
    pub fn set_parameter(&mut self, name: &str, value: f64) -> Result<(), String> {
        if let Some(index) = self.parameter_names.iter().position(|n| n == name) {
            if index < self.parameters.len() {
                self.parameters[index] = value;
                Ok(())
            } else {
                Err("参数索引超出范围".to_string())
            }
        } else {
            Err(format!("未知参数名: {}", name))
        }
    }
    
    pub fn get_parameter(&self, name: &str) -> Option<f64> {
        self.parameter_names.iter().position(|n| n == name)
            .and_then(|index| self.parameters.get(index).copied())
    }
    
    pub fn clamp_parameters(&mut self) {
        for (i, param) in self.parameters.iter_mut().enumerate() {
            if i < self.bounds.len() {
                let (min, max) = self.bounds[i];
                *param = param.clamp(min, max);
            }
        }
    }
}

/// 峰形计算器trait
pub trait PeakShapeCalculator {
    fn calculate(&self, x: f64, params: &PeakShapeParams) -> f64;
    fn calculate_derivative(&self, x: f64, params: &PeakShapeParams, param_index: usize) -> f64;
    fn calculate_second_derivative(&self, x: f64, params: &PeakShapeParams, param_index: usize) -> f64;
}

/// 高斯峰形计算器
pub struct GaussianCalculator;

impl PeakShapeCalculator for GaussianCalculator {
    fn calculate(&self, x: f64, params: &PeakShapeParams) -> f64 {
        let amplitude = params.get_parameter("amplitude").unwrap_or(0.0);
        let center = params.get_parameter("center").unwrap_or(0.0);
        let sigma = params.get_parameter("sigma").unwrap_or(1.0);
        
        let exponent = -((x - center).powi(2)) / (2.0 * sigma.powi(2));
        amplitude * exponent.exp()
    }
    
    fn calculate_derivative(&self, x: f64, params: &PeakShapeParams, param_index: usize) -> f64 {
        let amplitude = params.get_parameter("amplitude").unwrap_or(0.0);
        let center = params.get_parameter("center").unwrap_or(0.0);
        let sigma = params.get_parameter("sigma").unwrap_or(1.0);
        
        let exponent = -((x - center).powi(2)) / (2.0 * sigma.powi(2));
        let exp_val = exponent.exp();
        
        match param_index {
            0 => exp_val, // 对amplitude的导数
            1 => amplitude * exp_val * (x - center) / sigma.powi(2), // 对center的导数
            2 => amplitude * exp_val * (x - center).powi(2) / sigma.powi(3), // 对sigma的导数
            _ => 0.0,
        }
    }
    
    fn calculate_second_derivative(&self, x: f64, params: &PeakShapeParams, param_index: usize) -> f64 {
        // 简化的二阶导数计算
        let h = 1e-6;
        let mut params_plus = params.clone();
        let mut params_minus = params.clone();
        
        if param_index < params.parameters.len() {
            params_plus.parameters[param_index] += h;
            params_minus.parameters[param_index] -= h;
        }
        
        let f_plus = self.calculate(x, &params_plus);
        let f_minus = self.calculate(x, &params_minus);
        
        (f_plus - 2.0 * self.calculate(x, params) + f_minus) / (h * h)
    }
}

/// 洛伦兹峰形计算器
pub struct LorentzianCalculator;

impl PeakShapeCalculator for LorentzianCalculator {
    fn calculate(&self, x: f64, params: &PeakShapeParams) -> f64 {
        let amplitude = params.get_parameter("amplitude").unwrap_or(0.0);
        let center = params.get_parameter("center").unwrap_or(0.0);
        let gamma = params.get_parameter("gamma").unwrap_or(1.0);
        
        let denominator = 1.0 + ((x - center) / gamma).powi(2);
        amplitude / denominator
    }
    
    fn calculate_derivative(&self, x: f64, params: &PeakShapeParams, param_index: usize) -> f64 {
        let amplitude = params.get_parameter("amplitude").unwrap_or(0.0);
        let center = params.get_parameter("center").unwrap_or(0.0);
        let gamma = params.get_parameter("gamma").unwrap_or(1.0);
        
        let denominator = 1.0 + ((x - center) / gamma).powi(2);
        
        match param_index {
            0 => 1.0 / denominator, // 对amplitude的导数
            1 => 2.0 * amplitude * (x - center) / (gamma.powi(2) * denominator.powi(2)), // 对center的导数
            2 => 2.0 * amplitude * (x - center).powi(2) / (gamma.powi(3) * denominator.powi(2)), // 对gamma的导数
            _ => 0.0,
        }
    }
    
    fn calculate_second_derivative(&self, x: f64, params: &PeakShapeParams, param_index: usize) -> f64 {
        // 简化的二阶导数计算
        let h = 1e-6;
        let mut params_plus = params.clone();
        let mut params_minus = params.clone();
        
        if param_index < params.parameters.len() {
            params_plus.parameters[param_index] += h;
            params_minus.parameters[param_index] -= h;
        }
        
        let f_plus = self.calculate(x, &params_plus);
        let f_minus = self.calculate(x, &params_minus);
        
        (f_plus - 2.0 * self.calculate(x, params) + f_minus) / (h * h)
    }
}

/// 伪Voigt峰形计算器
pub struct PseudoVoigtCalculator;

impl PeakShapeCalculator for PseudoVoigtCalculator {
    fn calculate(&self, x: f64, params: &PeakShapeParams) -> f64 {
        let amplitude = params.get_parameter("amplitude").unwrap_or(0.0);
        let center = params.get_parameter("center").unwrap_or(0.0);
        let sigma = params.get_parameter("sigma").unwrap_or(1.0);
        let mixing = params.get_parameter("mixing").unwrap_or(0.5);
        
        // 高斯部分
        let gaussian_exp = -((x - center).powi(2)) / (2.0 * sigma.powi(2));
        let gaussian = gaussian_exp.exp();
        
        // 洛伦兹部分
        let lorentzian = 1.0 / (1.0 + ((x - center) / sigma).powi(2));
        
        // 混合
        amplitude * (mixing * lorentzian + (1.0 - mixing) * gaussian)
    }
    
    fn calculate_derivative(&self, x: f64, params: &PeakShapeParams, param_index: usize) -> f64 {
        // 简化的导数计算
        let h = 1e-6;
        let mut params_plus = params.clone();
        let mut params_minus = params.clone();
        
        if param_index < params.parameters.len() {
            params_plus.parameters[param_index] += h;
            params_minus.parameters[param_index] -= h;
        }
        
        let f_plus = self.calculate(x, &params_plus);
        let f_minus = self.calculate(x, &params_minus);
        
        (f_plus - f_minus) / (2.0 * h)
    }
    
    fn calculate_second_derivative(&self, x: f64, params: &PeakShapeParams, param_index: usize) -> f64 {
        // 简化的二阶导数计算
        let h = 1e-6;
        let mut params_plus = params.clone();
        let mut params_minus = params.clone();
        
        if param_index < params.parameters.len() {
            params_plus.parameters[param_index] += h;
            params_minus.parameters[param_index] -= h;
        }
        
        let f_plus = self.calculate(x, &params_plus);
        let f_minus = self.calculate(x, &params_minus);
        
        (f_plus - 2.0 * self.calculate(x, params) + f_minus) / (h * h)
    }
}

/// 峰形计算器工厂
pub struct PeakShapeCalculatorFactory;

impl PeakShapeCalculatorFactory {
    pub fn create_calculator(shape_type: &PeakShapeType) -> Box<dyn PeakShapeCalculator> {
        match shape_type {
            PeakShapeType::Gaussian => Box::new(GaussianCalculator),
            PeakShapeType::Lorentzian => Box::new(LorentzianCalculator),
            PeakShapeType::PseudoVoigt => Box::new(PseudoVoigtCalculator),
            _ => Box::new(GaussianCalculator), // 默认使用高斯
        }
    }
}

/// 峰形分析器
#[derive(Debug)]
pub struct PeakShapeAnalyzer;

impl PeakShapeAnalyzer {
    /// 分析峰形并推荐最佳峰形类型
    pub fn analyze_peak_shape(&self, x_data: &[f64], y_data: &[f64]) -> PeakShapeType {
        if x_data.len() < 10 {
            return PeakShapeType::Gaussian;
        }
        
        // 计算峰的不对称性
        let asymmetry = self.calculate_asymmetry(x_data, y_data);
        
        // 计算拖尾程度
        let tailing = self.calculate_tailing(x_data, y_data);
        
        // 根据特征选择峰形
        if tailing > 0.3 {
            PeakShapeType::ExponentiallyModifiedGaussian
        } else if asymmetry > 0.2 {
            PeakShapeType::BiGaussian
        } else {
            PeakShapeType::Gaussian
        }
    }
    
    /// 计算峰的不对称性
    fn calculate_asymmetry(&self, x_data: &[f64], y_data: &[f64]) -> f64 {
        if x_data.is_empty() {
            return 0.0;
        }
        
        let max_idx = y_data.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap().0;
        let peak_center = x_data[max_idx];
        let peak_height = y_data[max_idx];
        let half_height = peak_height / 2.0;
        
        // 找到左右半高宽
        let mut left_hwhm = 0.0;
        let mut right_hwhm = 0.0;
        
        for i in (0..max_idx).rev() {
            if y_data[i] <= half_height {
                left_hwhm = peak_center - x_data[i];
                break;
            }
        }
        
        for i in (max_idx + 1)..x_data.len() {
            if y_data[i] <= half_height {
                right_hwhm = x_data[i] - peak_center;
                break;
            }
        }
        
        if left_hwhm > 0.0 && right_hwhm > 0.0 {
            (right_hwhm - left_hwhm).abs() / (left_hwhm + right_hwhm)
        } else {
            0.0
        }
    }
    
    /// 计算峰的拖尾程度
    fn calculate_tailing(&self, x_data: &[f64], y_data: &[f64]) -> f64 {
        if x_data.is_empty() {
            return 0.0;
        }
        
        let max_idx = y_data.iter().enumerate()
            .max_by(|a, b| a.1.partial_cmp(b.1).unwrap())
            .unwrap().0;
        let peak_center = x_data[max_idx];
        let peak_height = y_data[max_idx];
        
        // 计算峰右侧的拖尾衰减
        let mut tail_decay = 0.0;
        let mut tail_points = 0;
        
        for (i, &x) in x_data.iter().enumerate() {
            if x > peak_center {
                let distance = x - peak_center;
                let expected_height = peak_height * (-distance / (peak_height * 0.1)).exp();
                if y_data[i] > expected_height * 0.1 {
                    tail_decay += (y_data[i] - expected_height).abs();
                    tail_points += 1;
                }
            }
        }
        
        if tail_points > 0 {
            tail_decay / tail_points as f64 / peak_height
        } else {
            0.0
        }
    }
}
