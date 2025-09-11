//! 控制器架构模块
//! 
//! 提供组件注册、策略控制、工作流管理等核心功能

pub mod component_registry;
pub mod strategy_controller;
pub mod workflow_controller;
pub mod config_manager;
pub mod strategy_builder;
pub mod component_factories;
pub mod peak_processing_controller;

pub use component_registry::*;
pub use strategy_controller::*;
pub use workflow_controller::*;
pub use config_manager::*;
pub use strategy_builder::*;
pub use component_factories::*;
pub use peak_processing_controller::*;
