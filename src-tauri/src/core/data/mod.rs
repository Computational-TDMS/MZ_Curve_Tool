//! Data structures for mass spectrometry analysis
//! 
//! This module contains all the core data structures used throughout the IMSFinder application.
//! The structures are organized into separate files for better maintainability:
//! 
//! - `container.rs`: Main data container for holding spectra, curves, and peaks
//! - `curve.rs`: Curve data structure with scientific parameters
//! - `peak.rs`: Peak data structure with high-precision parameters
//! - `processing.rs`: Processing results, errors, and configuration

pub mod container;
pub mod curve;
pub mod peak;
pub mod processing;

// Re-export the main types for convenience
pub use container::DataContainer;
pub use curve::Curve;
pub use peak::{Peak, PeakType, DetectionAlgorithm};
pub use processing::{ProcessingResult, ProcessingError, ProcessingProgress, ProcessingConfig, ProcessingStatus};

/// 处理请求参数
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProcessingRequest {
    pub file_path: String,
    pub mz_range: String,
    pub rt_range: String,
    pub ms_level: u8,
    pub mode: String,
}
