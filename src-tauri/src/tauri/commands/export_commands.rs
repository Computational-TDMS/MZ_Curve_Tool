//! 数据导出相关命令

use tauri::State;
use crate::tauri::state::AppStateManager;
use crate::core::loaders::mzdata_loader::DataLoader;
use super::{ExportParams, ExportResultInfo};

/// 快速导出曲线数据到文件夹
#[tauri::command]
pub async fn export_curves_to_folder(
    output_folder: String,
    container: crate::core::data::container::SerializableDataContainer,
    _app: tauri::AppHandle,
    state: State<'_, AppStateManager>
) -> Result<ExportResultInfo, String> {
    {
        let mut app_state = state.lock();
        app_state.add_message("info", "曲线导出", &format!("开始导出曲线数据到文件夹: {}", output_folder));
    }
    
    // 使用优化的曲线TSV导出器
    let export_manager = crate::core::exporters::export_manager::ExportManager::new();
    
    // 准备导出配置
    let export_config = serde_json::json!({
        "output_folder": output_folder,
        "include_curve_data": true,
        "include_metadata": true,
        "decimal_precision": 6
    });
    
    // 将SerializableDataContainer转换为DataContainer
    let data_container: crate::core::data::DataContainer = container.into();
    
    if data_container.curves.is_empty() {
        {
            let mut app_state = state.lock();
            app_state.add_message("error", "导出失败", "没有可导出的曲线数据");
        }
        return Err("没有可导出的曲线数据".to_string());
    }
    
    // 执行导出
    match export_manager.export("curve_tsv", &data_container, export_config).await {
        Ok(result) => {
            let mut app_state = state.lock();
            app_state.add_message("success", "曲线导出完成", &format!("成功导出到文件夹: {}", output_folder));
            
            Ok(ExportResultInfo {
                success: true,
                message: format!("成功导出 {} 个文件到文件夹: {}", 
                    result.metadata.get("exported_files").and_then(|v| v.as_array()).map(|arr| arr.len()).unwrap_or(0),
                    output_folder),
                filename: result.filename,
                file_size: result.metadata.get("total_size_bytes").and_then(|v| v.as_u64()).unwrap_or(0) as usize,
                mime_type: result.mime_type,
            })
        },
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "导出失败", &format!("错误: {}", e));
            }
            Err(format!("导出失败: {}", e))
        }
    }
}

/// 导出TSV数据
#[tauri::command]
pub async fn export_tsv(params: ExportParams, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<ExportResultInfo, String> {
    {
        let mut app_state = state.lock();
        app_state.add_message("info", "TSV导出", &format!("开始导出TSV数据: {}", params.file_path));
    }
    
    // 使用真实的ExportManager进行导出
    let export_manager = crate::core::exporters::export_manager::ExportManager::new();
    
    // 准备导出配置
    let export_config = serde_json::json!({
        "output_path": params.output_path,
        "include_curves": params.include_curves,
        "include_peaks": params.include_peaks,
        "include_metadata": params.include_metadata
    });
    
    // 创建数据容器（这里需要从当前状态获取数据）
    let mut container = crate::core::data::DataContainer::new();
    
    // 从应用状态获取当前处理的数据
    let current_files = {
        let app_state = state.lock();
        app_state.current_files.clone()
    };
    
    if !current_files.is_empty() {
        // 加载当前文件的数据
        match DataLoader::load_from_file(&current_files[0]) {
            Ok(data) => container = data,
            Err(e) => {
                {
                    let mut app_state = state.lock();
                    app_state.add_message("error", "导出失败", &format!("无法加载数据: {}", e));
                }
                return Err(format!("无法加载数据: {}", e));
            }
        }
    }
    
    // 执行导出
    match export_manager.export("tsv", &container, export_config).await {
        Ok(result) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("success", "TSV导出完成", &format!("文件已导出: {}", result.filename));
            }
            
            Ok(ExportResultInfo {
                success: true,
                filename: result.filename,
                file_size: result.data.len(),
                mime_type: "text/tab-separated-values".to_string(),
                message: "TSV导出成功".to_string(),
            })
        }
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "TSV导出失败", &format!("错误: {}", e));
            }
            Err(format!("TSV导出失败: {}", e))
        }
    }
}

/// 导出JSON数据
#[tauri::command]
pub async fn export_json(_params: ExportParams, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<ExportResultInfo, String> {
    {
        let mut app_state = state.lock();
        app_state.add_message("error", "JSON导出失败", "JSON导出器尚未实现");
    }
    Err("JSON导出器尚未实现".to_string())
}

/// 导出图表数据
#[tauri::command]
pub async fn export_plot(params: ExportParams, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<ExportResultInfo, String> {
    {
        let mut app_state = state.lock();
        app_state.add_message("info", "Plotly图表导出", &format!("开始导出Plotly图表数据: {}", params.file_path));
    }
    
    // 使用真实的ExportManager进行导出
    let export_manager = crate::core::exporters::export_manager::ExportManager::new();
    
    // 准备Plotly导出配置
    let export_config = serde_json::json!({
        "output_path": params.output_path,
        "include_curves": params.include_curves,
        "include_peaks": params.include_peaks,
        "include_metadata": params.include_metadata,
        "chart_type": "combined",
        "show_peaks": true,
        "show_fit": false,
        "title": "IMS Data Visualization",
        "x_axis_title": "Drift Time (ms)",
        "y_axis_title": "Intensity",
        "width": 1000,
        "height": 600
    });
    
    // 创建数据容器
    let mut container = crate::core::data::DataContainer::new();
    
    // 从应用状态获取当前处理的数据
    let current_files = {
        let app_state = state.lock();
        app_state.current_files.clone()
    };
    
    if !current_files.is_empty() {
        match DataLoader::load_from_file(&current_files[0]) {
            Ok(data) => container = data,
            Err(e) => {
                {
                    let mut app_state = state.lock();
                    app_state.add_message("error", "导出失败", &format!("无法加载数据: {}", e));
                }
                return Err(format!("无法加载数据: {}", e));
            }
        }
    }
    
    // 执行Plotly导出
    match export_manager.export("plotly", &container, export_config).await {
        Ok(result) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("success", "Plotly图表导出完成", &format!("文件已导出: {}", result.filename));
            }
            
            Ok(ExportResultInfo {
                success: true,
                filename: result.filename,
                file_size: result.data.len(),
                mime_type: "application/json".to_string(),
                message: "Plotly图表导出成功".to_string(),
            })
        }
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "Plotly图表导出失败", &format!("错误: {}", e));
            }
            Err(format!("Plotly图表导出失败: {}", e))
        }
    }
}

// 光谱数据导出参数结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpectroExportParams {
    pub file_path: String,
    pub output_path: Option<String>,
    pub include_header: bool,
    pub decimal_precision: usize,
    pub include_metadata: bool,
    pub filter_by_ms_level: Option<u8>,
    pub mz_range_min: Option<f64>,
    pub mz_range_max: Option<f64>,
    pub rt_range_min: Option<f64>,
    pub rt_range_max: Option<f64>,
    pub intensity_threshold: f64,
}

/// 导出光谱数据为TSV格式 (mz, dt, intensity)
#[tauri::command]
pub async fn export_spectro_tsv(
    params: SpectroExportParams,
    _app: tauri::AppHandle,
    state: State<'_, AppStateManager>
) -> Result<ExportResultInfo, String> {
    log::info!("📊 开始导出光谱数据为TSV格式: {}", params.file_path);
    log::info!("📊 导出参数: {:?}", params);
    
    {
        let mut app_state = state.lock();
        app_state.add_message("info", "光谱数据导出", &format!("开始导出光谱数据: {}", params.file_path));
    }
    
    // 使用真实的ExportManager进行导出
    let export_manager = crate::core::exporters::export_manager::ExportManager::new();
    
    // 准备导出配置
    let mut export_config = serde_json::json!({
        "include_header": params.include_header,
        "decimal_precision": params.decimal_precision,
        "include_metadata": params.include_metadata,
        "filter_by_ms_level": params.filter_by_ms_level,
        "mz_range_min": params.mz_range_min,
        "mz_range_max": params.mz_range_max,
        "rt_range_min": params.rt_range_min,
        "rt_range_max": params.rt_range_max,
        "intensity_threshold": params.intensity_threshold
    });
    
    // 如果指定了输出路径，添加到配置中
    if let Some(output_path) = &params.output_path {
        export_config["output_path"] = serde_json::json!(output_path);
    }
    
    // 加载数据
    let container = match DataLoader::load_from_file(&params.file_path) {
        Ok(container) => container,
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "光谱数据导出失败", &format!("无法加载文件: {}", e));
            }
            return Err(format!("无法加载文件: {}", e));
        }
    };
    
    if container.spectra.is_empty() {
        {
            let mut app_state = state.lock();
            app_state.add_message("error", "光谱数据导出失败", "没有可导出的光谱数据");
        }
        return Err("没有可导出的光谱数据".to_string());
    }
    
    // 执行导出
    match export_manager.export("spectro_tsv", &container, export_config).await {
        Ok(result) => {
            {
                let mut app_state = state.lock();
                if let Some(output_path) = &params.output_path {
                    app_state.add_message("success", "光谱数据导出完成", &format!("文件已保存到: {}", output_path));
                } else {
                    app_state.add_message("success", "光谱数据导出完成", &format!("文件已导出: {}", result.filename));
                }
            }
            
            Ok(ExportResultInfo {
                success: true,
                filename: result.filename,
                file_size: result.data.len(),
                mime_type: "text/tab-separated-values".to_string(),
                message: if params.output_path.is_some() {
                    format!("光谱数据TSV导出成功，已保存到: {}", params.output_path.unwrap())
                } else {
                    "光谱数据TSV导出成功".to_string()
                },
            })
        }
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "光谱数据导出失败", &format!("错误: {}", e));
            }
            Err(format!("光谱数据TSV导出失败: {}", e))
        }
    }
}
