//! 文件操作相关命令

use tauri::State;
use crate::tauri::state::{AppStateManager, ProcessingStatus};
use crate::core::loaders::mzdata_loader::DataLoader;
use mzdata::prelude::SpectrumLike;
use super::{FileInfo, ValidationResult, DataRanges};

/// 步骤1: 加载文件并获取基本信息
#[tauri::command]
pub async fn load_file(file_path: String, app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<FileInfo, String> {
    log::info!("🚀 开始加载文件: {}", file_path);
    
    // 发送状态更新事件
    state.emit_status_update(&app, &ProcessingStatus::Loading);
    state.emit_progress_update(&app, 0, 100, "开始加载文件...");
    
    let mut app_state = state.lock();
    
    // 更新状态为加载中
    app_state.set_processing_status(ProcessingStatus::Loading);
    app_state.add_message("info", "文件加载", &format!("开始加载文件: {}", file_path));
    
    // 发送日志消息事件
    if let Some(last_message) = app_state.messages.last() {
        state.emit_log_message(&app, last_message);
    }
    
    log::info!("📊 状态已更新为: Loading");
    
    // 获取文件信息
    log::info!("📁 获取文件元数据...");
    state.emit_progress_update(&app, 10, 100, "获取文件元数据...");
    
    let metadata = std::fs::metadata(&file_path)
        .map_err(|e| {
            log::error!("❌ 无法读取文件元数据: {}", e);
            state.emit_status_update(&app, &ProcessingStatus::Error(format!("无法读取文件: {}", e)));
            format!("无法读取文件: {}", e)
        })?;
    
    log::info!("📊 文件大小: {} bytes", metadata.len());
    state.emit_progress_update(&app, 20, 100, &format!("文件大小: {} bytes", metadata.len()));
    
    let file_name = std::path::Path::new(&file_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("未知文件")
        .to_string();
    
    let format = std::path::Path::new(&file_path)
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("未知")
        .to_string();
    
    log::info!("📄 文件名: {}, 格式: {}", file_name, format);
    
    // 使用真实的DataLoader加载文件，支持进度报告
    log::info!("🔄 开始使用DataLoader加载文件...");
    state.emit_progress_update(&app, 30, 100, "使用DataLoader加载文件...");
    
    // 使用带进度报告的DataLoader
    let (is_valid, spectra_count, data_ranges) = match DataLoader::load_from_file(&file_path) {
        Ok(container) => {
            let count = container.spectra.len();
            log::info!("✅ 文件加载成功: {} 个光谱", count);
            log::info!("📈 曲线数量: {}", container.curves.len());
            log::info!("🔍 峰数量: {}", container.total_peak_count());
            
            // 发送加载完成进度更新
            state.emit_progress_update(&app, 70, 100, &format!("成功加载 {} 个光谱", count));
            
            // 从元数据中提取数据范围
            let ranges = if let (Some(rt_min), Some(rt_max), Some(mz_min), Some(mz_max)) = (
                container.metadata.get("rt_min").and_then(|v| v.as_f64()),
                container.metadata.get("rt_max").and_then(|v| v.as_f64()),
                container.metadata.get("mz_min").and_then(|v| v.as_f64()),
                container.metadata.get("mz_max").and_then(|v| v.as_f64()),
            ) {
                // 获取可用的 MS 级别
                let ms_levels: Vec<u8> = container.spectra.iter()
                    .map(|s| s.ms_level())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect();
                
                Some(DataRanges {
                    rt_min,
                    rt_max,
                    mz_min,
                    mz_max,
                    ms_levels,
                })
            } else {
                None
            };
            
            if let Some(ref ranges) = ranges {
                log::info!("📊 数据范围 - RT: {:.2} - {:.2}, m/z: {:.2} - {:.2}", 
                    ranges.rt_min, ranges.rt_max, ranges.mz_min, ranges.mz_max);
            }
            
            // 缓存文件数据以提高后续操作性能
            state.cache_file(&file_path, container.clone());
            
            state.emit_progress_update(&app, 80, 100, &format!("成功加载 {} 个光谱", count));
            
            app_state.add_message("success", "文件加载成功", &format!("成功加载 {} 个光谱", count));
            
            // 发送成功消息
            if let Some(last_message) = app_state.messages.last() {
                state.emit_log_message(&app, last_message);
            }
            
            (true, Some(count), ranges)
        }
        Err(e) => {
            log::error!("❌ 文件加载失败: {}", e);
            state.emit_status_update(&app, &ProcessingStatus::Error(format!("文件加载失败: {}", e)));
            app_state.add_message("error", "文件加载失败", &format!("错误: {}", e));
            
            // 发送错误消息
            if let Some(last_message) = app_state.messages.last() {
                state.emit_log_message(&app, last_message);
            }
            
            (false, None, None)
        }
    };
    
    let file_info = FileInfo {
        path: file_path.clone(),
        name: file_name.clone(),
        size: metadata.len(),
        format,
        is_valid,
        spectra_count,
        data_ranges,
    };
    
    // 更新状态
    log::info!("🔄 更新应用状态...");
    state.emit_progress_update(&app, 90, 100, "更新应用状态...");
    
    app_state.set_current_files(vec![file_path.clone()]);
    app_state.set_processing_status(ProcessingStatus::Idle);
    app_state.add_message("success", "文件加载完成", &format!("文件信息已获取，包含 {} 个光谱", spectra_count.unwrap_or(0)));
    
    // 发送最终状态更新
    state.emit_status_update(&app, &ProcessingStatus::Idle);
    state.emit_progress_update(&app, 100, 100, "文件加载完成");
    
    // 发送最终日志消息
    if let Some(last_message) = app_state.messages.last() {
        state.emit_log_message(&app, last_message);
    }
    
    log::info!("✅ 文件加载命令完成: {}", file_info.name);
    Ok(file_info)
}

/// 清理文件缓存
#[tauri::command]
pub async fn clear_file_cache(_app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<String, String> {
    log::info!("🗑️ 清理文件缓存");
    state.clear_file_cache();
    Ok("文件缓存已清理".to_string())
}

/// 步骤2: 验证文件并获取数据范围
#[tauri::command]
pub async fn validate_file(file_path: String, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<ValidationResult, String> {
    log::info!("🔍 开始验证文件: {}", file_path);
    
    let mut app_state = state.lock();
    
    app_state.add_message("info", "文件验证", &format!("验证文件: {}", file_path));
    log::info!("📊 开始文件格式和数据完整性验证...");
    
    // 使用真实的DataLoader验证文件
    log::info!("🔄 使用DataLoader验证文件...");
    let result = match DataLoader::load_from_file(&file_path) {
        Ok(container) => {
            let spectra_count = container.spectra.len();
            log::info!("✅ 文件验证成功: {} 个光谱", spectra_count);
            
            // 从元数据中获取真实的数据范围
            let data_ranges = if !container.spectra.is_empty() {
                let rt_min = container.metadata.get("rt_min")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(0.0);
                let rt_max = container.metadata.get("rt_max")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(60.0);
                let mz_min = container.metadata.get("mz_min")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(50.0);
                let mz_max = container.metadata.get("mz_max")
                    .and_then(|v| v.as_f64())
                    .unwrap_or(2000.0);
                
                log::info!("📊 数据范围 - m/z: {:.2}-{:.2}, RT: {:.2}-{:.2}", mz_min, mz_max, rt_min, rt_max);
                
                // 获取MS级别
                let mut ms_levels = std::collections::HashSet::new();
                for spectrum in &container.spectra {
                    ms_levels.insert(spectrum.ms_level());
                }
                let ms_levels: Vec<u8> = ms_levels.into_iter().collect();
                log::info!("🔬 MS级别: {:?}", ms_levels);
                
                Some(DataRanges {
                    mz_min,
                    mz_max,
                    rt_min,
                    rt_max,
                    ms_levels,
                })
            } else {
                log::warn!("⚠️  没有光谱数据，无法获取数据范围");
                None
            };
            
            ValidationResult {
                is_valid: true,
                message: "文件格式有效".to_string(),
                spectra_count: Some(spectra_count),
                file_size: std::fs::metadata(&file_path).ok().map(|m| m.len()),
                data_ranges,
            }
        }
        Err(e) => {
            log::error!("❌ 文件验证失败: {}", e);
            ValidationResult {
                is_valid: false,
                message: format!("文件验证失败: {}", e),
                spectra_count: None,
                file_size: std::fs::metadata(&file_path).ok().map(|m| m.len()),
                data_ranges: None,
            }
        }
    };
    
    if result.is_valid {
        app_state.add_message("success", "验证成功", &format!("文件包含 {} 个光谱", result.spectra_count.unwrap_or(0)));
    } else {
        app_state.add_message("error", "验证失败", &result.message);
    }
    
    Ok(result)
}
