//! 峰分析相关命令

use tauri::State;
use crate::tauri::state::{AppStateManager, ProcessingStatus};
use crate::core::processors::base::Processor;
use super::{PeakAnalysisParams, PeakAnalysisResult};

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
