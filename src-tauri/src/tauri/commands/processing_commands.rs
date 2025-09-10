//! 数据处理相关命令

use tauri::State;
use crate::tauri::state::AppStateManager;
use crate::core::loaders::mzdata_loader::DataLoader;
use crate::core::processors::base::Processor;
use crate::core::state::{DTCurvePoint, PeakInfo, CurveData, CurveMetadata};

// 基线校正参数结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct BaselineCorrectionResult {
    pub success: bool,
    pub corrected_curve: Option<CurveData>,
    pub baseline_curve: Option<CurveData>,
    pub correction_method: String,
    pub processing_time: u64,
    pub message: String,
}

// 峰重叠处理参数结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OverlappingPeaksParams {
    pub file_path: String,
    pub method: String, // "fbf", "sharpen_cwt", "emg_nlls", "extreme_overlap"
    pub peaks: Vec<PeakInfo>,
    pub curve: CurveData,
    pub config: Option<serde_json::Value>,
}

// 峰重叠处理结果结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct OverlappingPeaksResult {
    pub success: bool,
    pub processed_peaks: Vec<PeakInfo>,
    pub processing_method: String,
    pub processing_time: u64,
    pub message: String,
}

// 数据平滑参数结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmoothDataParams {
    pub file_path: String,
    pub method: String, // "moving_average", "savitzky_golay", "gaussian", "lowess"
    pub window_size: Option<usize>,
    pub polynomial_order: Option<u32>, // Savitzky-Golay多项式阶数
    pub sigma: Option<f64>, // 高斯平滑参数
    pub span: Option<f64>, // LOWESS平滑参数
}

// 数据平滑结果结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SmoothDataResult {
    pub success: bool,
    pub smoothed_curve: CurveData,
    pub smoothing_method: String,
    pub processing_time: u64,
    pub message: String,
}

// 噪声降低参数结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NoiseReductionParams {
    pub file_path: String,
    pub method: String, // "wavelet", "fourier", "median_filter", "wiener_filter"
    pub threshold: Option<f64>,
    pub wavelet_type: Option<String>, // "daubechies", "coiflets", "biorthogonal"
    pub decomposition_level: Option<u32>,
    pub cutoff_frequency: Option<f64>, // 傅里叶滤波截止频率
}

// 噪声降低结果结构
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NoiseReductionResult {
    pub success: bool,
    pub denoised_curve: CurveData,
    pub noise_reduction_method: String,
    pub snr_improvement: Option<f64>,
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
