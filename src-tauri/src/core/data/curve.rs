use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Curve data - contains complete scientific parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Curve {
    /// Curve unique identifier
    pub id: String,
    /// Curve type ("DT", "TIC", "XIC", "EIC", "BPC", "TIC_MS1", "TIC_MS2", etc.)
    pub curve_type: String,
    
    // === Data points ===
    /// X-axis data points (time, m/z, etc.)
    pub x_values: Vec<f64>,
    /// Y-axis data points (intensity values)
    pub y_values: Vec<f64>,
    
    // === Axis labels and units ===
    /// X-axis label
    pub x_label: String,
    /// Y-axis label
    pub y_label: String,
    /// X-axis unit
    pub x_unit: String,
    /// Y-axis unit
    pub y_unit: String,
    
    // === Data range ===
    /// X-axis minimum value
    pub x_min: f64,
    /// X-axis maximum value
    pub x_max: f64,
    /// Y-axis minimum value
    pub y_min: f64,
    /// Y-axis maximum value
    pub y_max: f64,
    
    // === Statistical parameters ===
    /// Total number of data points
    pub point_count: usize,
    /// Total ion current (TIC)
    pub total_ion_current: f64,
    /// Average intensity
    pub mean_intensity: f64,
    /// Intensity standard deviation
    pub intensity_std: f64,
    /// Baseline intensity
    pub baseline_intensity: f64,
    /// Signal-to-noise ratio
    pub signal_to_noise_ratio: f64,
    
    // === Mass spectrometry related parameters ===
    /// m/z range (for XIC/EIC curves)
    pub mz_range: Option<(f64, f64)>,
    /// Retention time range
    pub rt_range: Option<(f64, f64)>,
    /// Drift time range
    pub dt_range: Option<(f64, f64)>,
    /// MS level
    pub ms_level: Option<u8>,
    
    // === Processing parameters ===
    /// Smoothing factor
    pub smoothing_factor: Option<f64>,
    /// Baseline correction method
    pub baseline_correction: Option<String>,
    /// Noise level
    pub noise_level: f64,
    /// Detection threshold
    pub detection_threshold: f64,
    
    // === Quality parameters ===
    /// Data quality score (0-1)
    pub quality_score: f64,
    /// Data completeness (0-1)
    pub completeness: f64,
    /// Whether there are missing data points
    pub has_missing_points: bool,
    
    // === Metadata ===
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Curve {
    /// Create a new curve
    pub fn new(
        id: String,
        curve_type: String,
        x_values: Vec<f64>,
        y_values: Vec<f64>,
        x_label: String,
        y_label: String,
        x_unit: String,
        y_unit: String,
    ) -> Self {
        let point_count = x_values.len();
        let x_min = x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let x_max: f64 = x_values.iter().fold(0.0, |a, &b| a.max(b));
        let y_min = y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        let y_max: f64 = y_values.iter().fold(0.0, |a, &b| a.max(b));
        
        let total_ion_current: f64 = y_values.iter().sum();
        let mean_intensity = total_ion_current / point_count as f64;
        let variance: f64 = y_values.iter()
            .map(|&y| (y - mean_intensity).powi(2))
            .sum::<f64>() / point_count as f64;
        let intensity_std = variance.sqrt();
        
        Self {
            id,
            curve_type,
            x_values,
            y_values,
            x_label,
            y_label,
            x_unit,
            y_unit,
            x_min,
            x_max,
            y_min,
            y_max,
            point_count,
            total_ion_current,
            mean_intensity,
            intensity_std,
            baseline_intensity: y_min,
            signal_to_noise_ratio: if intensity_std > 0.0 { (y_max - y_min) / intensity_std } else { 0.0 },
            mz_range: None,
            rt_range: None,
            dt_range: None,
            ms_level: None,
            smoothing_factor: None,
            baseline_correction: None,
            noise_level: intensity_std,
            detection_threshold: y_min + 3.0 * intensity_std,
            quality_score: 1.0,
            completeness: 1.0,
            has_missing_points: false,
            metadata: HashMap::new(),
        }
    }
    
    /// Calculate signal-to-noise ratio
    pub fn calculate_signal_to_noise(&mut self) {
        if self.intensity_std > 0.0 {
            self.signal_to_noise_ratio = (self.y_max - self.baseline_intensity) / self.intensity_std;
        }
    }
    
    /// Set m/z range
    pub fn set_mz_range(&mut self, mz_min: f64, mz_max: f64) {
        self.mz_range = Some((mz_min, mz_max));
    }
    
    /// Set retention time range
    pub fn set_rt_range(&mut self, rt_min: f64, rt_max: f64) {
        self.rt_range = Some((rt_min, rt_max));
    }
    
    /// Set drift time range
    pub fn set_dt_range(&mut self, dt_min: f64, dt_max: f64) {
        self.dt_range = Some((dt_min, dt_max));
    }
    
    /// Get data point at index
    pub fn get_point(&self, index: usize) -> Option<(f64, f64)> {
        if index < self.point_count {
            Some((self.x_values[index], self.y_values[index]))
        } else {
            None
        }
    }
    
    /// Find peak in the curve
    pub fn find_peak(&self, x_value: f64, tolerance: f64) -> Option<usize> {
        for (index, &x) in self.x_values.iter().enumerate() {
            if (x - x_value).abs() <= tolerance {
                return Some(index);
            }
        }
        None
    }
    
    /// Get intensity at specific x value (interpolated)
    pub fn get_intensity_at(&self, x_value: f64) -> Option<f64> {
        if self.x_values.is_empty() {
            return None;
        }
        
        // Find the two closest points
        let mut closest_index = 0;
        let mut min_distance = (self.x_values[0] - x_value).abs();
        
        for (i, &x) in self.x_values.iter().enumerate() {
            let distance = (x - x_value).abs();
            if distance < min_distance {
                min_distance = distance;
                closest_index = i;
            }
        }
        
        // Simple linear interpolation
        if closest_index == 0 {
            Some(self.y_values[0])
        } else if closest_index == self.x_values.len() - 1 {
            Some(self.y_values[closest_index])
        } else {
            let x1 = self.x_values[closest_index - 1];
            let x2 = self.x_values[closest_index];
            let y1 = self.y_values[closest_index - 1];
            let y2 = self.y_values[closest_index];
            
            let interpolated = y1 + (y2 - y1) * (x_value - x1) / (x2 - x1);
            Some(interpolated)
        }
    }
    
    /// Calculate area under the curve
    pub fn calculate_area(&self) -> f64 {
        if self.point_count < 2 {
            return 0.0;
        }
        
        let mut area = 0.0;
        for i in 1..self.point_count {
            let dx = self.x_values[i] - self.x_values[i - 1];
            let avg_y = (self.y_values[i] + self.y_values[i - 1]) / 2.0;
            area += dx * avg_y;
        }
        area
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
