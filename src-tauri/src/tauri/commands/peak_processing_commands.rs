//! 峰处理命令
//! 
//! 提供前端调用峰处理工作流的Tauri命令

use tauri::State;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::core::data::{Curve, Peak, ProcessingError};
use crate::core::processors::peak_fitting::controllers::{
    PeakProcessingController, ProcessingStrategy, ComponentType
};
use crate::tauri::state::AppStateManager;
use serde_json::Value;

/// 峰处理请求
#[derive(Debug, Serialize, Deserialize)]
pub struct PeakProcessingRequest {
    /// 峰列表
    pub peaks: Vec<Peak>,
    /// 曲线数据
    pub curve: Curve,
    /// 处理模式
    pub mode: ProcessingMode,
    /// 用户配置
    pub config: Option<Value>,
    /// 手动覆盖（仅混合模式使用）
    pub manual_overrides: Option<HashMap<String, String>>,
}

/// 处理模式
#[derive(Debug, Serialize, Deserialize)]
pub enum ProcessingMode {
    /// 自动模式
    Automatic,
    /// 手动模式
    Manual {
        strategy: ProcessingStrategyRequest,
    },
    /// 混合模式
    Hybrid {
        manual_overrides: HashMap<String, String>,
    },
    /// 预定义策略模式
    Predefined {
        strategy_name: String,
    },
}

/// 处理策略请求
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStrategyRequest {
    pub name: String,
    pub description: String,
    pub peak_detection: String,
    pub overlap_processing: String,
    pub fitting_method: String,
    pub optimization_algorithm: String,
    pub advanced_algorithm: Option<String>,
    pub post_processing: Option<String>,
    pub configuration: Value,
}

impl From<ProcessingStrategyRequest> for ProcessingStrategy {
    fn from(req: ProcessingStrategyRequest) -> Self {
        ProcessingStrategy::new(req.name, req.description)
            .with_peak_detection(req.peak_detection)
            .with_overlap_processing(req.overlap_processing)
            .with_fitting_method(req.fitting_method)
            .with_optimization_algorithm(req.optimization_algorithm)
            .with_advanced_algorithm(req.advanced_algorithm.unwrap_or_default())
            .with_post_processing(req.post_processing.unwrap_or_default())
            .with_configuration(req.configuration)
    }
}

/// 峰处理响应
#[derive(Debug, Serialize, Deserialize)]
pub struct PeakProcessingResponse {
    /// 处理后的峰列表
    pub peaks: Vec<Peak>,
    /// 处理统计信息
    pub statistics: ProcessingStatistics,
    /// 处理日志
    pub logs: Vec<String>,
    /// 是否成功
    pub success: bool,
    /// 错误信息（如果有）
    pub error: Option<String>,
}

/// 处理统计信息
#[derive(Debug, Serialize, Deserialize)]
pub struct ProcessingStatistics {
    /// 输入峰数量
    pub input_peak_count: usize,
    /// 输出峰数量
    pub output_peak_count: usize,
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
    /// 使用的策略名称
    pub strategy_name: String,
    /// 质量分数
    pub quality_score: f64,
    /// 各阶段执行时间
    pub stage_times: HashMap<String, u64>,
}

/// 组件信息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct ComponentInfo {
    pub component_type: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
}

/// 策略信息响应
#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyInfo {
    pub name: String,
    pub description: String,
    pub peak_detection: String,
    pub overlap_processing: String,
    pub fitting_method: String,
    pub optimization_algorithm: String,
    pub advanced_algorithm: Option<String>,
    pub post_processing: Option<String>,
}

/// 处理峰数据
#[tauri::command]
pub async fn process_peaks(
    request: PeakProcessingRequest,
    state_manager: State<'_, AppStateManager>,
) -> Result<PeakProcessingResponse, String> {
    let start_time = std::time::Instant::now();
    let mut logs = Vec::new();
    
    logs.push(format!("开始处理峰数据，输入峰数量: {}", request.peaks.len()));
    
    // 获取峰处理控制器
    let controller_arc = state_manager.get_peak_processing_controller_arc();
    let controller_guard = controller_arc.lock().map_err(|e| format!("无法获取控制器锁: {}", e))?;
    let controller = controller_guard.as_ref().ok_or("峰处理控制器未初始化")?;
    
    let result = match &request.mode {
        ProcessingMode::Automatic => {
            logs.push("使用自动模式处理".to_string());
            controller.process_automatic(&request.peaks, &request.curve, request.config.as_ref())
        },
        ProcessingMode::Manual { strategy } => {
            logs.push(format!("使用手动模式处理，策略: {}", strategy.name));
            let strategy = ProcessingStrategy::from(strategy.clone());
            controller.process_manual(&request.peaks, &request.curve, strategy, request.config.as_ref())
        },
        ProcessingMode::Hybrid { manual_overrides } => {
            logs.push("使用混合模式处理".to_string());
            controller.process_hybrid(&request.peaks, &request.curve, manual_overrides.clone(), request.config.as_ref())
        },
        ProcessingMode::Predefined { strategy_name } => {
            logs.push(format!("使用预定义策略处理: {}", strategy_name));
            controller.process_with_predefined_strategy(&request.peaks, &request.curve, strategy_name, request.config.as_ref())
        },
    };
    
    let processing_time = start_time.elapsed().as_millis() as u64;
    
    match result {
        Ok(peaks) => {
            logs.push(format!("处理完成，输出峰数量: {}", peaks.len()));
            
            let statistics = ProcessingStatistics {
                input_peak_count: request.peaks.len(),
                output_peak_count: peaks.len(),
                processing_time_ms: processing_time,
                strategy_name: match &request.mode {
                    ProcessingMode::Manual { strategy } => strategy.name.clone(),
                    ProcessingMode::Predefined { strategy_name } => strategy_name.clone(),
                    _ => "auto".to_string(),
                },
                quality_score: calculate_quality_score(&peaks),
                stage_times: HashMap::new(), // TODO: 从工作流控制器获取
            };
            
            Ok(PeakProcessingResponse {
                peaks,
                statistics,
                logs,
                success: true,
                error: None,
            })
        },
        Err(e) => {
            logs.push(format!("处理失败: {}", e));
            let input_peak_count = request.peaks.len();
            Ok(PeakProcessingResponse {
                peaks: request.peaks,
                statistics: ProcessingStatistics {
                    input_peak_count,
                    output_peak_count: 0,
                    processing_time_ms: processing_time,
                    strategy_name: "failed".to_string(),
                    quality_score: 0.0,
                    stage_times: HashMap::new(),
                },
                logs,
                success: false,
                error: Some(e.to_string()),
            })
        }
    }
}

/// 获取可用组件列表
#[tauri::command]
pub async fn get_available_components(
    state_manager: State<'_, AppStateManager>,
) -> Result<Vec<ComponentInfo>, String> {
    let controller_arc = state_manager.get_peak_processing_controller_arc();
    let controller_guard = controller_arc.lock().map_err(|e| format!("无法获取控制器锁: {}", e))?;
    let controller = controller_guard.as_ref().ok_or("峰处理控制器未初始化")?;
    
    let components = controller.list_available_components();
    
    let component_infos: Vec<ComponentInfo> = components
        .iter()
        .map(|desc| ComponentInfo {
            component_type: format!("{:?}", desc.component_type),
            name: desc.name.clone(),
            version: desc.version.clone(),
            description: desc.description.clone(),
            capabilities: desc.capabilities.clone(),
        })
        .collect();
    
    Ok(component_infos)
}

/// 获取可用策略列表
#[tauri::command]
pub async fn get_available_strategies(
    state_manager: State<'_, AppStateManager>,
) -> Result<Vec<StrategyInfo>, String> {
    let controller_arc = state_manager.get_peak_processing_controller_arc();
    let controller_guard = controller_arc.lock().map_err(|e| format!("无法获取控制器锁: {}", e))?;
    let controller = controller_guard.as_ref().ok_or("峰处理控制器未初始化")?;
    
    let strategies = controller.get_available_strategies();
    
    let strategy_infos: Vec<StrategyInfo> = strategies
        .iter()
        .map(|name| {
            // 这里应该从控制器获取策略详情，简化实现
            StrategyInfo {
                name: name.to_string(),
                description: format!("{} 策略", name),
                peak_detection: "advanced_analyzer".to_string(),
                overlap_processing: "fbf_preprocessor".to_string(),
                fitting_method: "multi_peak".to_string(),
                optimization_algorithm: "levenberg_marquardt".to_string(),
                advanced_algorithm: Some("emg_algorithm".to_string()),
                post_processing: Some("quality_validation".to_string()),
            }
        })
        .collect();
    
    Ok(strategy_infos)
}

/// 获取组件详细信息
#[tauri::command]
pub async fn get_component_info(
    component_type: String,
    component_name: String,
    state_manager: State<'_, AppStateManager>,
) -> Result<Option<ComponentInfo>, String> {
    let comp_type = match component_type.as_str() {
        "PeakAnalyzer" => ComponentType::PeakAnalyzer,
        "ParameterOptimizer" => ComponentType::ParameterOptimizer,
        "AdvancedAlgorithm" => ComponentType::AdvancedAlgorithm,
        "OverlapProcessor" => ComponentType::OverlapProcessor,
        "FittingMethod" => ComponentType::FittingMethod,
        "PeakDetector" => ComponentType::PeakDetector,
        "PostProcessor" => ComponentType::PostProcessor,
        _ => return Err(format!("不支持的组件类型: {}", component_type)),
    };
    
    let controller_arc = state_manager.get_peak_processing_controller_arc();
    let controller_guard = controller_arc.lock().map_err(|e| format!("无法获取控制器锁: {}", e))?;
    let controller = controller_guard.as_ref().ok_or("峰处理控制器未初始化")?;
    
    if let Some(desc) = controller.get_component_info(&comp_type, &component_name) {
        Ok(Some(ComponentInfo {
            component_type: format!("{:?}", desc.component_type),
            name: desc.name.clone(),
            version: desc.version.clone(),
            description: desc.description.clone(),
            capabilities: desc.capabilities.clone(),
        }))
    } else {
        Ok(None)
    }
}

/// 验证配置
#[tauri::command]
pub async fn validate_config(
    config_name: String,
    config: Value,
    state_manager: State<'_, AppStateManager>,
) -> Result<bool, String> {
    let controller_arc = state_manager.get_peak_processing_controller_arc();
    let controller_guard = controller_arc.lock().map_err(|e| format!("无法获取控制器锁: {}", e))?;
    let controller = controller_guard.as_ref().ok_or("峰处理控制器未初始化")?;
    
    match controller.validate_config(&config_name, &config) {
        Ok(_) => Ok(true),
        Err(e) => Err(e.to_string()),
    }
}

/// 获取配置架构
#[tauri::command]
pub async fn get_config_schema(
    config_name: String,
    state_manager: State<'_, AppStateManager>,
) -> Result<Option<Value>, String> {
    let controller_arc = state_manager.get_peak_processing_controller_arc();
    let controller_guard = controller_arc.lock().map_err(|e| format!("无法获取控制器锁: {}", e))?;
    let controller = controller_guard.as_ref().ok_or("峰处理控制器未初始化")?;
    
    Ok(controller.get_config_schema(&config_name))
}

/// 初始化峰处理控制器
#[tauri::command]
pub async fn init_peak_processing_controller(
    state_manager: State<'_, AppStateManager>,
) -> Result<String, String> {
    match state_manager.init_peak_processing_controller() {
        Ok(_) => Ok("峰处理控制器初始化成功".to_string()),
        Err(e) => Err(format!("峰处理控制器初始化失败: {}", e)),
    }
}

/// 计算质量分数
fn calculate_quality_score(peaks: &[Peak]) -> f64 {
    if peaks.is_empty() {
        return 0.0;
    }
    
    let total_score: f64 = peaks.iter()
        .map(|peak| peak.get_quality_score())
        .sum();
    
    total_score / peaks.len() as f64
}
