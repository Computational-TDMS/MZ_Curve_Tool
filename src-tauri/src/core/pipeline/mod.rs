//! 峰分析流水线模块
//! 
//! 提供独立的流水线处理功能，整合各种处理器而不改变现有架构

pub mod pipeline_manager;
pub mod pipeline_commands;
#[cfg(test)]
mod tests;

pub use pipeline_manager::PipelineManager;
// SerializableDataContainer 现在从 crate::core::data::container 导入
