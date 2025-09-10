use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::curve::Curve;
use super::peak::Peak;

/// Processing result containing curves and peaks
#[derive(Debug, Clone)]
pub struct ProcessingResult {
    pub curves: Vec<Curve>,
    pub peaks: Vec<Peak>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ProcessingResult {
    /// Create a new processing result
    pub fn new() -> Self {
        Self {
            curves: Vec::new(),
            peaks: Vec::new(),
            metadata: HashMap::new(),
        }
    }
    
    /// Add a curve to the result
    pub fn add_curve(&mut self, curve: Curve) {
        self.curves.push(curve);
    }
    
    /// Add a peak to the result
    pub fn add_peak(&mut self, peak: Peak) {
        self.peaks.push(peak);
    }
    
    /// Get the number of curves
    pub fn curve_count(&self) -> usize {
        self.curves.len()
    }
    
    /// Get the number of peaks
    pub fn peak_count(&self) -> usize {
        self.peaks.len()
    }
    
    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
    
    /// Merge with another processing result
    pub fn merge(&mut self, other: ProcessingResult) {
        self.curves.extend(other.curves);
        self.peaks.extend(other.peaks);
        self.metadata.extend(other.metadata);
    }
}

impl Default for ProcessingResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Processing error types
#[derive(Debug, thiserror::Error)]
pub enum ProcessingError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("Data error: {0}")]
    DataError(String),
    #[error("Processing error: {0}")]
    ProcessError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("mzdata error: {0}")]
    MzDataError(String),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("Mathematical error: {0}")]
    MathError(String),
    #[error("Validation error: {0}")]
    ValidationError(String),
}

impl ProcessingError {
    /// Create a configuration error
    pub fn config_error(message: &str) -> Self {
        Self::ConfigError(message.to_string())
    }
    
    /// Create a data error
    pub fn data_error(message: &str) -> Self {
        Self::DataError(message.to_string())
    }
    
    /// Create a processing error
    pub fn process_error(message: &str) -> Self {
        Self::ProcessError(message.to_string())
    }
    
    /// Create a mathematical error
    pub fn math_error(message: &str) -> Self {
        Self::MathError(message.to_string())
    }
    
    /// Create a validation error
    pub fn validation_error(message: &str) -> Self {
        Self::ValidationError(message.to_string())
    }
}

/// Processing status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProcessingStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Processing progress information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingProgress {
    pub status: ProcessingStatus,
    pub current_step: String,
    pub total_steps: usize,
    pub current_step_index: usize,
    pub progress_percentage: f64,
    pub estimated_remaining_time: Option<u64>, // in milliseconds
    pub start_time: Option<u64>, // timestamp
    pub end_time: Option<u64>, // timestamp
    pub error_message: Option<String>,
}

impl ProcessingProgress {
    /// Create a new processing progress
    pub fn new(total_steps: usize) -> Self {
        Self {
            status: ProcessingStatus::Pending,
            current_step: String::new(),
            total_steps,
            current_step_index: 0,
            progress_percentage: 0.0,
            estimated_remaining_time: None,
            start_time: None,
            end_time: None,
            error_message: None,
        }
    }
    
    /// Update progress
    pub fn update(&mut self, step_index: usize, step_name: &str) {
        self.current_step_index = step_index;
        self.current_step = step_name.to_string();
        self.progress_percentage = if self.total_steps > 0 {
            (step_index as f64 / self.total_steps as f64) * 100.0
        } else {
            0.0
        };
    }
    
    /// Mark as completed
    pub fn mark_completed(&mut self) {
        self.status = ProcessingStatus::Completed;
        self.progress_percentage = 100.0;
        self.end_time = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64);
    }
    
    /// Mark as failed
    pub fn mark_failed(&mut self, error_message: String) {
        self.status = ProcessingStatus::Failed;
        self.error_message = Some(error_message);
        self.end_time = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64);
    }
    
    /// Start processing
    pub fn start(&mut self) {
        self.status = ProcessingStatus::InProgress;
        self.start_time = Some(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as u64);
    }
}

/// Processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub enabled: bool,
    pub priority: u32,
}

impl ProcessingConfig {
    /// Create a new processing configuration
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            parameters: HashMap::new(),
            enabled: true,
            priority: 0,
        }
    }
    
    /// Add a parameter
    pub fn add_parameter(&mut self, key: String, value: serde_json::Value) {
        self.parameters.insert(key, value);
    }
    
    /// Get a parameter
    pub fn get_parameter(&self, key: &str) -> Option<&serde_json::Value> {
        self.parameters.get(key)
    }
    
    /// Get a parameter with default value
    pub fn get_parameter_or_default<T>(&self, key: &str, default: T) -> T 
    where
        T: for<'de> Deserialize<'de> + Clone,
    {
        self.parameters.get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
            .unwrap_or(default)
    }
}
