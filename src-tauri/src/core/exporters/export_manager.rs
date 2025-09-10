use std::collections::HashMap;
use serde_json::Value;

use crate::core::data::{DataContainer, ProcessingError};
use super::base::{Exporter, ExportResult, ExportConfig};

/// Export manager that handles multiple export formats
pub struct ExportManager {
    exporters: HashMap<String, Box<dyn Exporter>>,
}

impl ExportManager {
    /// Create a new export manager with default exporters
    pub fn new() -> Self {
        let mut manager = Self {
            exporters: HashMap::new(),
        };
        
        // Register default exporters
        manager.register_exporter("tsv", Box::new(super::TsvExporter));
        manager.register_exporter("plotly", Box::new(super::PlotlyExporter));
        manager.register_exporter("curve_tsv", Box::new(super::CurveTsvExporter));
        manager.register_exporter("spectro_tsv", Box::new(super::SpectroTsvExporter));
        
        manager
    }
    
    /// Register a new exporter
    pub fn register_exporter(&mut self, name: &str, exporter: Box<dyn Exporter>) {
        self.exporters.insert(name.to_string(), exporter);
    }
    
    /// Get list of available exporters
    pub fn available_exporters(&self) -> Vec<String> {
        self.exporters.keys().cloned().collect()
    }
    
    /// Export data using the specified exporter
    pub async fn export(
        &self,
        exporter_name: &str,
        data: &DataContainer,
        config: Value,
    ) -> Result<ExportResult, ProcessingError> {
        let exporter = self.exporters.get(exporter_name)
            .ok_or_else(|| ProcessingError::ConfigError(
                format!("Exporter '{}' not found. Available exporters: {:?}", 
                    exporter_name, self.available_exporters())
            ))?;
        
        exporter.export(data, config).await
    }
    
    /// Export data to multiple formats
    pub async fn export_multiple(
        &self,
        formats: &[String],
        data: &DataContainer,
        config: Value,
    ) -> Result<Vec<ExportResult>, ProcessingError> {
        let mut results = Vec::new();
        
        for format in formats {
            match self.export(format, data, config.clone()).await {
                Ok(result) => results.push(result),
                Err(e) => {
                    eprintln!("Failed to export to {}: {}", format, e);
                    // Continue with other formats
                }
            }
        }
        
        if results.is_empty() {
            return Err(ProcessingError::ProcessError(
                "All export attempts failed".to_string()
            ));
        }
        
        Ok(results)
    }
    
    /// Get exporter information
    pub fn get_exporter_info(&self, name: &str) -> Option<ExporterInfo> {
        self.exporters.get(name).map(|exporter| ExporterInfo {
            name: exporter.name().to_string(),
            description: exporter.description().to_string(),
            file_extension: exporter.file_extension().to_string(),
            mime_type: exporter.mime_type().to_string(),
            config_schema: exporter.config_schema(),
        })
    }
    
    /// Get all exporter information
    pub fn get_all_exporter_info(&self) -> HashMap<String, ExporterInfo> {
        let mut info = HashMap::new();
        for (name, exporter) in &self.exporters {
            info.insert(name.clone(), ExporterInfo {
                name: exporter.name().to_string(),
                description: exporter.description().to_string(),
                file_extension: exporter.file_extension().to_string(),
                mime_type: exporter.mime_type().to_string(),
                config_schema: exporter.config_schema(),
            });
        }
        info
    }
}

impl Default for ExportManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about an exporter
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExporterInfo {
    pub name: String,
    pub description: String,
    pub file_extension: String,
    pub mime_type: String,
    pub config_schema: Value,
}

/// Batch export configuration
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BatchExportConfig {
    /// List of formats to export to
    pub formats: Vec<String>,
    /// Base configuration for all exports
    pub base_config: ExportConfig,
    /// Format-specific configurations
    pub format_configs: HashMap<String, Value>,
    /// Output directory
    pub output_dir: Option<String>,
    /// File prefix
    pub file_prefix: Option<String>,
}

impl Default for BatchExportConfig {
    fn default() -> Self {
        Self {
            formats: vec!["tsv".to_string()],
            base_config: ExportConfig::default(),
            format_configs: HashMap::new(),
            output_dir: None,
            file_prefix: None,
        }
    }
}

/// Batch export result
#[derive(Debug, Clone)]
pub struct BatchExportResult {
    pub results: Vec<ExportResult>,
    pub failed_formats: Vec<String>,
    pub total_files: usize,
    pub total_size: usize,
}

impl ExportManager {
    /// Perform batch export to multiple formats
    pub async fn batch_export(
        &self,
        data: &DataContainer,
        config: BatchExportConfig,
    ) -> Result<BatchExportResult, ProcessingError> {
        let mut results = Vec::new();
        let mut failed_formats = Vec::new();
        let mut total_size = 0;
        
        for format in &config.formats {
            // Get format-specific config or use base config
            let format_config = config.format_configs.get(format)
                .cloned()
                .unwrap_or_else(|| serde_json::to_value(&config.base_config).unwrap());
            
            match self.export(format, data, format_config).await {
                Ok(mut result) => {
                    // Apply file prefix if specified
                    if let Some(prefix) = &config.file_prefix {
                        result.filename = format!("{}_{}", prefix, result.filename);
                    }
                    
                    total_size += result.data.len();
                    results.push(result);
                }
                Err(e) => {
                    eprintln!("Failed to export to {}: {}", format, e);
                    failed_formats.push(format.clone());
                }
            }
        }
        
        Ok(BatchExportResult {
            results,
            failed_formats,
            total_files: config.formats.len(),
            total_size,
        })
    }
}
