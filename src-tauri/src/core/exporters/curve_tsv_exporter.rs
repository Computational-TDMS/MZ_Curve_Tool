use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::core::data::{DataContainer, ProcessingError};
use super::base::{Exporter, ExportResult};

/// 优化的曲线TSV导出器 - 专门用于快速导出曲线数据
pub struct CurveTsvExporter;

#[async_trait]
impl Exporter for CurveTsvExporter {
    fn name(&self) -> &str {
        "curve_tsv_exporter"
    }

    fn description(&self) -> &str {
        "优化的曲线TSV导出器，专门用于快速导出曲线数据到文件夹"
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
                "output_folder": {
                    "type": "string",
                    "description": "输出文件夹路径"
                },
                "include_curve_data": {
                    "type": "boolean",
                    "default": true,
                    "description": "是否包含曲线数据点"
                },
                "include_metadata": {
                    "type": "boolean",
                    "default": true,
                    "description": "是否包含元数据"
                },
                "decimal_precision": {
                    "type": "integer",
                    "default": 6,
                    "minimum": 0,
                    "maximum": 10,
                    "description": "小数精度"
                }
            },
            "required": ["output_folder"]
        })
    }

    async fn export(
        &self,
        data: &DataContainer,
        config: Value,
    ) -> Result<ExportResult, ProcessingError> {
        let output_folder = config["output_folder"]
            .as_str()
            .ok_or_else(|| ProcessingError::ConfigError("output_folder missing".to_string()))?;
        
        let include_curve_data = config["include_curve_data"].as_bool().unwrap_or(true);
        let include_metadata = config["include_metadata"].as_bool().unwrap_or(true);
        let decimal_precision = config["decimal_precision"].as_u64().unwrap_or(6) as usize;

        // 创建输出文件夹
        fs::create_dir_all(output_folder)
            .map_err(|e| ProcessingError::DataError(format!("无法创建输出文件夹: {}", e)))?;

        let mut exported_files = Vec::new();
        let mut total_size = 0;

        // 导出每条曲线到单独的TSV文件
        for (index, curve) in data.curves.iter().enumerate() {
            let filename = format!("curve_{}_{}.tsv", index + 1, sanitize_filename(&curve.curve_type));
            let filepath = Path::new(output_folder).join(&filename);
            
            let mut content = String::new();
            
            // 添加元数据头部
            if include_metadata {
                content.push_str(&format!("# Curve: {}\n", curve.id));
                content.push_str(&format!("# Type: {}\n", curve.curve_type));
                content.push_str(&format!("# X Label: {} ({})\n", curve.x_label, curve.x_unit));
                content.push_str(&format!("# Y Label: {} ({})\n", curve.y_label, curve.y_unit));
                content.push_str(&format!("# Data Points: {}\n", curve.point_count));
                
                if let (Some(min), Some(max)) = (curve.x_values.first(), curve.x_values.last()) {
                    content.push_str(&format!("# X Range: {:.6} - {:.6}\n", min, max));
                }
                
                if let (Some(min), Some(max)) = (curve.y_values.first(), curve.y_values.last()) {
                    content.push_str(&format!("# Y Range: {:.6} - {:.6}\n", min, max));
                }
                
                // 添加m/z范围信息
                if let Some((mz_min, mz_max)) = curve.mz_range {
                    content.push_str(&format!("# M/Z Range: {:.6} - {:.6}\n", mz_min, mz_max));
                }
                
                content.push_str("#\n");
            }
            
            // 添加表头
            content.push_str(&format!("{}\t{}\n", curve.x_label, curve.y_label));
            
            // 添加数据点
            if include_curve_data {
                for (x, y) in curve.x_values.iter().zip(curve.y_values.iter()) {
                    content.push_str(&format!("{:.prec$}\t{:.prec$}\n", 
                        x, y, prec = decimal_precision));
                }
            }
            
            // 写入文件
            fs::write(&filepath, content)
                .map_err(|e| ProcessingError::DataError(format!("无法写入文件 {}: {}", filename, e)))?;
            
            let file_size = fs::metadata(&filepath)
                .map_err(|e| ProcessingError::DataError(format!("无法获取文件大小: {}", e)))?
                .len();
            
            exported_files.push(filename);
            total_size += file_size;
        }
        
        // 创建汇总文件
        let summary_filename = "export_summary.txt";
        let summary_path = Path::new(output_folder).join(summary_filename);
        let mut summary_content = String::new();
        
        summary_content.push_str(&format!("导出时间: {}\n", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC")));
        summary_content.push_str(&format!("导出曲线数量: {}\n", data.curves.len()));
        summary_content.push_str(&format!("导出文件数量: {}\n", exported_files.len()));
        summary_content.push_str(&format!("总文件大小: {} bytes\n", total_size));
        summary_content.push_str("\n导出的文件:\n");
        
        for file in &exported_files {
            summary_content.push_str(&format!("  - {}\n", file));
        }
        
        summary_content.push_str("\n曲线信息:\n");
        for (index, curve) in data.curves.iter().enumerate() {
            summary_content.push_str(&format!("  {}: {} ({} 个数据点)\n", 
                index + 1, curve.curve_type, curve.point_count));
        }
        
        fs::write(&summary_path, summary_content)
            .map_err(|e| ProcessingError::DataError(format!("无法写入汇总文件: {}", e)))?;
        
        // 创建元数据文件
        if include_metadata {
            let metadata_filename = "metadata.json";
            let metadata_path = Path::new(output_folder).join(metadata_filename);
            
            let mut metadata = HashMap::new();
            metadata.insert("export_time".to_string(), serde_json::json!(chrono::Utc::now().to_rfc3339()));
            metadata.insert("curve_count".to_string(), serde_json::json!(data.curves.len()));
            metadata.insert("exported_files".to_string(), serde_json::json!(exported_files));
            metadata.insert("total_size_bytes".to_string(), serde_json::json!(total_size));
            
            let curves_metadata: Vec<serde_json::Value> = data.curves.iter().map(|curve| {
                serde_json::json!({
                    "id": curve.id,
                    "type": curve.curve_type,
                    "x_label": curve.x_label,
                    "y_label": curve.y_label,
                    "x_unit": curve.x_unit,
                    "y_unit": curve.y_unit,
                    "point_count": curve.point_count,
"mz_min": curve.mz_range.map(|r| r.0),
                    "mz_max": curve.mz_range.map(|r| r.1)
                })
            }).collect();
            
            metadata.insert("curves".to_string(), serde_json::json!(curves_metadata));
            
            let metadata_json = serde_json::to_string_pretty(&metadata)
                .map_err(|e| ProcessingError::DataError(format!("无法序列化元数据: {}", e)))?;
            
            fs::write(&metadata_path, metadata_json)
                .map_err(|e| ProcessingError::DataError(format!("无法写入元数据文件: {}", e)))?;
        }
        
        let mut result_metadata = HashMap::new();
        result_metadata.insert("exported_files".to_string(), serde_json::json!(exported_files));
        result_metadata.insert("total_size_bytes".to_string(), serde_json::json!(total_size));
        result_metadata.insert("output_folder".to_string(), serde_json::json!(output_folder));
        
        Ok(ExportResult {
            data: format!("导出完成，共 {} 个文件，总大小 {} bytes", exported_files.len(), total_size).into_bytes(),
            filename: format!("curve_export_{}", chrono::Utc::now().format("%Y%m%d_%H%M%S")),
            mime_type: self.mime_type().to_string(),
            metadata: result_metadata,
        })
    }
}


/// 清理文件名，移除非法字符
fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '_' || c == '-' { c } else { '_' })
        .collect()
}
