//! Tauri命令模块
//! 包含所有前端调用的后端命令

pub mod file_commands;
pub mod curve_commands;
pub mod peak_commands;
pub mod export_commands;
pub mod config_commands;
pub mod visualization_commands;
pub mod processing_commands;
pub mod peak_processing_commands;

// 重新导出所有命令
pub use file_commands::*;
pub use curve_commands::*;
pub use peak_commands::*;
pub use export_commands::*;
pub use config_commands::*;
pub use visualization_commands::*;
pub use processing_commands::*;
pub use peak_processing_commands::*;

// 公共结构体定义
use serde::{Deserialize, Serialize};

// 文件信息结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub name: String,
    pub size: u64,
    pub format: String,
    pub is_valid: bool,
    pub spectra_count: Option<usize>,
    pub data_ranges: Option<DataRanges>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRanges {
    pub rt_min: f64,
    pub rt_max: f64,
    pub mz_min: f64,
    pub mz_max: f64,
    pub ms_levels: Vec<u8>,
}

// 验证结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub message: String,
    pub spectra_count: Option<usize>,
    pub file_size: Option<u64>,
    pub data_ranges: Option<DataRanges>,
}

// 曲线提取参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveExtractionParams {
    pub file_path: String,
    pub mz_range: String,
    pub rt_range: String,
    pub ms_level: u8,
    pub curve_type: String, // "dt", "tic", "xic"
}

// 峰检测参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakDetectionParams {
    pub detection_method: String,
    pub sensitivity: f64,
    pub threshold_multiplier: f64,
    pub min_peak_width: f64,
    pub max_peak_width: f64,
}

// 峰拟合参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakFittingParams {
    pub fitting_method: String,
    pub overlapping_method: Option<String>,
    pub fit_quality_threshold: f64,
    pub max_iterations: u32,
}

// 峰分析参数（保留向后兼容）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakAnalysisParams {
    pub curve_data: crate::core::state::CurveData,
    pub detection_method: String,
    pub fitting_method: String,
    pub overlapping_method: Option<String>,
    pub sensitivity: f64,
    pub threshold_multiplier: f64,
    pub min_peak_width: f64,
    pub max_peak_width: f64,
}

// 峰分析结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakAnalysisResult {
    pub success: bool,
    pub peaks_tsv: String, // 峰数据TSV
    pub fitted_curve_tsv: String, // 拟合曲线TSV
    pub peak_count: usize,
    pub processing_time: u64,
    pub error: Option<String>,
}

// 批量处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProcessingResult {
    pub success: bool,
    pub processed_files: Vec<String>,
    pub failed_files: Vec<String>,
    pub total_curves: usize,
    pub total_peaks: usize,
    pub processing_time: u64,
    pub error: Option<String>,
}

// 进度更新事件
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub percentage: f64,
}

// 导出结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResultInfo {
    pub success: bool,
    pub filename: String,
    pub file_size: usize,
    pub mime_type: String,
    pub message: String,
}

// 导出参数结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportParams {
    pub file_path: String,
    pub export_format: String, // "tsv", "json", "plot"
    pub output_path: Option<String>,
    pub include_curves: bool,
    pub include_peaks: bool,
    pub include_metadata: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveDisplayData {
    pub id: String,
    pub curve_type: String,
    pub x_label: String,
    pub y_label: String,
    pub x_unit: String,
    pub y_unit: String,
    pub point_count: usize,
    pub x_values: Vec<f64>,
    pub y_values: Vec<f64>,
    pub mz_min: Option<f64>,
    pub mz_max: Option<f64>,
}
