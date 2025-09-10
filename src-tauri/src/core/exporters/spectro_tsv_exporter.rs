use async_trait::async_trait;
use serde_json::Value;
use mzdata::prelude::*;
use std::fs;
use std::path::Path;

use crate::core::data::{DataContainer, ProcessingError};
use super::base::{Exporter, ExportResult, ExportConfig, helpers};

/// Spectro TSV exporter for exporting spectra data in mz, dt, intensity format
pub struct SpectroTsvExporter;

#[async_trait]
impl Exporter for SpectroTsvExporter {
    fn name(&self) -> &str {
        "spectro_tsv_exporter"
    }

    fn description(&self) -> &str {
        "Export spectra data to TSV format with mz, dt, intensity columns"
    }

    fn file_extension(&self) -> &str {
        "tsv"
    }

    fn mime_type(&self) -> &str {
        "text/tab-separated-values"
    }

    fn config_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "include_header": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include header row in the export"
                },
                "decimal_precision": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 15,
                    "default": 6,
                    "description": "Decimal precision for numeric values"
                },
                "include_metadata": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include metadata in the export"
                },
                "filter_by_ms_level": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 3,
                    "description": "Filter spectra by MS level (optional)"
                },
                "mz_range_min": {
                    "type": "number",
                    "description": "Minimum m/z value to include (optional)"
                },
                "mz_range_max": {
                    "type": "number",
                    "description": "Maximum m/z value to include (optional)"
                },
                "rt_range_min": {
                    "type": "number",
                    "description": "Minimum retention time to include (optional)"
                },
                "rt_range_max": {
                    "type": "number",
                    "description": "Maximum retention time to include (optional)"
                },
                "intensity_threshold": {
                    "type": "number",
                    "minimum": 0,
                    "description": "Minimum intensity threshold (optional)"
                },
                "output_path": {
                    "type": "string",
                    "description": "Output file path (optional, if not provided, data will be returned)"
                }
            }
        })
    }

    async fn export(
        &self,
        data: &DataContainer,
        config: Value,
    ) -> Result<ExportResult, ProcessingError> {
        log::info!("🚀 SpectroTsvExporter: 开始导出，配置: {}", config);
        
        let export_config: ExportConfig = serde_json::from_value(config.clone())
            .unwrap_or_default();
        
        let include_header = config["include_header"].as_bool().unwrap_or(true);
        let decimal_precision = config["decimal_precision"].as_u64().unwrap_or(6) as usize;
        let include_metadata = config["include_metadata"].as_bool().unwrap_or(true);
        let filter_by_ms_level = config["filter_by_ms_level"].as_u64().map(|v| v as u8);
        let mz_range_min = config["mz_range_min"].as_f64();
        let mz_range_max = config["mz_range_max"].as_f64();
        let rt_range_min = config["rt_range_min"].as_f64();
        let rt_range_max = config["rt_range_max"].as_f64();
        let intensity_threshold = config["intensity_threshold"].as_f64().unwrap_or(0.0);
        let output_path = config["output_path"].as_str();
        
        log::info!("📊 SpectroTsvExporter: 解析参数 - output_path: {:?}", output_path);

        let mut content = String::new();
        
        // Add metadata section
        if include_metadata {
            content.push_str("# Spectra Data Export\n");
            content.push_str(&format!("# Export Time: {}\n", helpers::generate_timestamp()));
            content.push_str(&format!("# Total Spectra: {}\n", data.spectra.len()));
            content.push_str(&format!("# Total Data Points: {}\n", self.count_total_data_points(data)));
            content.push_str("#\n");
        }

        // Build header - 只输出纯粹的三列
        if include_header {
            content.push_str("mz\tdt\tintensity\n");
        }

        // Process each spectrum
        let mut total_points = 0;
        for spectrum in &data.spectra {
            // Apply MS level filter
            if let Some(ms_level) = filter_by_ms_level {
                if spectrum.ms_level() != ms_level {
                    continue;
                }
            }

            // Get ion mobility (drift time)
            let drift_time = spectrum.ion_mobility().unwrap_or(0.0);
            
            // Get retention time for RT range filtering
            let retention_time = spectrum.start_time();

            // Apply RT range filter
            if let Some(min) = rt_range_min {
                if retention_time < min {
                    continue;
                }
            }
            if let Some(max) = rt_range_max {
                if retention_time > max {
                    continue;
                }
            }

            // Process each peak in the spectrum
            let peaks = spectrum.peaks();
            for peak in peaks.iter() {
                let mz = peak.mz();
                let intensity = peak.intensity() as f64;

                // Apply m/z range filter
                if let Some(min) = mz_range_min {
                    if mz < min {
                        continue;
                    }
                }
                if let Some(max) = mz_range_max {
                    if mz > max {
                        continue;
                    }
                }

                // Apply intensity threshold filter - 过滤强度为0的点
                if intensity <= intensity_threshold {
                    continue;
                }

                // Build data row - 只输出纯粹的三列
                let row = format!(
                    "{}\t{}\t{}\n",
                    helpers::format_float(mz, decimal_precision),
                    helpers::format_float(drift_time, decimal_precision),
                    helpers::format_float(intensity, decimal_precision)
                );

                content.push_str(&row);
                total_points += 1;
            }
        }

        let mut metadata = helpers::create_export_metadata(
            self.name(),
            data.spectra.len(),
            total_points,
            &export_config,
        );
        metadata.insert("total_data_points".to_string(), serde_json::json!(total_points));
        metadata.insert("filtered_by_ms_level".to_string(), serde_json::json!(filter_by_ms_level));
        metadata.insert("mz_range".to_string(), serde_json::json!({
            "min": mz_range_min,
            "max": mz_range_max
        }));
        metadata.insert("rt_range".to_string(), serde_json::json!({
            "min": rt_range_min,
            "max": rt_range_max
        }));
        metadata.insert("intensity_threshold".to_string(), serde_json::json!(intensity_threshold));

        // 如果指定了输出路径，直接写入文件
        if let Some(path) = output_path {
            log::info!("📁 SpectroTsvExporter: 准备写入文件到路径: {}", path);
            let filepath = Path::new(path);
            
            // 确保父目录存在
            if let Some(parent) = filepath.parent() {
                log::info!("📁 创建父目录: {:?}", parent);
                fs::create_dir_all(parent)
                    .map_err(|e| {
                        log::error!("❌ 无法创建目录: {}", e);
                        ProcessingError::DataError(format!("无法创建目录: {}", e))
                    })?;
            }
            
            // 写入文件
            log::info!("📝 开始写入文件，内容长度: {} 字节", content.len());
            fs::write(filepath, &content)
                .map_err(|e| {
                    log::error!("❌ 写入文件失败: {}", e);
                    ProcessingError::DataError(format!("无法写入文件 {}: {}", path, e))
                })?;
            
            let file_size = fs::metadata(filepath)
                .map_err(|e| {
                    log::error!("❌ 无法获取文件大小: {}", e);
                    ProcessingError::DataError(format!("无法获取文件大小: {}", e))
                })?
                .len();
            
            log::info!("✅ 文件写入成功，大小: {} 字节", file_size);
            
            metadata.insert("file_size_bytes".to_string(), serde_json::json!(file_size));
            metadata.insert("output_path".to_string(), serde_json::json!(path));
            
            Ok(ExportResult {
                data: content.into_bytes(),
                filename: filepath.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("spectra_data.tsv")
                    .to_string(),
                mime_type: self.mime_type().to_string(),
                metadata,
            })
        } else {
            log::info!("📄 SpectroTsvExporter: 没有指定输出路径，返回数据");
            // 没有指定输出路径，返回数据
            let filename = format!("spectra_data_{}.tsv", helpers::generate_timestamp());
            
            Ok(ExportResult {
                data: content.into_bytes(),
                filename,
                mime_type: self.mime_type().to_string(),
                metadata,
            })
        }
    }
}

impl SpectroTsvExporter {
    /// Count total data points across all spectra
    fn count_total_data_points(&self, data: &DataContainer) -> usize {
        data.spectra.iter()
            .map(|spectrum| spectrum.peaks().len())
            .sum()
    }
}
