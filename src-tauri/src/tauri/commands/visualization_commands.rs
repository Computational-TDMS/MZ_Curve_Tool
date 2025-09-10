//! 可视化相关命令

use tauri::State;
use crate::tauri::state::AppStateManager;
use crate::core::loaders::mzdata_loader::DataLoader;
use uuid::Uuid;

// 可视化参数结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlotGenerationParams {
    pub file_path: String,
    pub plot_type: String, // "line", "scatter", "bar", "heatmap"
    pub data_type: String, // "tic", "xic", "dt", "peaks"
    pub mz_range: Option<(f64, f64)>,
    pub rt_range: Option<(f64, f64)>,
    pub show_peaks: bool,
    pub show_baseline: bool,
    pub color_scheme: String,
    pub title: Option<String>,
}

// 可视化结果结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlotData {
    pub plot_id: String,
    pub plot_type: String,
    pub data: serde_json::Value, // Plotly数据格式
    pub layout: serde_json::Value, // Plotly布局
    pub config: serde_json::Value, // Plotly配置
    pub metadata: PlotMetadata,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PlotMetadata {
    pub title: String,
    pub x_axis_label: String,
    pub y_axis_label: String,
    pub data_points: usize,
    pub generated_at: String,
    pub file_path: String,
}

// 可视化结果结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisualizationResult {
    pub success: bool,
    pub plot_data: Option<PlotData>,
    pub message: String,
}

/// 生成图表数据
#[tauri::command]
pub async fn generate_plot(params: PlotGenerationParams, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<VisualizationResult, String> {
    {
        let mut app_state = state.lock();
        app_state.add_message("info", "Plotly图表生成", &format!("开始生成Plotly图表: {} - {}", params.file_path, params.plot_type));
    }
    
    // 生成唯一的图表ID
    let plot_id = format!("plot_{}", Uuid::new_v4());
    
    // 使用真实的ExportManager生成Plotly数据
    let export_manager = crate::core::exporters::export_manager::ExportManager::new();
    
    // 加载数据
    let container = match DataLoader::load_from_file(&params.file_path) {
        Ok(container) => container,
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "图表生成失败", &format!("无法加载文件: {}", e));
            }
            return Err(format!("无法加载文件: {}", e));
        }
    };
    
    // 准备Plotly导出配置
    let export_config = serde_json::json!({
        "include_curves": true,
        "include_peaks": params.show_peaks,
        "include_metadata": true,
        "chart_type": params.plot_type,
        "show_peaks": params.show_peaks,
        "show_fit": false,
        "title": params.title.clone().unwrap_or_else(|| "IMS Data Visualization".to_string()),
        "x_axis_title": "Drift Time (ms)",
        "y_axis_title": "Intensity",
        "width": 1000,
        "height": 600
    });
    
    // 生成Plotly数据
    match export_manager.export("plotly", &container, export_config).await {
        Ok(result) => {
            // 解析Plotly JSON数据
            let plotly_json: serde_json::Value = match serde_json::from_slice(&result.data) {
                Ok(json) => json,
                Err(e) => {
                    {
                        let mut app_state = state.lock();
                        app_state.add_message("error", "图表生成失败", &format!("JSON解析失败: {}", e));
                    }
                    return Err(format!("JSON解析失败: {}", e));
                }
            };
            
            let plot_data = PlotData {
                plot_id: plot_id.clone(),
                plot_type: params.plot_type.clone(),
                data: plotly_json["data"].clone(),
                layout: plotly_json["layout"].clone(),
                config: plotly_json["config"].clone(),
                metadata: PlotMetadata {
                    title: params.title.clone().unwrap_or_else(|| "IMS Data Visualization".to_string()),
                    x_axis_label: "Drift Time (ms)".to_string(),
                    y_axis_label: "Intensity".to_string(),
                    data_points: container.curves.iter().map(|c| c.point_count).sum(),
                    generated_at: chrono::Utc::now().to_rfc3339(),
                    file_path: params.file_path.clone(),
                },
            };
            
            {
                let mut app_state = state.lock();
                app_state.add_message("success", "Plotly图表生成完成", &format!("图表 {} 已生成", plot_id));
            }
            
            Ok(VisualizationResult {
                success: true,
                plot_data: Some(plot_data),
                message: "Plotly图表生成成功".to_string(),
            })
        }
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "图表生成失败", &format!("错误: {}", e));
            }
            Err(format!("图表生成失败: {}", e))
        }
    }
}

/// 更新图表数据
#[tauri::command]
pub async fn update_plot(plot_id: String, _new_data: serde_json::Value, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<VisualizationResult, String> {
    log::info!("📊 开始更新图表: {}", plot_id);
    
    let mut app_state = state.lock();
    
    app_state.add_message("info", "图表更新", &format!("开始更新图表: {}", plot_id));
    
    // 这里应该实现真实的图表更新逻辑
    // 例如：从内存中查找图表，更新数据，重新渲染等
    log::info!("🔄 图表更新功能尚未实现");
    
    app_state.add_message("error", "图表更新失败", "图表更新功能尚未实现");
    
    Err("图表更新功能尚未实现".to_string())
}

/// 导出图表为图片
#[tauri::command]
pub async fn export_plot_image(plot_id: String, format: String, output_path: String, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<super::ExportResultInfo, String> {
    log::info!("📸 开始导出图表为图片: {} -> {}", plot_id, output_path);
    
    let mut app_state = state.lock();
    
    app_state.add_message("info", "图表导出", &format!("开始导出图表 {} 为 {} 格式", plot_id, format));
    
    // 这里应该实现真实的图表导出逻辑
    // 例如：使用Plotly的导出功能，或者调用系统截图API
    log::info!("🔄 图表导出功能尚未实现");
    
    app_state.add_message("error", "图表导出失败", "图表导出功能尚未实现");
    
    Err("图表导出功能尚未实现".to_string())
}

/// 获取图表配置
#[tauri::command]
pub async fn get_plot_config(plot_id: String, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<serde_json::Value, String> {
    log::info!("⚙️ 开始获取图表配置: {}", plot_id);
    
    let mut app_state = state.lock();
    
    app_state.add_message("info", "获取图表配置", &format!("获取图表 {} 的配置", plot_id));
    
    // 这里应该实现真实的图表配置获取逻辑
    // 例如：从内存中的图表管理器获取配置
    log::info!("🔄 图表配置获取功能尚未实现");
    
    app_state.add_message("error", "获取图表配置失败", "图表配置获取功能尚未实现");
    
    Err("图表配置获取功能尚未实现".to_string())
}
