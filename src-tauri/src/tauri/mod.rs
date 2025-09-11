//! Tauri应用模块
//! 包含Tauri相关的状态管理和命令

pub mod commands;
pub mod state;

// 重新导出 - 避免重复导出
pub use commands::{
    FileInfo, ValidationResult, DataRanges, CurveExtractionParams,
    PeakAnalysisParams, PeakAnalysisResult, BatchProcessingResult, ProgressUpdate,
    ExportResultInfo, ExportParams, CurveDisplayData,
    load_file, validate_file, clear_file_cache, extract_curve, analyze_peaks, batch_process_files,
    get_app_state, update_processing_params, get_processing_status,
    export_curves_to_folder, export_tsv, export_json, export_plot, export_spectro_tsv,
    get_curve_data_for_display, baseline_correction, overlapping_peaks, smooth_data, noise_reduction,
    save_config, load_config, reset_config, get_default_params,
    generate_plot, update_plot, export_plot_image, get_plot_config
};

// 重新导出pipeline命令 - 暂时注释掉，因为pipeline模块不存在
// pub use crate::core::pipeline::pipeline_commands::{
//     PeakDetectionParams, PeakFittingParams, PeakEnhancementParams, 
//     CurveReconstructionParams, BaselineCorrectionParams, PipelineExecutionParams,
//     PipelineStepParams, PipelineExecutionResult,
//     detect_peaks, fit_peaks, enhance_peaks, reconstruct_curves, 
//     baseline_correction_pipeline, execute_pipeline
// };
pub use state::{
    AppState, AppStateManager, ProcessingParams, ProcessingStatus, ProcessingResult,
    ProcessingData, DTCurvePoint, PeakInfo, VisualizationData, PeakData, ChartMetadata,
    MultiCurveData, MultiCurveMetadata, LogMessage, CurveData
};
pub use crate::core::state::CurveMetadata;
