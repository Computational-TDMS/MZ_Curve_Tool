//! Tauri应用模块
//! 包含Tauri相关的状态管理和命令

pub mod commands;
pub mod state;

// 重新导出 - 避免重复导出
pub use commands::{
    FileInfo, ValidationResult, CurveExtractionParams,
    PeakAnalysisParams, PeakAnalysisResult, BatchProcessingResult, ProgressUpdate,
    load_file, validate_file, extract_curve, analyze_peaks, batch_process_files,
    get_app_state, update_processing_params, get_processing_status
};

// 重新导出pipeline命令
pub use crate::core::pipeline::pipeline_commands::{
    PeakDetectionParams, PeakFittingParams, PeakEnhancementParams, 
    CurveReconstructionParams, BaselineCorrectionParams, PipelineExecutionParams,
    PipelineStepParams, PipelineExecutionResult,
    detect_peaks, fit_peaks, enhance_peaks, reconstruct_curves, 
    baseline_correction_pipeline, execute_pipeline
};
pub use state::{
    AppState, AppStateManager, ProcessingParams, ProcessingStatus, ProcessingResult,
    ProcessingData, DTCurvePoint, PeakInfo, VisualizationData, PeakData, ChartMetadata,
    MultiCurveData, MultiCurveMetadata, DataRanges, LogMessage, CurveData
};
pub use crate::core::state::CurveMetadata;
