use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Peak shape type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PeakType {
    Gaussian,
    Lorentzian,
    PseudoVoigt,
    AsymmetricGaussian,
    /// Exponentially Modified Gaussian - 指数修正高斯峰
    EMG,
    /// Bi-Gaussian - 双高斯峰
    BiGaussian,
    /// Voigt with exponential tail - Voigt峰加指数尾
    VoigtExponentialTail,
    /// Pearson-IV distribution - Pearson-IV分布峰
    PearsonIV,
    /// Non-Linear Curve - 非线性曲线峰
    NLC,
    /// GMG Bayesian - GMG贝叶斯峰
    GMGBayesian,
    Custom(String),
}

/// Detection algorithm enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DetectionAlgorithm {
    CWT,
    PeakFinder,
    Simple,
    SavitzkyGolay,
    Custom(String),
}

/// Peak data - high precision mass spectrometry data format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Peak {
    /// Peak unique identifier
    pub id: String,
    /// Belonging curve ID
    pub curve_id: String,
    
    // === Basic position and intensity parameters (high precision) ===
    /// Peak center position (time unit: ms or min, precision: 1e-6)
    pub center: f64,
    /// Peak maximum intensity (original intensity unit, precision: 1e-3)
    pub amplitude: f64,
    /// Peak area (intensity × time unit, precision: 1e-3)
    pub area: f64,
    
    // === Peak width parameters (high precision) ===
    /// Full Width at Half Maximum (precision: 1e-6)
    pub fwhm: f64,
    /// Half Width at Half Maximum (precision: 1e-6)
    pub hwhm: f64,
    /// Peak width standard deviation σ (Gaussian fit parameter, precision: 1e-6)
    pub sigma: f64,
    /// Peak width parameter (Lorentzian fit parameter, precision: 1e-6)
    pub gamma: f64,
    /// Exponential decay constant (for EMG peaks, precision: 1e-6)
    pub tau: f64,
    
    // === Peak shape parameters (high precision) ===
    /// Left half width (distance from center to left half peak point, precision: 1e-6)
    pub left_hwhm: f64,
    /// Right half width (distance from center to right half peak point, precision: 1e-6)
    pub right_hwhm: f64,
    /// Peak shape asymmetry factor (right_hwhm / left_hwhm, precision: 1e-4)
    pub asymmetry_factor: f64,
    
    // === Peak boundary parameters (high precision) ===
    /// Left boundary position (peak start position, precision: 1e-6)
    pub left_boundary: f64,
    /// Right boundary position (peak end position, precision: 1e-6)
    pub right_boundary: f64,
    /// Peak span (right_boundary - left_boundary, precision: 1e-6)
    pub peak_span: f64,
    
    // === Fit quality parameters (high precision) ===
    /// Fit goodness R² (precision: 1e-6)
    pub rsquared: f64,
    /// Fit residual sum of squares (precision: 1e-6)
    pub residual_sum_squares: f64,
    /// Fit standard error (precision: 1e-6)
    pub standard_error: f64,
    /// Number of fit parameters
    pub parameter_count: u32,
    
    // === Peak type and parameters ===
    /// Peak shape type
    pub peak_type: PeakType,
    /// Peak shape mixing parameter (Lorentzian proportion in Pseudo-Voigt, precision: 1e-4)
    pub mixing_parameter: f64,
    
    // === Statistical parameters (high precision) ===
    /// Peak height to baseline ratio (precision: 1e-4)
    pub signal_to_baseline_ratio: f64,
    /// Peak area to total area ratio (precision: 1e-6)
    pub area_percentage: f64,
    /// Peak intensity to maximum intensity ratio (precision: 1e-6)
    pub intensity_percentage: f64,
    
    // === Derivative parameters (high precision) ===
    /// Left derivative (peak left side slope, precision: 1e-6)
    pub left_derivative: f64,
    /// Right derivative (peak right side slope, precision: 1e-6)
    pub right_derivative: f64,
    /// Derivative ratio (right_derivative / left_derivative, precision: 1e-4)
    pub derivative_ratio: f64,
    
    // === Mass spectrometry related parameters (high precision) ===
    /// Mass-to-charge ratio (m/z, precision: 1e-6)
    pub mz: Option<f64>,
    /// Retention time (min, precision: 1e-6)
    pub retention_time: Option<f64>,
    /// Drift time (ms, precision: 1e-6)
    pub drift_time: Option<f64>,
    /// MS level
    pub ms_level: Option<u8>,
    
    // === Detection parameters ===
    /// Detection algorithm name
    pub detection_algorithm: DetectionAlgorithm,
    /// Detection threshold (precision: 1e-3)
    pub detection_threshold: f64,
    /// Detection confidence (precision: 1e-4)
    pub confidence: f64,
    
    // === Fit parameters (for multi-peak fitting) ===
    /// Fit parameter vector (precision: 1e-6)
    pub fit_parameters: Vec<f64>,
    /// Fit parameter standard errors (precision: 1e-6)
    pub fit_parameter_errors: Vec<f64>,
    /// Fit covariance matrix (precision: 1e-6)
    pub fit_covariance_matrix: Option<Vec<Vec<f64>>>,
    
    // === Metadata ===
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Peak {
    /// Create a new peak
    pub fn new(
        id: String,
        curve_id: String,
        center: f64,
        amplitude: f64,
        peak_type: PeakType,
    ) -> Self {
        Self {
            id,
            curve_id,
            center,
            amplitude,
            area: 0.0,
            fwhm: 0.0,
            hwhm: 0.0,
            sigma: 0.0,
            gamma: 0.0,
            tau: 0.0,
            left_hwhm: 0.0,
            right_hwhm: 0.0,
            asymmetry_factor: 1.0,
            left_boundary: center,
            right_boundary: center,
            peak_span: 0.0,
            rsquared: 0.0,
            residual_sum_squares: 0.0,
            standard_error: 0.0,
            parameter_count: 0,
            peak_type,
            mixing_parameter: 0.0,
            signal_to_baseline_ratio: 0.0,
            area_percentage: 0.0,
            intensity_percentage: 0.0,
            left_derivative: 0.0,
            right_derivative: 0.0,
            derivative_ratio: 0.0,
            mz: None,
            retention_time: None,
            drift_time: None,
            ms_level: None,
            detection_algorithm: DetectionAlgorithm::Simple,
            detection_threshold: 0.0,
            confidence: 0.0,
            fit_parameters: Vec::new(),
            fit_parameter_errors: Vec::new(),
            fit_covariance_matrix: None,
            metadata: HashMap::new(),
        }
    }
    
    /// Calculate asymmetry factor
    pub fn calculate_asymmetry_factor(&mut self) {
        if self.left_hwhm > 0.0 {
            self.asymmetry_factor = self.right_hwhm / self.left_hwhm;
        }
    }
    
    /// Calculate peak span
    pub fn calculate_peak_span(&mut self) {
        self.peak_span = self.right_boundary - self.left_boundary;
    }
    
    /// Calculate derivative ratio
    pub fn calculate_derivative_ratio(&mut self) {
        if self.left_derivative != 0.0 {
            self.derivative_ratio = self.right_derivative / self.left_derivative;
        }
    }
    
    /// Set mass spectrometry related parameters
    pub fn set_ms_parameters(&mut self, mz: Option<f64>, rt: Option<f64>, dt: Option<f64>, ms_level: Option<u8>) {
        self.mz = mz;
        self.retention_time = rt;
        self.drift_time = dt;
        self.ms_level = ms_level;
    }
    
    /// Set detection parameters
    pub fn set_detection_parameters(&mut self, algorithm: DetectionAlgorithm, threshold: f64, confidence: f64) {
        self.detection_algorithm = algorithm;
        self.detection_threshold = threshold;
        self.confidence = confidence;
    }
    
    /// Set fit parameters
    pub fn set_fit_parameters(&mut self, parameters: Vec<f64>, errors: Vec<f64>, covariance: Option<Vec<Vec<f64>>>) {
        self.fit_parameters = parameters;
        self.fit_parameter_errors = errors;
        self.fit_covariance_matrix = covariance;
        self.parameter_count = self.fit_parameters.len() as u32;
    }
    
    /// Calculate peak area (using fit parameters)
    pub fn calculate_area_from_fit(&mut self) {
        match self.peak_type {
            PeakType::Gaussian => {
                if self.fit_parameters.len() >= 3 {
                    let amplitude = self.fit_parameters[0];
                    let sigma = self.fit_parameters[2];
                    self.area = amplitude * sigma * (std::f64::consts::PI * 2.0).sqrt();
                }
            }
            PeakType::Lorentzian => {
                if self.fit_parameters.len() >= 3 {
                    let amplitude = self.fit_parameters[0];
                    let gamma = self.fit_parameters[2];
                    self.area = amplitude * gamma * std::f64::consts::PI;
                }
            }
            PeakType::PseudoVoigt => {
                if self.fit_parameters.len() >= 4 {
                    let amplitude = self.fit_parameters[0];
                    let sigma = self.fit_parameters[2];
                    let gamma = self.fit_parameters[3];
                    let mixing = self.mixing_parameter;
                    let gaussian_area = amplitude * sigma * (std::f64::consts::PI * 2.0).sqrt();
                    let lorentzian_area = amplitude * gamma * std::f64::consts::PI;
                    self.area = mixing * lorentzian_area + (1.0 - mixing) * gaussian_area;
                }
            }
            PeakType::EMG => {
                if self.fit_parameters.len() >= 4 {
                    let amplitude = self.fit_parameters[0];
                    let sigma = self.fit_parameters[2];
                    let tau = self.fit_parameters[3]; // 指数衰减常数
                    // EMG面积计算：A * σ * √(2π) * exp(σ²/(2τ²))
                    self.area = amplitude * sigma * (std::f64::consts::PI * 2.0).sqrt() * (sigma * sigma / (2.0 * tau * tau)).exp();
                }
            }
            PeakType::BiGaussian => {
                if self.fit_parameters.len() >= 6 {
                    let amplitude = self.fit_parameters[0];
                    let sigma1 = self.fit_parameters[2];
                    let sigma2 = self.fit_parameters[3];
                    let mixing = self.fit_parameters[4]; // 两个高斯的混合比例
                    let area1 = amplitude * sigma1 * (std::f64::consts::PI * 2.0).sqrt();
                    let area2 = amplitude * sigma2 * (std::f64::consts::PI * 2.0).sqrt();
                    self.area = mixing * area1 + (1.0 - mixing) * area2;
                }
            }
            PeakType::VoigtExponentialTail => {
                if self.fit_parameters.len() >= 5 {
                    let amplitude = self.fit_parameters[0];
                    let sigma = self.fit_parameters[2];
                    let _gamma = self.fit_parameters[3];
                    let tau = self.fit_parameters[4]; // 指数尾衰减常数
                    let voigt_area = amplitude * sigma * (std::f64::consts::PI * 2.0).sqrt() * 0.5; // 简化的Voigt面积
                    let tail_area = amplitude * tau; // 指数尾面积
                    self.area = voigt_area + tail_area;
                }
            }
            PeakType::PearsonIV => {
                if self.fit_parameters.len() >= 5 {
                    let amplitude = self.fit_parameters[0];
                    let a = self.fit_parameters[2]; // 形状参数
                    let b = self.fit_parameters[3]; // 形状参数
                    let _c = self.fit_parameters[4]; // 形状参数
                    // Pearson-IV面积计算（简化版本）
                    self.area = amplitude * (std::f64::consts::PI * 2.0).sqrt() * (1.0 + a * a / (b * b)).sqrt();
                }
            }
            PeakType::NLC => {
                if self.fit_parameters.len() >= 4 {
                    let amplitude = self.fit_parameters[0];
                    let sigma = self.fit_parameters[2];
                    let nonlinear_param = self.fit_parameters[3];
                    // 非线性曲线面积计算（简化版本）
                    self.area = amplitude * sigma * (std::f64::consts::PI * 2.0).sqrt() * (1.0 + nonlinear_param);
                }
            }
            PeakType::GMGBayesian => {
                if self.fit_parameters.len() >= 4 {
                    let amplitude = self.fit_parameters[0];
                    let sigma = self.fit_parameters[2];
                    let bayesian_weight = self.fit_parameters[3];
                    // GMG贝叶斯面积计算
                    self.area = amplitude * sigma * (std::f64::consts::PI * 2.0).sqrt() * bayesian_weight;
                }
            }
            _ => {
                // For other types, use simple trapezoidal integration
                self.area = self.amplitude * self.fwhm * 0.5;
            }
        }
    }
    
    /// Get peak width at specified height
    pub fn get_width_at_height(&self, height_fraction: f64) -> Option<f64> {
        if height_fraction <= 0.0 || height_fraction >= 1.0 {
            return None;
        }
        
        let _target_intensity = self.amplitude * height_fraction;
        
        // This is a simplified calculation
        // In practice, you would need the actual peak data to calculate this accurately
        match self.peak_type {
            PeakType::Gaussian => {
                // For Gaussian: width = 2 * sigma * sqrt(-2 * ln(height_fraction))
                Some(2.0 * self.sigma * (-2.0 * height_fraction.ln()).sqrt())
            }
            PeakType::Lorentzian => {
                // For Lorentzian: width = 2 * gamma * sqrt(1/height_fraction - 1)
                Some(2.0 * self.gamma * (1.0 / height_fraction - 1.0).sqrt())
            }
            _ => {
                // For other types, use FWHM as approximation
                Some(self.fwhm * height_fraction)
            }
        }
    }
    
    /// Check if peak is well-resolved (FWHM criterion)
    pub fn is_well_resolved(&self, resolution_threshold: f64) -> bool {
        self.fwhm <= resolution_threshold
    }
    
    /// Check if peak is symmetric
    pub fn is_symmetric(&self, asymmetry_tolerance: f64) -> bool {
        (self.asymmetry_factor - 1.0).abs() <= asymmetry_tolerance
    }
    
    /// Get peak quality score
    pub fn get_quality_score(&self) -> f64 {
        let mut score = 0.0;
        
        // R² contribution (40%)
        score += self.rsquared * 0.4;
        
        // Symmetry contribution (20%)
        let symmetry_score = if self.is_symmetric(0.2) { 1.0 } else { 0.5 };
        score += symmetry_score * 0.2;
        
        // Confidence contribution (20%)
        score += self.confidence * 0.2;
        
        // Resolution contribution (20%)
        let resolution_score = if self.is_well_resolved(1.0) { 1.0 } else { 0.5 };
        score += resolution_score * 0.2;
        
        score.min(1.0)
    }
    
    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}
