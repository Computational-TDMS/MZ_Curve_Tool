//! MZ Curve GUI - Tauri应用库
//! 提供质谱数据处理的前后端通信接口

// 模块声明
pub mod tauri;
pub mod core;

use crate::tauri::state::{AppState, AppStateManager};
use crate::tauri::commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    ::tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .manage(AppStateManager::new(AppState::default()))
        .invoke_handler(::tauri::generate_handler![
            // 文件操作API
            load_file,
            validate_file,
            clear_file_cache,
            // 数据处理API
            extract_curve,
            analyze_peaks,
            batch_process_files,
            // 流水线API - 暂时注释掉，因为命令不存在
            // detect_peaks,
            // fit_peaks,
            // enhance_peaks,
            // reconstruct_curves,
            // baseline_correction_pipeline,
            // execute_pipeline,
            // 状态管理API
            get_app_state,
            update_processing_params,
            get_processing_status,
            // 数据导出API
            get_curve_data_for_display,
            export_curves_to_folder,
            export_tsv,
            export_json,
            export_plot,
            export_spectro_tsv,
            // 高级处理API
            baseline_correction,
            overlapping_peaks,
            smooth_data,
            noise_reduction,
            // 配置管理API
            save_config,
            load_config,
            reset_config,
            get_default_params,
            // 可视化API
            generate_plot,
            update_plot,
            export_plot_image,
            get_plot_config,
            // 峰处理工作流API
            init_peak_processing_controller,
            process_peaks,
            get_available_components,
            get_available_strategies,
            get_component_info,
            validate_config,
            get_config_schema,
            // 系统信息API (暂时注释掉，因为命令不存在)
            // get_system_info,
            // get_memory_usage,
            // get_disk_space,
        ])
        .run(::tauri::generate_context!())
        .expect("error while running tauri application");
}
