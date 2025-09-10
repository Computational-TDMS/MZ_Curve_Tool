//! 曲线提取相关命令

use tauri::State;
use crate::tauri::state::{AppStateManager, ProcessingStatus};
use crate::core::loaders::mzdata_loader::DataLoader;
use crate::core::processors::base::Processor;
use super::{CurveExtractionParams, BatchProcessingResult, CurveDisplayData};

/// 步骤3: 提取曲线数据
#[tauri::command]
pub async fn extract_curve(
    params: CurveExtractionParams,
    _app: tauri::AppHandle,
    state: State<'_, AppStateManager>
) -> Result<crate::core::data::container::SerializableDataContainer, String> {
    log::info!("📈 开始提取曲线数据");
    log::info!("📊 参数: 文件={}, m/z范围={}, RT范围={}, MS级别={}, 曲线类型={}", 
        params.file_path, params.mz_range, params.rt_range, params.ms_level, params.curve_type);
    
    {
    let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Extracting);
        app_state.add_message("info", "曲线提取", &format!("开始提取 {} 曲线", params.curve_type));
        log::info!("📊 状态已更新为: Extracting");
    }
    
    let start_time = std::time::Instant::now();
    log::info!("⏱️ 开始曲线提取，开始时间: {:?}", start_time);
    
    // 首先尝试从缓存获取文件数据，避免重复加载
    let container = if let Some(cached_container) = state.get_cached_file(&params.file_path) {
        log::info!("🚀 使用缓存的文件数据，跳过重新加载");
        cached_container
    } else {
        log::info!("📁 缓存中未找到文件，开始加载: {}", params.file_path);
        match DataLoader::load_from_file(&params.file_path) {
            Ok(container) => {
                // 缓存新加载的文件
                state.cache_file(&params.file_path, container.clone());
                container
            },
            Err(e) => {
                {
                    let mut app_state = state.lock();
                    app_state.add_message("error", "文件加载失败", &format!("错误: {}", e));
                }
                return Err(format!("无法加载文件: {}", e));
            }
        }
    };
    
    // 根据曲线类型选择不同的提取器
    let result = match params.curve_type.as_str() {
        "dt" => {
            // 使用DTExtractor
            let extractor = crate::core::processors::dt_extractor::DTExtractor;
            let config = serde_json::json!({
                "mz_range": params.mz_range,
                "rt_range": params.rt_range,
                "ms_level": params.ms_level
            });
            extractor.process(container, config).await
        },
        "tic" => {
            // 使用TICExtractor
            let extractor = crate::core::processors::tic_extractor::TICExtractor;
            let config = serde_json::json!({
                "rt_range": params.rt_range,
                "ms_level": params.ms_level
                // TIC不需要mz_range，会使用全m/z范围
            });
            extractor.process(container, config).await
        },
        "xic" => {
            // 使用XICExtractor
            let extractor = crate::core::processors::xic_extractor::XICExtractor;
            let config = serde_json::json!({
                "mz_range": params.mz_range,
                "rt_range": params.rt_range,
                "ms_level": params.ms_level
            });
            extractor.process(container, config).await
        },
        _ => {
            return Err(format!("不支持的曲线类型: {}", params.curve_type));
        }
    };
    
    let result = match result {
        Ok(result) => result,
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "曲线提取失败", &format!("错误: {}", e));
            }
            return Err(format!("曲线提取失败: {}", e));
        }
    };
    
    // 检查结果
    if result.curves.is_empty() {
        {
            let mut app_state = state.lock();
            app_state.add_message("error", "曲线提取失败", "未找到符合条件的曲线数据");
        }
        return Err("未找到符合条件的曲线数据".to_string());
    }
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    log::info!("⏱️ 曲线提取完成，总耗时: {}ms", processing_time);
    
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Idle);
        app_state.add_message("success", "曲线提取完成", &format!("提取了 {} 条曲线，耗时 {}ms", result.curves.len(), processing_time));
    }
    
    // 将ProcessingResult转换为DataContainer
    let data_container = crate::core::data::DataContainer {
        metadata: result.metadata,
        spectra: Vec::new(), // ProcessingResult没有spectra字段，使用空向量
        curves: result.curves,
        peaks: result.peaks,
    };
    
    // 转换为可序列化的数据容器
    let serializable_container = crate::core::data::container::SerializableDataContainer::from(data_container);
    
    Ok(serializable_container)
}

/// 批量处理多个文件
#[tauri::command]
pub async fn batch_process_files(
    file_paths: Vec<String>,
    params: CurveExtractionParams,
    app: tauri::AppHandle,
    state: State<'_, AppStateManager>
) -> Result<BatchProcessingResult, String> {
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Extracting);
        app_state.add_message("info", "批量处理", &format!("开始批量处理 {} 个文件", file_paths.len()));
    }
    
    let start_time = std::time::Instant::now();
    let mut processed_files = Vec::new();
    let mut failed_files = Vec::new();
    let mut total_curves = 0;
    let total_peaks = 0;
    
    for file_path in file_paths {
        let mut file_params = params.clone();
        file_params.file_path = file_path.clone();
        
        match extract_curve(file_params, app.clone(), state.clone()).await {
            Ok(container) => {
                processed_files.push(file_path);
                total_curves += 1;
                {
                    let mut app_state = state.lock();
                    app_state.add_message("success", "文件处理完成", &format!("成功处理: {} 条曲线", container.curves.len()));
                }
            }
            Err(e) => {
                failed_files.push(file_path);
                {
                    let mut app_state = state.lock();
                    app_state.add_message("error", "文件处理失败", &format!("处理失败: {}", e));
                }
            }
        }
    }
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    let result = BatchProcessingResult {
        success: !processed_files.is_empty(),
        processed_files,
        failed_files: failed_files.clone(),
        total_curves,
        total_peaks,
        processing_time,
        error: if failed_files.is_empty() { None } else { Some("部分文件处理失败".to_string()) },
    };
    
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Idle);
        app_state.add_message("success", "批量处理完成", &format!("成功处理 {} 个文件，失败 {} 个", result.processed_files.len(), result.failed_files.len()));
    }
    
    Ok(result)
}

/// 获取曲线数据用于显示
#[tauri::command]
pub async fn get_curve_data_for_display(
    _app: tauri::AppHandle,
    state: State<'_, AppStateManager>
) -> Result<Vec<CurveDisplayData>, String> {
    // 从应用状态获取当前处理的数据
    let current_files = {
        let app_state = state.lock();
        app_state.current_files.clone()
    };
    
    let container = if !current_files.is_empty() {
        // 加载当前文件的数据
        match DataLoader::load_from_file(&current_files[0]) {
            Ok(data) => data,
            Err(e) => {
                return Err(format!("无法加载数据: {}", e));
            }
        }
    } else {
        return Err("没有可显示的数据".to_string());
    };
    
    // 转换为显示格式
    let mut display_data = Vec::new();
    for curve in &container.curves {
        // 只取前100个数据点用于显示
        let max_points = 100;
        let x_values: Vec<f64> = curve.x_values.iter().take(max_points).cloned().collect();
        let y_values: Vec<f64> = curve.y_values.iter().take(max_points).cloned().collect();
        
        display_data.push(CurveDisplayData {
            id: curve.id.clone(),
            curve_type: curve.curve_type.clone(),
            x_label: curve.x_label.clone(),
            y_label: curve.y_label.clone(),
            x_unit: curve.x_unit.clone(),
            y_unit: curve.y_unit.clone(),
            point_count: curve.point_count,
            x_values,
            y_values,
            mz_min: curve.mz_range.map(|r| r.0),
            mz_max: curve.mz_range.map(|r| r.1),
        });
    }
    
    Ok(display_data)
}
