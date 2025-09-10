//! Tauri应用状态管理
//! 使用Tauri的状态管理来统一管理应用状态

use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::Emitter;

/// 应用状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// 当前处理状态
    pub processing_status: ProcessingStatus,
    /// 当前文件信息
    pub current_files: Vec<String>,
    /// 处理参数
    pub processing_params: ProcessingParams,
    /// 处理结果
    pub processing_result: Option<ProcessingResult>,
    /// 多曲线数据
    pub multi_curve_data: Option<MultiCurveData>,
    /// 数据范围
    pub data_ranges: Option<DataRanges>,
    /// 日志消息
    pub messages: Vec<LogMessage>,
}

/// 处理状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessingStatus {
    Idle,
    Loading,
    Extracting,
    Analyzing,
    Exporting,
    Error(String),
    Success,
}

/// 处理参数
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessingParams {
    pub mz_min: f64,
    pub mz_max: f64,
    pub rt_min: f64,
    pub rt_max: f64,
    pub ms_level: u8,
    pub mode: String, // "dt" or "tic"
    pub sensitivity: f64,
    pub fit_type: String,
    pub max_iterations: u32,
    pub peak_detection_threshold: f64,
    pub peak_fitting_method: String,
    pub baseline_correction_method: String,
    pub smoothing_enabled: bool,
    pub smoothing_method: String,
    pub smoothing_window_size: u32,
}

/// 处理结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingResult {
    pub success: bool,
    pub data: Option<ProcessingData>,
    pub error: Option<String>,
    pub processing_time: Option<u64>,
}

/// 处理数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingData {
    pub dt_curve: Vec<DTCurvePoint>,
    pub peaks: Vec<PeakInfo>,
    pub visualization_data: VisualizationData,
}

/// DT曲线点
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DTCurvePoint {
    pub drift_time: f64,
    pub intensity: f64,
}

/// 峰值信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakInfo {
    pub center: f64,
    pub amplitude: f64,
    pub width: f64,
    pub area: f64,
    pub rsquared: f64,
}

/// 可视化数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationData {
    pub dt_curve: Vec<DTCurvePoint>,
    pub peaks: Vec<PeakData>,
    pub metadata: ChartMetadata,
}

/// 峰值数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakData {
    pub center: f64,
    pub amplitude: f64,
    pub width: f64,
    pub area: f64,
    pub rsquared: f64,
}

/// 图表元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartMetadata {
    pub total_points: usize,
    pub peak_count: usize,
    pub max_intensity: f64,
    pub min_drift_time: f64,
    pub max_drift_time: f64,
}

/// 多曲线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiCurveData {
    pub success: bool,
    pub curves: Vec<CurveData>,
    pub peaks: Vec<PeakInfo>,
    pub metadata: MultiCurveMetadata,
}

/// 曲线数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveData {
    pub x: Vec<f64>,
    pub y: Vec<f64>,
    pub mode: String,
    pub file_name: String,
    pub color: String,
}

/// 多曲线元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiCurveMetadata {
    pub total_files: usize,
    pub total_points: usize,
    pub peak_count: usize,
    pub max_intensity: f64,
    pub min_drift_time: f64,
    pub max_drift_time: f64,
}

/// 数据范围
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRanges {
    pub mz_min: f64,
    pub mz_max: f64,
    pub rt_min: f64,
    pub rt_max: f64,
    pub ms_levels: Vec<u8>,
}

/// 日志消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    pub id: String,
    pub level: String, // "info", "warning", "error", "success"
    pub title: String,
    pub content: String,
    pub timestamp: String,
}

/// 进度更新
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub current: usize,
    pub total: usize,
    pub message: String,
    pub percentage: f64,
}

/// 状态管理器
pub struct AppStateManager {
    state: Mutex<AppState>,
    file_cache: Mutex<std::collections::HashMap<String, crate::core::data::container::DataContainer>>,
}

impl AppStateManager {
    pub fn new(state: AppState) -> Self {
        Self {
            state: Mutex::new(state),
            file_cache: Mutex::new(std::collections::HashMap::new()),
        }
    }
    
    pub fn lock(&self) -> std::sync::MutexGuard<'_, AppState> {
        self.state.lock().unwrap()
    }
    
    pub fn try_lock(&self) -> Result<std::sync::MutexGuard<'_, AppState>, std::sync::TryLockError<std::sync::MutexGuard<'_, AppState>>> {
        self.state.try_lock()
    }
    
    /// 发送状态更新事件到前端
    pub fn emit_status_update(&self, app_handle: &tauri::AppHandle, status: &ProcessingStatus) {
        let _ = app_handle.emit("status-updated", status);
    }
    
    /// 发送日志消息事件到前端
    pub fn emit_log_message(&self, app_handle: &tauri::AppHandle, message: &LogMessage) {
        let _ = app_handle.emit("log-message", message);
    }
    
    /// 发送进度更新事件到前端
    pub fn emit_progress_update(&self, app_handle: &tauri::AppHandle, current: usize, total: usize, message: &str) {
        let progress = ProgressUpdate {
            current,
            total,
            message: message.to_string(),
            percentage: if total > 0 { (current as f64 / total as f64) * 100.0 } else { 0.0 },
        };
        let _ = app_handle.emit("progress-updated", &progress);
    }
    
    /// 缓存文件数据
    pub fn cache_file(&self, file_path: &str, container: crate::core::data::container::DataContainer) {
        if let Ok(mut cache) = self.file_cache.lock() {
            cache.insert(file_path.to_string(), container);
            log::info!("📦 文件已缓存: {}", file_path);
        }
    }
    
    /// 获取缓存的文件数据
    pub fn get_cached_file(&self, file_path: &str) -> Option<crate::core::data::container::DataContainer> {
        if let Ok(cache) = self.file_cache.lock() {
            if let Some(container) = cache.get(file_path) {
                log::info!("🚀 使用缓存文件: {}", file_path);
                return Some(container.clone());
            }
        }
        None
    }
    
    /// 清除文件缓存
    pub fn clear_file_cache(&self) {
        if let Ok(mut cache) = self.file_cache.lock() {
            cache.clear();
            log::info!("🗑️ 文件缓存已清除");
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            processing_status: ProcessingStatus::Idle,
            current_files: Vec::new(),
            processing_params: ProcessingParams {
                mz_min: 100.0,
                mz_max: 200.0,
                rt_min: 0.0,
                rt_max: 100.0,
                ms_level: 1,
                mode: "dt".to_string(),
                sensitivity: 0.5,
                fit_type: "gaussian".to_string(),
                max_iterations: 100,
                peak_detection_threshold: 0.1,
                peak_fitting_method: "gaussian".to_string(),
                baseline_correction_method: "linear".to_string(),
                smoothing_enabled: false,
                smoothing_method: "moving_average".to_string(),
                smoothing_window_size: 5,
            },
            processing_result: None,
            multi_curve_data: None,
            data_ranges: None,
            messages: Vec::new(),
        }
    }
}

impl AppState {
    /// 添加日志消息
    pub fn add_message(&mut self, level: &str, title: &str, content: &str) {
        let message = LogMessage {
            id: uuid::Uuid::new_v4().to_string(),
            level: level.to_string(),
            title: title.to_string(),
            content: content.to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
        };
        self.messages.push(message.clone());
        
        // 限制消息数量，避免内存泄漏
        if self.messages.len() > 1000 {
            self.messages.drain(0..100);
        }
    }
    
    /// 设置处理状态
    pub fn set_processing_status(&mut self, status: ProcessingStatus) {
        self.processing_status = status;
    }
    
    /// 设置当前文件
    pub fn set_current_files(&mut self, files: Vec<String>) {
        self.current_files = files;
    }
    
    /// 设置处理参数
    pub fn set_processing_params(&mut self, params: ProcessingParams) {
        self.processing_params = params;
    }
    
    /// 设置处理结果
    pub fn set_processing_result(&mut self, result: ProcessingResult) {
        self.processing_result = Some(result);
    }
    
    /// 设置多曲线数据
    pub fn set_multi_curve_data(&mut self, data: MultiCurveData) {
        self.multi_curve_data = Some(data);
    }
    
    /// 设置数据范围
    pub fn set_data_ranges(&mut self, ranges: DataRanges) {
        self.data_ranges = Some(ranges);
    }
    
    /// 清空所有数据
    pub fn reset(&mut self) {
        self.processing_status = ProcessingStatus::Idle;
        self.current_files.clear();
        self.processing_result = None;
        self.multi_curve_data = None;
        self.data_ranges = None;
        self.messages.clear();
    }
}
