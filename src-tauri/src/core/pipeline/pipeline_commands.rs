//! 流水线相关的Tauri命令
//! 
//! 提供流水线处理的Tauri命令接口

use tauri::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::tauri::state::{AppStateManager, ProcessingStatus};
use super::PipelineManager;
use crate::core::data::container::SerializableDataContainer;

/// 峰检测参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakDetectionParams {
    pub method: String, // "cwt", "simple", "peak_finder"
    pub sensitivity: f64,
    pub threshold_multiplier: f64,
    pub min_peak_width: f64,
    pub max_peak_width: f64,
}

/// 峰拟合参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakFittingParams {
    pub method: String, // "gaussian", "lorentzian", "pseudo_voigt"
    pub min_peak_width: f64,
    pub max_peak_width: f64,
    pub fit_quality_threshold: f64,
}

/// 峰增强参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakEnhancementParams {
    pub quality_threshold: f64,
    pub boundary_method: String,
    pub separation_analysis: bool,
}

/// 曲线还原参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurveReconstructionParams {
    pub resolution: usize,
    pub include_baseline: bool,
    pub include_individual_peaks: bool,
}

/// 基线校正参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineCorrectionParams {
    pub method: String, // "linear", "polynomial", "moving_average", "asymmetric_least_squares"
    pub parameters: HashMap<String, serde_json::Value>,
}

/// 流水线执行参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineExecutionParams {
    pub steps: Vec<PipelineStepParams>,
}

/// 流水线步骤参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineStepParams {
    pub step_type: String, // "detection", "fitting", "enhancement", "reconstruction", "baseline"
    pub method: String,
    pub config: HashMap<String, serde_json::Value>,
}

/// 流水线执行结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineExecutionResult {
    pub success: bool,
    pub container: SerializableDataContainer,
    pub execution_time: u64,
    pub steps_completed: Vec<String>,
    pub error: Option<String>,
}

/// 步骤1: 峰检测流水线
#[tauri::command]
pub async fn detect_peaks(
    container: SerializableDataContainer,
    params: PeakDetectionParams,
    _app: tauri::AppHandle,
    state: State<'_, AppStateManager>,
) -> Result<SerializableDataContainer, String> {
    log::info!("🔍 开始峰检测流水线");
    
    // 更新状态
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Analyzing);
        app_state.add_message("info", "峰检测", &format!("使用 {} 方法检测峰", params.method));
    }
    
    let start_time = std::time::Instant::now();
    
    // 创建峰检测配置
    let config = serde_json::json!({
        "detection_method": params.method,
        "sensitivity": params.sensitivity,
        "threshold_multiplier": params.threshold_multiplier,
        "min_peak_width": params.min_peak_width,
        "max_peak_width": params.max_peak_width,
        "fitting_method": "none" // 只进行检测，不进行拟合
    });
    
    // 创建流水线管理器并执行峰检测
    let pipeline = PipelineManager::new()
        .add_peak_detection(&params.method, config);
    
    let result = match pipeline.execute(container).await {
        Ok(result) => result,
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "峰检测失败", &format!("错误: {}", e));
            }
            return Err(format!("峰检测失败: {}", e));
        }
    };
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    // 更新状态
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Idle);
        app_state.add_message("success", "峰检测完成", &format!("检测到 {} 个峰，耗时 {}ms", result.container.peak_count(), processing_time));
    }
    
    Ok(result.container)
}

/// 步骤2: 峰拟合流水线
#[tauri::command]
pub async fn fit_peaks(
    container: SerializableDataContainer,
    params: PeakFittingParams,
    _app: tauri::AppHandle,
    state: State<'_, AppStateManager>,
) -> Result<SerializableDataContainer, String> {
    log::info!("📊 开始峰拟合流水线");
    
    // 更新状态
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Analyzing);
        app_state.add_message("info", "峰拟合", &format!("使用 {} 方法拟合峰", params.method));
    }
    
    let start_time = std::time::Instant::now();
    
    // 创建峰拟合配置
    let config = serde_json::json!({
        "fitting_method": params.method,
        "min_peak_width": params.min_peak_width,
        "max_peak_width": params.max_peak_width,
        "fit_quality_threshold": params.fit_quality_threshold,
        "detection_method": "none" // 跳过检测，只进行拟合
    });
    
    // 创建流水线管理器并执行峰拟合
    let pipeline = PipelineManager::new()
        .add_peak_fitting(&params.method, config);
    
    let result = match pipeline.execute(container).await {
        Ok(result) => result,
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "峰拟合失败", &format!("错误: {}", e));
            }
            return Err(format!("峰拟合失败: {}", e));
        }
    };
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    // 更新状态
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Idle);
        app_state.add_message("success", "峰拟合完成", &format!("拟合了 {} 个峰，耗时 {}ms", result.container.peak_count(), processing_time));
    }
    
    Ok(result.container)
}

/// 步骤3: 峰增强流水线
#[tauri::command]
pub async fn enhance_peaks(
    container: SerializableDataContainer,
    params: PeakEnhancementParams,
    _app: tauri::AppHandle,
    state: State<'_, AppStateManager>,
) -> Result<SerializableDataContainer, String> {
    log::info!("✨ 开始峰增强流水线");
    
    // 更新状态
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Analyzing);
        app_state.add_message("info", "峰增强", &format!("使用 {} 方法增强峰", params.boundary_method));
    }
    
    let start_time = std::time::Instant::now();
    
    // 创建峰增强配置
    let config = serde_json::json!({
        "quality_threshold": params.quality_threshold,
        "boundary_method": params.boundary_method,
        "separation_analysis": params.separation_analysis
    });
    
    // 创建流水线管理器并执行峰增强
    let pipeline = PipelineManager::new()
        .add_peak_enhancement(&params.boundary_method, config);
    
    let result = match pipeline.execute(container).await {
        Ok(result) => result,
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "峰增强失败", &format!("错误: {}", e));
            }
            return Err(format!("峰增强失败: {}", e));
        }
    };
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    // 更新状态
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Idle);
        app_state.add_message("success", "峰增强完成", &format!("增强了 {} 个峰，耗时 {}ms", result.container.peak_count(), processing_time));
    }
    
    Ok(result.container)
}

/// 步骤4: 曲线还原流水线
#[tauri::command]
pub async fn reconstruct_curves(
    container: SerializableDataContainer,
    params: CurveReconstructionParams,
    _app: tauri::AppHandle,
    state: State<'_, AppStateManager>,
) -> Result<SerializableDataContainer, String> {
    log::info!("📈 开始曲线还原流水线");
    
    // 更新状态
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Analyzing);
        app_state.add_message("info", "曲线还原", "开始还原拟合曲线");
    }
    
    let start_time = std::time::Instant::now();
    
    // 创建曲线还原配置
    let config = serde_json::json!({
        "resolution": params.resolution,
        "include_baseline": params.include_baseline,
        "include_individual_peaks": params.include_individual_peaks
    });
    
    // 创建流水线管理器并执行曲线还原
    let pipeline = PipelineManager::new()
        .add_curve_reconstruction("default", config);
    
    let result = match pipeline.execute(container).await {
        Ok(result) => result,
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "曲线还原失败", &format!("错误: {}", e));
            }
            return Err(format!("曲线还原失败: {}", e));
        }
    };
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    // 更新状态
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Idle);
        app_state.add_message("success", "曲线还原完成", &format!("还原了 {} 条曲线，耗时 {}ms", result.container.curve_count(), processing_time));
    }
    
    Ok(result.container)
}

/// 步骤5: 基线校正流水线
#[tauri::command]
pub async fn baseline_correction_pipeline(
    container: SerializableDataContainer,
    params: BaselineCorrectionParams,
    _app: tauri::AppHandle,
    state: State<'_, AppStateManager>,
) -> Result<SerializableDataContainer, String> {
    log::info!("📏 开始基线校正流水线");
    
    // 更新状态
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Analyzing);
        app_state.add_message("info", "基线校正", &format!("使用 {} 方法校正基线", params.method));
    }
    
    let start_time = std::time::Instant::now();
    
    // 创建基线校正配置
    let mut config = serde_json::json!({
        "method": params.method
    });
    
    // 添加方法特定的参数
    for (key, value) in params.parameters {
        config[key] = value;
    }
    
    // 创建流水线管理器并执行基线校正
    let pipeline = PipelineManager::new()
        .add_baseline_correction(&params.method, config);
    
    let result = match pipeline.execute(container).await {
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
    
    // 更新状态
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Idle);
        app_state.add_message("success", "基线校正完成", &format!("校正了 {} 条曲线，耗时 {}ms", result.container.curve_count(), processing_time));
    }
    
    Ok(result.container)
}

/// 完整流水线执行
#[tauri::command]
pub async fn execute_pipeline(
    container: SerializableDataContainer,
    params: PipelineExecutionParams,
    _app: tauri::AppHandle,
    state: State<'_, AppStateManager>,
) -> Result<PipelineExecutionResult, String> {
    log::info!("🚀 开始执行完整流水线");
    
    // 更新状态
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Analyzing);
        app_state.add_message("info", "流水线执行", &format!("执行 {} 个步骤", params.steps.len()));
    }
    
    let start_time = std::time::Instant::now();
    
    // 创建流水线管理器
    let mut pipeline = PipelineManager::new();
    
    // 添加各个步骤
    for step in params.steps {
        let config = serde_json::to_value(step.config).unwrap_or(serde_json::json!({}));
        
        match step.step_type.as_str() {
            "detection" => {
                pipeline = pipeline.add_peak_detection(&step.method, config);
            }
            "fitting" => {
                pipeline = pipeline.add_peak_fitting(&step.method, config);
            }
            "enhancement" => {
                pipeline = pipeline.add_peak_enhancement(&step.method, config);
            }
            "reconstruction" => {
                pipeline = pipeline.add_curve_reconstruction(&step.method, config);
            }
            "baseline" => {
                pipeline = pipeline.add_baseline_correction(&step.method, config);
            }
            _ => {
                {
                    let mut app_state = state.lock();
                    app_state.add_message("error", "流水线执行失败", &format!("未知的步骤类型: {}", step.step_type));
                }
                return Err(format!("未知的步骤类型: {}", step.step_type));
            }
        }
    }
    
    // 执行流水线
    let result = match pipeline.execute(container).await {
        Ok(result) => result,
        Err(e) => {
            {
                let mut app_state = state.lock();
                app_state.add_message("error", "流水线执行失败", &format!("错误: {}", e));
            }
            return Err(format!("流水线执行失败: {}", e));
        }
    };
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    // 更新状态
    {
        let mut app_state = state.lock();
        app_state.set_processing_status(ProcessingStatus::Idle);
        app_state.add_message("success", "流水线执行完成", &format!("完成了 {} 个步骤，耗时 {}ms", result.steps_completed.len(), processing_time));
    }
    
    Ok(PipelineExecutionResult {
        success: result.success,
        container: result.container,
        execution_time: result.execution_time,
        steps_completed: result.steps_completed,
        error: result.error,
    })
}
