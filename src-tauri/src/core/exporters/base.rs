use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;

use crate::core::data::{DataContainer, ProcessingError};

/// Base trait for all data exporters
#[async_trait]
pub trait Exporter: Send + Sync {
    /// Get the name of the exporter
    fn name(&self) -> &str;
    
    /// Get the description of the exporter
    fn description(&self) -> &str;
    
    /// Get the file extension for this exporter
    fn file_extension(&self) -> &str;
    
    /// Get the MIME type for this exporter
    fn mime_type(&self) -> &str;
    
    /// Get the configuration schema for this exporter
    fn config_schema(&self) -> Value;
    
    /// Export data to the specified format
    async fn export(
        &self,
        data: &DataContainer,
        config: Value,
    ) -> Result<ExportResult, ProcessingError>;
}

/// Export result containing the exported data and metadata
#[derive(Debug, Clone)]
pub struct ExportResult {
    /// The exported data as bytes
    pub data: Vec<u8>,
    /// The filename (without path)
    pub filename: String,
    /// The MIME type of the exported data
    pub mime_type: String,
    /// Additional metadata about the export
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Export configuration for common options
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportConfig {
    /// Include header row in the export
    pub include_header: bool,
    /// Decimal precision for numeric values
    pub decimal_precision: usize,
    /// Include metadata in the export
    pub include_metadata: bool,
    /// Custom separator for delimited formats
    pub separator: Option<String>,
    /// Include curve data in the export
    pub include_curves: bool,
    /// Include peak data in the export
    pub include_peaks: bool,
    /// Include fitted curves for visualization
    pub include_fitted_curves: Option<bool>,
    /// Number of points for fitted curves
    pub fitted_curve_points: Option<usize>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            include_header: true,
            decimal_precision: 6,
            include_metadata: true,
            separator: None,
            include_curves: true,
            include_peaks: true,
            include_fitted_curves: Some(true),
            fitted_curve_points: Some(100),
        }
    }
}

/// Helper functions for exporters
pub mod helpers {
    use super::*;
    
    /// Format a floating point number with specified precision
    pub fn format_float(value: f64, precision: usize) -> String {
        format!("{:.precision$}", value, precision = precision)
    }
    
    /// Escape CSV/TSV values
    pub fn escape_delimited_value(value: &str, separator: &str) -> String {
        if value.contains(separator) || value.contains('"') || value.contains('\n') {
            format!("\"{}\"", value.replace('"', "\"\""))
        } else {
            value.to_string()
        }
    }
    
    /// Generate timestamp for filenames
    pub fn generate_timestamp() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        format!("{}", timestamp)
    }
    
    /// Create export metadata
    pub fn create_export_metadata(
        exporter_name: &str,
        curve_count: usize,
        peak_count: usize,
        config: &ExportConfig,
    ) -> HashMap<String, serde_json::Value> {
        let mut metadata = HashMap::new();
        metadata.insert("exporter".to_string(), serde_json::json!(exporter_name));
        metadata.insert("export_timestamp".to_string(), serde_json::json!(generate_timestamp()));
        metadata.insert("curve_count".to_string(), serde_json::json!(curve_count));
        metadata.insert("peak_count".to_string(), serde_json::json!(peak_count));
        metadata.insert("include_header".to_string(), serde_json::json!(config.include_header));
        metadata.insert("decimal_precision".to_string(), serde_json::json!(config.decimal_precision));
        metadata.insert("include_metadata".to_string(), serde_json::json!(config.include_metadata));
        metadata
    }
}
