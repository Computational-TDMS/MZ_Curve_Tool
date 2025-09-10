//! Tauri命令实现
//! 包含所有前端调用的后端命令

use tauri::State;
use serde::{Deserialize, Serialize};
use crate::tauri::state::{AppState, AppStateManager, ProcessingParams, ProcessingStatus};
use crate::core::loaders::mzdata_loader::DataLoader;
use crate::core::state::{DTCurvePoint, PeakInfo, CurveData, CurveMetadata};
use crate::core::processors::base::Processor;
use crate::core::data::DataContainer;
use mzdata::prelude::SpectrumLike;
use uuid::Uuid;
// use crate::core::exporters::export_manager::ExportManager; // 暂时注释掉，避免导入问题
// use crate::core::exporters::base::ExportResult; // 暂时注释掉，避免导入问题
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

// 曲线数据（TSV格式）

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
    pub curve_data: CurveData,
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
            log::info!("🔍 峰数量: {}", container.peaks.len());
            
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
    let data_container = DataContainer {
        metadata: result.metadata,
        spectra: Vec::new(), // ProcessingResult没有spectra字段，使用空向量
        curves: result.curves,
        peaks: result.peaks,
    };
    
    // 转换为可序列化的数据容器
    let serializable_container = crate::core::data::container::SerializableDataContainer::from(data_container);
    
    Ok(serializable_container)
}

// 峰检测和峰拟合命令已移至 pipeline_commands.rs 中

/// 步骤4: 峰分析（保留向后兼容）
#[tauri::command]
pub async fn analyze_peaks(
    params: PeakAnalysisParams,
    _app: tauri::AppHandle,
    state: State<'_, AppStateManager>
) -> Result<PeakAnalysisResult, String> {
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Analyzing);
        app_state.add_message("info", "峰分析", "开始峰分析...");
    }
    
    let start_time = std::time::Instant::now();
    
    // 使用真实的PeakAnalyzer进行峰分析
    log::info!("🔧 创建峰分析器: 检测={}, 拟合={}, 重叠处理={:?}", 
        params.detection_method, params.fitting_method, params.overlapping_method);
    
    let peak_analyzer = match crate::core::processors::peak_analyzer::PeakAnalyzer::new_with_overlapping_processing(
        &params.detection_method,
        &params.fitting_method,
        params.overlapping_method.as_deref()
    ) {
        Ok(analyzer) => {
            log::info!("✅ 峰分析器创建成功");
            analyzer
        }
        Err(e) => {
            log::error!("❌ 峰分析器创建失败: {}", e);
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "峰分析器创建失败", &format!("错误: {}", e));
            }
            return Err(format!("峰分析器创建失败: {}", e));
        }
    };
    
    // 转换CurveData到DataContainer
    log::info!("🔄 转换曲线数据到DataContainer...");
    let mut container = crate::core::data::DataContainer::new();
    
    // 创建Curve对象
    let x_values: Vec<f64> = params.curve_data.data_points.iter().map(|p| p.drift_time).collect();
    let y_values: Vec<f64> = params.curve_data.data_points.iter().map(|p| p.intensity).collect();
    
    log::info!("📊 曲线数据: {} 个数据点, X范围: {:.2}-{:.2}, Y范围: {:.2}-{:.2}", 
        x_values.len(), 
        x_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
        x_values.iter().fold(0.0_f64, |a, &b| a.max(b)),
        y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b)),
        y_values.iter().fold(0.0_f64, |a, &b| a.max(b))
    );
    
    let curve = crate::core::data::Curve::new(
        format!("curve_{}", uuid::Uuid::new_v4()),
        params.curve_data.curve_type.clone(),
        x_values,
        y_values,
        "Drift Time".to_string(),
        "Intensity".to_string(),
        "ms".to_string(),
        "counts".to_string(),
    );
    
    container.curves.push(curve);
    log::info!("✅ 曲线数据转换完成");
    
    // 准备配置
    let config = serde_json::json!({
        "detection_method": params.detection_method,
        "fitting_method": params.fitting_method,
        "overlapping_processing": params.overlapping_method.unwrap_or_else(|| "auto".to_string()),
        "sensitivity": params.sensitivity,
        "threshold_multiplier": params.threshold_multiplier,
        "min_peak_width": params.min_peak_width,
        "max_peak_width": params.max_peak_width
    });
    
    // 执行峰分析
    let result = match peak_analyzer.process(container.clone(), config).await {
        Ok(result) => result,
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "峰分析失败", &format!("错误: {}", e));
            }
            return Err(format!("峰分析失败: {}", e));
        }
    };
    
    // 生成TSV格式的峰数据
    log::info!("📊 生成峰数据TSV...");
    let mut peaks_tsv = String::new();
    peaks_tsv.push_str("id\tcenter\tamplitude\tfwhm\tarea\trsquared\tquality_score\tconfidence\tasymmetry_factor\n");
    
    for peak in &result.peaks {
        let quality_score = peak.get_quality_score();
        let confidence = peak.metadata.get("confidence")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let asymmetry = peak.metadata.get("asymmetry_factor")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
            
        peaks_tsv.push_str(&format!(
            "{}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.6}\t{:.3}\t{:.3}\t{:.3}\n",
            peak.id,
            peak.center,
            peak.amplitude,
            peak.fwhm,
            peak.area,
            peak.rsquared,
            quality_score,
            confidence,
            asymmetry
        ));
    }
    
    log::info!("✅ 峰数据TSV生成完成: {} 个峰", result.peaks.len());
    
    // 生成拟合曲线TSV数据
    log::info!("📈 生成拟合曲线TSV...");
    let mut fitted_curve_tsv = String::new();
    fitted_curve_tsv.push_str("x\ty_original\ty_fitted\n");
    
    if !result.curves.is_empty() {
        let curve = &result.curves[0];
        // 获取原始曲线数据
        let original_curve = &container.curves[0];
        
        for (i, (x, y_orig)) in original_curve.x_values.iter().zip(original_curve.y_values.iter()).enumerate() {
            let y_fitted = if i < curve.y_values.len() {
                curve.y_values[i]
            } else {
                *y_orig // 如果没有拟合值，使用原始值
            };
            fitted_curve_tsv.push_str(&format!("{:.6}\t{:.6}\t{:.6}\n", x, y_orig, y_fitted));
        }
    }
    
    log::info!("✅ 拟合曲线TSV生成完成");
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    let analysis_result = PeakAnalysisResult {
        success: true,
        peaks_tsv,
        fitted_curve_tsv,
        peak_count: result.peaks.len(),
        processing_time,
        error: None,
    };
    
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Idle);
        app_state.add_message("success", "峰分析完成", &format!("检测到 {} 个峰，耗时 {}ms", analysis_result.peak_count, processing_time));
    }
    
    Ok(analysis_result)
}

/// 获取应用状态
#[tauri::command]
pub fn get_app_state(state: State<'_, AppStateManager>) -> Result<AppState, String> {
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
pub fn get_processing_status(state: State<'_, AppStateManager>) -> Result<ProcessingStatus, String> {
    let app_state = state.lock();
    Ok(app_state.processing_status.clone())
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

// 基线校正参数结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineCorrectionParams {
    pub file_path: String,
    pub method: String, // "linear", "polynomial", "moving_average", "asymmetric_least_squares"
    pub degree: Option<u32>, // 多项式次数
    pub window_size: Option<usize>, // 移动平均窗口大小
    pub lambda: Option<f64>, // 非对称最小二乘参数
    pub p: Option<f64>, // 非对称最小二乘参数
    pub max_iterations: Option<usize>, // 最大迭代次数
}

// 基线校正结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineCorrectionResult {
    pub success: bool,
    pub corrected_curve: Option<CurveData>,
    pub baseline_curve: Option<CurveData>,
    pub correction_method: String,
    pub processing_time: u64,
    pub message: String,
}

/// 基线校正处理
#[tauri::command]
pub async fn baseline_correction(params: BaselineCorrectionParams, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<BaselineCorrectionResult, String> {
    {
        let mut app_state = state.lock();
        app_state.add_message("info", "基线校正", &format!("开始基线校正: {} - {}", params.file_path, params.method));
    }
    
    let start_time = std::time::Instant::now();
    
    // 加载数据
    let container = match DataLoader::load_from_file(&params.file_path) {
        Ok(container) => container,
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "基线校正失败", &format!("无法加载文件: {}", e));
            }
            return Err(format!("无法加载文件: {}", e));
        }
    };
    
    // 使用真实的BaselineProcessor进行基线校正
    let baseline_processor = crate::core::processors::baseline_correction::BaselineProcessor::new();
    
    // 准备配置
    let mut config = serde_json::json!({
        "method": params.method,
        "preserve_original": true,
        "output_baseline": true
    });
    
    // 添加方法特定的参数
    match params.method.as_str() {
        "polynomial" => {
            if let Some(degree) = params.degree {
                config["degree"] = serde_json::json!(degree);
            }
        }
        "moving_average" => {
            if let Some(window_size) = params.window_size {
                config["window_size"] = serde_json::json!(window_size);
            }
        }
        "asymmetric_least_squares" => {
            if let Some(lambda) = params.lambda {
                config["lambda"] = serde_json::json!(lambda);
            }
            if let Some(p) = params.p {
                config["p"] = serde_json::json!(p);
            }
            if let Some(max_iterations) = params.max_iterations {
                config["max_iterations"] = serde_json::json!(max_iterations);
            }
        }
        _ => {}
    }
    
    // 执行基线校正
    let result = match baseline_processor.process(container, config).await {
        Ok(result) => result,
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "基线校正失败", &format!("错误: {}", e));
            }
            return Err(format!("基线校正失败: {}", e));
        }
    };
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    // 转换结果到API格式
    let mut corrected_curve = None;
    let mut baseline_curve = None;
    
    for curve in &result.curves {
        if curve.curve_type == "Baseline" {
            // 基线曲线
            let data_points: Vec<DTCurvePoint> = curve.x_values.iter()
                .zip(curve.y_values.iter())
                .map(|(&x, &y)| DTCurvePoint { drift_time: x, intensity: y })
                .collect();
            
            baseline_curve = Some(CurveData {
        file_name: format!("{}_baseline", params.file_path),
        curve_type: "Baseline".to_string(),
                data_points,
        metadata: CurveMetadata {
                    total_points: curve.point_count,
                    rt_range: (curve.x_min, curve.x_max),
                    intensity_range: (curve.y_min, curve.y_max),
                    max_intensity: curve.y_max,
                    max_intensity_rt: curve.x_values[curve.y_values.iter().position(|&y| y == curve.y_max).unwrap_or(0)],
                },
            });
        } else {
            // 校正后的曲线
            let data_points: Vec<DTCurvePoint> = curve.x_values.iter()
                .zip(curve.y_values.iter())
                .map(|(&x, &y)| DTCurvePoint { drift_time: x, intensity: y })
                .collect();
            
            corrected_curve = Some(CurveData {
                file_name: format!("{}_baseline_corrected", params.file_path),
                curve_type: curve.curve_type.clone(),
                data_points,
                metadata: CurveMetadata {
                    total_points: curve.point_count,
                    rt_range: (curve.x_min, curve.x_max),
                    intensity_range: (curve.y_min, curve.y_max),
                    max_intensity: curve.y_max,
                    max_intensity_rt: curve.x_values[curve.y_values.iter().position(|&y| y == curve.y_max).unwrap_or(0)],
                },
            });
        }
    }
    
    {
        let mut app_state = state.lock();
        app_state.add_message("success", "基线校正完成", &format!("使用 {} 方法完成基线校正", params.method));
    }
    
    Ok(BaselineCorrectionResult {
        success: true,
        corrected_curve,
        baseline_curve,
        correction_method: params.method,
        processing_time,
        message: "基线校正成功".to_string(),
    })
}

// 峰重叠处理参数结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlappingPeaksParams {
    pub file_path: String,
    pub method: String, // "fbf", "sharpen_cwt", "emg_nlls", "extreme_overlap"
    pub peaks: Vec<PeakInfo>,
    pub curve: CurveData,
    pub config: Option<serde_json::Value>,
}

// 峰重叠处理结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverlappingPeaksResult {
    pub success: bool,
    pub processed_peaks: Vec<PeakInfo>,
    pub processing_method: String,
    pub processing_time: u64,
    pub message: String,
}

/// 峰重叠处理
#[tauri::command]
pub async fn overlapping_peaks(params: OverlappingPeaksParams, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<OverlappingPeaksResult, String> {
    log::info!("🔍 开始峰重叠处理: {} - {}", params.file_path, params.method);
    
    let mut app_state = state.lock();
    
    app_state.add_message("info", "峰重叠处理", &format!("开始峰重叠处理: {} - {}", params.file_path, params.method));
    
    let start_time = std::time::Instant::now();
    
    // 使用真实的峰重叠处理算法
    log::info!("🔄 使用 {} 方法处理 {} 个重叠峰", params.method, params.peaks.len());
    
    // 根据方法选择不同的处理器
    let result = match params.method.as_str() {
        "fbf" => {
            log::info!("📊 使用FBF方法处理峰重叠");
            // 这里应该调用真实的FBF处理器
            Err::<Vec<PeakInfo>, String>("FBF处理器尚未实现".to_string())
        }
        "sharpen_cwt" => {
            log::info!("📊 使用Sharpen CWT方法处理峰重叠");
            // 这里应该调用真实的Sharpen CWT处理器
            Err::<Vec<PeakInfo>, String>("Sharpen CWT处理器尚未实现".to_string())
        }
        "emg_nlls" => {
            log::info!("📊 使用EMG NLLS方法处理峰重叠");
            // 这里应该调用真实的EMG NLLS处理器
            Err::<Vec<PeakInfo>, String>("EMG NLLS处理器尚未实现".to_string())
        }
        "extreme_overlap" => {
            log::info!("📊 使用Extreme Overlap方法处理峰重叠");
            // 这里应该调用真实的Extreme Overlap处理器
            Err::<Vec<PeakInfo>, String>("Extreme Overlap处理器尚未实现".to_string())
        }
        _ => {
            log::error!("❌ 未知的峰重叠处理方法: {}", params.method);
            Err(format!("未知的峰重叠处理方法: {}", params.method))
        }
    };
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    match result {
        Ok(processed_peaks) => {
            log::info!("✅ 峰重叠处理成功: {} 个峰", processed_peaks.len());
    app_state.add_message("success", "峰重叠处理完成", &format!("使用 {} 方法处理了 {} 个峰", params.method, processed_peaks.len()));
    
    Ok(OverlappingPeaksResult {
        success: true,
        processed_peaks,
        processing_method: params.method,
        processing_time,
        message: "峰重叠处理成功".to_string(),
    })
        }
        Err(e) => {
            log::error!("❌ 峰重叠处理失败: {}", e);
            app_state.add_message("error", "峰重叠处理失败", &e);
            Err(e)
        }
    }
}

// 数据平滑参数结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmoothDataParams {
    pub file_path: String,
    pub method: String, // "moving_average", "savitzky_golay", "gaussian", "lowess"
    pub window_size: Option<usize>,
    pub polynomial_order: Option<u32>, // Savitzky-Golay多项式阶数
    pub sigma: Option<f64>, // 高斯平滑参数
    pub span: Option<f64>, // LOWESS平滑参数
}

// 数据平滑结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmoothDataResult {
    pub success: bool,
    pub smoothed_curve: CurveData,
    pub smoothing_method: String,
    pub processing_time: u64,
    pub message: String,
}

/// 数据平滑处理
#[tauri::command]
pub async fn smooth_data(params: SmoothDataParams, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<SmoothDataResult, String> {
    log::info!("📊 开始数据平滑: {} - {}", params.file_path, params.method);
    
    let mut app_state = state.lock();
    
    app_state.add_message("info", "数据平滑", &format!("开始数据平滑: {} - {}", params.file_path, params.method));
    
    let start_time = std::time::Instant::now();
    
    // 加载原始数据
    log::info!("🔄 加载原始数据...");
    let _container = match DataLoader::load_from_file(&params.file_path) {
        Ok(container) => {
            log::info!("✅ 数据加载成功: {} 条曲线", container.curves.len());
            container
        }
        Err(e) => {
            log::error!("❌ 数据加载失败: {}", e);
            app_state.add_message("error", "数据平滑失败", &format!("无法加载文件: {}", e));
            return Err(format!("无法加载文件: {}", e));
        }
    };
    
    // 使用真实的数据平滑算法
    log::info!("🔄 使用 {} 方法进行数据平滑", params.method);
    
    let result = match params.method.as_str() {
        "moving_average" => {
            log::info!("📊 使用移动平均方法");
            if let Some(window_size) = params.window_size {
                log::info!("📊 窗口大小: {}", window_size);
                // 这里应该调用真实的移动平均处理器
                Err::<(CurveData, f64), String>("移动平均处理器尚未实现".to_string())
            } else {
                Err("移动平均方法需要指定窗口大小".to_string())
            }
        }
        "savitzky_golay" => {
            log::info!("📊 使用Savitzky-Golay方法");
            if let Some(polynomial_order) = params.polynomial_order {
                log::info!("📊 多项式阶数: {}", polynomial_order);
                // 这里应该调用真实的Savitzky-Golay处理器
                Err("Savitzky-Golay处理器尚未实现".to_string())
            } else {
                Err("Savitzky-Golay方法需要指定多项式阶数".to_string())
            }
        }
        "gaussian" => {
            log::info!("📊 使用高斯平滑方法");
            if let Some(sigma) = params.sigma {
                log::info!("📊 高斯参数σ: {}", sigma);
                // 这里应该调用真实的高斯平滑处理器
                Err("高斯平滑处理器尚未实现".to_string())
            } else {
                Err("高斯平滑方法需要指定σ参数".to_string())
            }
        }
        "lowess" => {
            log::info!("📊 使用LOWESS方法");
            if let Some(span) = params.span {
                log::info!("📊 LOWESS参数span: {}", span);
                // 这里应该调用真实的LOWESS处理器
                Err("LOWESS处理器尚未实现".to_string())
            } else {
                Err("LOWESS方法需要指定span参数".to_string())
            }
        }
        _ => {
            log::error!("❌ 未知的数据平滑方法: {}", params.method);
            Err(format!("未知的数据平滑方法: {}", params.method))
        }
    };
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    match result {
        Ok((smoothed_curve, _smoothing_factor)) => {
            log::info!("✅ 数据平滑成功: {} 个数据点", smoothed_curve.metadata.total_points);
    app_state.add_message("success", "数据平滑完成", &format!("使用 {} 方法完成数据平滑", params.method));
    
    Ok(SmoothDataResult {
        success: true,
        smoothed_curve,
        smoothing_method: params.method,
        processing_time,
        message: "数据平滑成功".to_string(),
    })
        }
        Err(e) => {
            log::error!("❌ 数据平滑失败: {}", e);
            app_state.add_message("error", "数据平滑失败", &e);
            Err(e)
        }
    }
}

// 噪声降低参数结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseReductionParams {
    pub file_path: String,
    pub method: String, // "wavelet", "fourier", "median_filter", "wiener_filter"
    pub threshold: Option<f64>,
    pub wavelet_type: Option<String>, // "daubechies", "coiflets", "biorthogonal"
    pub decomposition_level: Option<u32>,
    pub cutoff_frequency: Option<f64>, // 傅里叶滤波截止频率
}

// 噪声降低结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseReductionResult {
    pub success: bool,
    pub denoised_curve: CurveData,
    pub noise_reduction_method: String,
    pub snr_improvement: Option<f64>,
    pub processing_time: u64,
    pub message: String,
}

/// 噪声降低处理
#[tauri::command]
pub async fn noise_reduction(params: NoiseReductionParams, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<NoiseReductionResult, String> {
    log::info!("🔇 开始噪声降低: {} - {}", params.file_path, params.method);
    
    let mut app_state = state.lock();
    
    app_state.add_message("info", "噪声降低", &format!("开始噪声降低: {} - {}", params.file_path, params.method));
    
    let start_time = std::time::Instant::now();
    
    // 加载原始数据
    log::info!("🔄 加载原始数据...");
    let _container = match DataLoader::load_from_file(&params.file_path) {
        Ok(container) => {
            log::info!("✅ 数据加载成功: {} 条曲线", container.curves.len());
            container
        }
        Err(e) => {
            log::error!("❌ 数据加载失败: {}", e);
            app_state.add_message("error", "噪声降低失败", &format!("无法加载文件: {}", e));
            return Err(format!("无法加载文件: {}", e));
        }
    };
    
    // 使用真实的噪声降低算法
    log::info!("🔄 使用 {} 方法进行噪声降低", params.method);
    
    let result = match params.method.as_str() {
        "wavelet" => {
            log::info!("📊 使用小波变换方法");
            if let Some(wavelet_type) = &params.wavelet_type {
                log::info!("📊 小波类型: {}", wavelet_type);
            }
            if let Some(decomposition_level) = params.decomposition_level {
                log::info!("📊 分解层数: {}", decomposition_level);
            }
            if let Some(threshold) = params.threshold {
                log::info!("📊 阈值: {}", threshold);
            }
            // 这里应该调用真实的小波变换处理器
            Err::<(CurveData, f64), String>("小波变换处理器尚未实现".to_string())
        }
        "fourier" => {
            log::info!("📊 使用傅里叶变换方法");
            if let Some(cutoff_frequency) = params.cutoff_frequency {
                log::info!("📊 截止频率: {}", cutoff_frequency);
            }
            // 这里应该调用真实的傅里叶变换处理器
            Err("傅里叶变换处理器尚未实现".to_string())
        }
        "median_filter" => {
            log::info!("📊 使用中值滤波方法");
            if let Some(threshold) = params.threshold {
                log::info!("📊 阈值: {}", threshold);
            }
            // 这里应该调用真实的中值滤波处理器
            Err("中值滤波处理器尚未实现".to_string())
        }
        "wiener_filter" => {
            log::info!("📊 使用维纳滤波方法");
            if let Some(threshold) = params.threshold {
                log::info!("📊 阈值: {}", threshold);
            }
            // 这里应该调用真实的维纳滤波处理器
            Err("维纳滤波处理器尚未实现".to_string())
        }
        _ => {
            log::error!("❌ 未知的噪声降低方法: {}", params.method);
            Err(format!("未知的噪声降低方法: {}", params.method))
        }
    };
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    match result {
        Ok((denoised_curve, snr_improvement)) => {
            log::info!("✅ 噪声降低成功: {} 个数据点, SNR提升: {:.2}", 
                denoised_curve.metadata.total_points, snr_improvement);
    app_state.add_message("success", "噪声降低完成", &format!("使用 {} 方法完成噪声降低", params.method));
    
    Ok(NoiseReductionResult {
        success: true,
        denoised_curve: denoised_curve,
        noise_reduction_method: params.method,
                snr_improvement: Some(snr_improvement),
        processing_time,
        message: "噪声降低成功".to_string(),
    })
        }
        Err(e) => {
            log::error!("❌ 噪声降低失败: {}", e);
            app_state.add_message("error", "噪声降低失败", &e);
            Err(e)
        }
    }
}

// 配置管理结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserConfig {
    pub processing_params: ProcessingParams,
    pub ui_settings: UiSettings,
    pub export_settings: ExportSettings,
    pub visualization_settings: VisualizationSettings,
    pub last_updated: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    pub theme: String, // "light", "dark", "auto"
    pub language: String, // "zh", "en"
    pub window_size: (u32, u32),
    pub window_position: (i32, i32),
    pub auto_save: bool,
    pub auto_save_interval: u32, // 分钟
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportSettings {
    pub default_format: String, // "tsv", "json", "plot"
    pub default_directory: String,
    pub include_metadata: bool,
    pub decimal_precision: usize,
    pub auto_export: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationSettings {
    pub default_plot_type: String, // "line", "scatter", "bar"
    pub color_scheme: String,
    pub show_grid: bool,
    pub show_legend: bool,
    pub auto_scale: bool,
    pub peak_highlighting: bool,
}

// 配置管理结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigResult {
    pub success: bool,
    pub message: String,
    pub config: Option<UserConfig>,
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

// 可视化参数结构
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotData {
    pub plot_id: String,
    pub plot_type: String,
    pub data: serde_json::Value, // Plotly数据格式
    pub layout: serde_json::Value, // Plotly布局
    pub config: serde_json::Value, // Plotly配置
    pub metadata: PlotMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlotMetadata {
    pub title: String,
    pub x_axis_label: String,
    pub y_axis_label: String,
    pub data_points: usize,
    pub generated_at: String,
    pub file_path: String,
}

// 可视化结果结构
#[derive(Debug, Clone, Serialize, Deserialize)]
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
pub async fn export_plot_image(plot_id: String, format: String, output_path: String, _app: tauri::AppHandle, state: State<'_, AppStateManager>) -> Result<ExportResultInfo, String> {
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

