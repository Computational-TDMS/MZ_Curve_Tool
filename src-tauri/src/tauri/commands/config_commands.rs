//! 配置管理相关命令

use tauri::State;
use crate::tauri::state::{AppStateManager, ProcessingParams};

// 配置管理结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UserConfig {
    pub processing_params: ProcessingParams,
    pub ui_settings: UiSettings,
    pub export_settings: ExportSettings,
    pub visualization_settings: VisualizationSettings,
    pub last_updated: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct UiSettings {
    pub theme: String, // "light", "dark", "auto"
    pub language: String, // "zh", "en"
    pub window_size: (u32, u32),
    pub window_position: (i32, i32),
    pub auto_save: bool,
    pub auto_save_interval: u32, // 分钟
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ExportSettings {
    pub default_format: String, // "tsv", "json", "plot"
    pub default_directory: String,
    pub include_metadata: bool,
    pub decimal_precision: usize,
    pub auto_export: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct VisualizationSettings {
    pub default_plot_type: String, // "line", "scatter", "bar"
    pub color_scheme: String,
    pub show_grid: bool,
    pub show_legend: bool,
    pub auto_scale: bool,
    pub peak_highlighting: bool,
}

// 配置管理结果结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ConfigResult {
    pub success: bool,
    pub message: String,
    pub config: Option<UserConfig>,
}

/// 获取应用状态
#[tauri::command]
pub fn get_app_state(state: State<'_, AppStateManager>) -> Result<crate::tauri::state::AppState, String> {
    let app_state = state.lock();
    Ok(app_state.clone())
}

/// 更新处理参数
#[tauri::command]
pub fn update_processing_params(
    params: ProcessingParams,
    state: State<'_, AppStateManager>
) -> Result<(), String> {
    let mut app_state = state.lock();
    app_state.set_processing_params(params);
    app_state.add_message("info", "参数更新", "处理参数已更新");
    Ok(())
}

/// 获取处理状态
#[tauri::command]
pub fn get_processing_status(state: State<'_, AppStateManager>) -> Result<crate::tauri::state::ProcessingStatus, String> {
    let app_state = state.lock();
    Ok(app_state.processing_status.clone())
}

/// 保存用户配置
#[tauri::command]
pub async fn save_config(config: UserConfig, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<ConfigResult, String> {
    log::info!("💾 开始保存用户配置");
    
    let mut app_state = state.lock();
    
    app_state.add_message("info", "配置保存", "开始保存用户配置");
    
    // 创建带时间戳的配置
    let config_with_timestamp = UserConfig {
        last_updated: chrono::Utc::now().to_rfc3339(),
        ..config
    };
    
    // 获取配置目录
    let config_dir = dirs::config_dir()
        .ok_or("无法获取配置目录")?
        .join("mz_curve_gui");
    
    // 创建配置目录（如果不存在）
    std::fs::create_dir_all(&config_dir)
        .map_err(|e| format!("无法创建配置目录: {}", e))?;
    
    let config_file = config_dir.join("config.json");
    
    // 序列化配置为JSON
    let config_json = serde_json::to_string_pretty(&config_with_timestamp)
        .map_err(|e| format!("配置序列化失败: {}", e))?;
    
    // 保存到文件
    std::fs::write(&config_file, config_json)
        .map_err(|e| format!("无法写入配置文件: {}", e))?;
    
    log::info!("✅ 配置已保存到: {:?}", config_file);
    app_state.add_message("success", "配置保存完成", "用户配置已保存");
    
    Ok(ConfigResult {
        success: true,
        message: "配置保存成功".to_string(),
        config: Some(config_with_timestamp),
    })
}

/// 加载用户配置
#[tauri::command]
pub async fn load_config(_app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<ConfigResult, String> {
    log::info!("📂 开始加载用户配置");
    
    let mut app_state = state.lock();
    
    app_state.add_message("info", "配置加载", "开始加载用户配置");
    
    // 获取配置目录和文件路径
    let config_dir = dirs::config_dir()
        .ok_or("无法获取配置目录")?
        .join("mz_curve_gui");
    
    let config_file = config_dir.join("config.json");
    
    // 尝试加载配置文件
    if config_file.exists() {
        log::info!("📄 找到配置文件: {:?}", config_file);
        
        // 读取配置文件
        let config_content = std::fs::read_to_string(&config_file)
            .map_err(|e| format!("无法读取配置文件: {}", e))?;
        
        // 反序列化配置
        let loaded_config: UserConfig = serde_json::from_str(&config_content)
            .map_err(|e| format!("配置文件格式错误: {}", e))?;
        
        log::info!("✅ 配置加载成功");
        app_state.add_message("success", "配置加载完成", "用户配置已加载");
        
        Ok(ConfigResult {
            success: true,
            message: "配置加载成功".to_string(),
            config: Some(loaded_config),
        })
    } else {
        log::info!("📄 配置文件不存在，使用默认配置");
        app_state.add_message("info", "配置加载", "使用默认配置");
        
        // 创建默认配置
        let default_config = UserConfig {
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
            ui_settings: UiSettings {
                theme: "light".to_string(),
                language: "zh".to_string(),
                window_size: (1200, 800),
                window_position: (100, 100),
                auto_save: true,
                auto_save_interval: 5,
            },
            export_settings: ExportSettings {
                default_format: "tsv".to_string(),
                default_directory: ".".to_string(),
                include_metadata: true,
                decimal_precision: 6,
                auto_export: false,
            },
            visualization_settings: VisualizationSettings {
                default_plot_type: "line".to_string(),
                color_scheme: "default".to_string(),
                show_grid: true,
                show_legend: true,
                auto_scale: true,
                peak_highlighting: true,
            },
            last_updated: chrono::Utc::now().to_rfc3339(),
        };
        
        app_state.add_message("success", "配置加载完成", "用户配置已加载");
        
        Ok(ConfigResult {
            success: true,
            message: "配置加载成功".to_string(),
            config: Some(default_config),
        })
    }
}

/// 重置为默认配置
#[tauri::command]
pub async fn reset_config(_app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<ConfigResult, String> {
    let mut app_state = state.lock();
    
    app_state.add_message("info", "配置重置", "开始重置为默认配置");
    
    // 创建默认配置
    let default_config = UserConfig {
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
        ui_settings: UiSettings {
            theme: "light".to_string(),
            language: "zh".to_string(),
            window_size: (1200, 800),
            window_position: (100, 100),
            auto_save: true,
            auto_save_interval: 5,
        },
        export_settings: ExportSettings {
            default_format: "tsv".to_string(),
            default_directory: ".".to_string(),
            include_metadata: true,
            decimal_precision: 6,
            auto_export: false,
        },
        visualization_settings: VisualizationSettings {
            default_plot_type: "line".to_string(),
            color_scheme: "default".to_string(),
            show_grid: true,
            show_legend: true,
            auto_scale: true,
            peak_highlighting: true,
        },
        last_updated: chrono::Utc::now().to_rfc3339(),
    };
    
    app_state.add_message("success", "配置重置完成", "已重置为默认配置");
    
    Ok(ConfigResult {
        success: true,
        message: "配置重置成功".to_string(),
        config: Some(default_config),
    })
}

/// 获取默认处理参数
#[tauri::command]
pub async fn get_default_params(_app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<ProcessingParams, String> {
    let mut app_state = state.lock();
    
    app_state.add_message("info", "获取默认参数", "获取默认处理参数");
    
    let default_params = ProcessingParams {
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
    };
    
    Ok(default_params)
}
