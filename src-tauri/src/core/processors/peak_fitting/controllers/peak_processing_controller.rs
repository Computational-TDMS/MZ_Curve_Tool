//! 峰处理控制器
//! 
//! 统一的峰处理入口，整合所有控制器功能

use std::sync::Arc;
use crate::core::data::{Curve, Peak, ProcessingError};
use super::{
    ComponentRegistry, StrategyController, WorkflowController, ConfigManager,
    ProcessingMode, ProcessingStrategy, WorkflowConfig, ComponentType,
    register_default_factories,
};
use serde_json::{Value, json};

/// 峰处理控制器 - 统一的处理入口
#[derive(Debug)]
pub struct PeakProcessingController {
    registry: Arc<ComponentRegistry>,
    strategy_controller: Arc<StrategyController>,
    workflow_controller: Arc<WorkflowController>,
    config_manager: Arc<ConfigManager>,
}

impl PeakProcessingController {
    /// 创建新的峰处理控制器
    pub fn new() -> Result<Self, ProcessingError> {
        let mut registry = ComponentRegistry::new();
        register_default_factories(&mut registry)?;
        
        let registry = Arc::new(registry);
        let strategy_controller = Arc::new(StrategyController::new(registry.clone()));
        let workflow_controller = Arc::new(WorkflowController::new(registry.clone(), strategy_controller.clone()));
        let config_manager = Arc::new(ConfigManager::new());
        
        Ok(Self {
            registry,
            strategy_controller,
            workflow_controller,
            config_manager,
        })
    }
    
    /// 使用自定义配置创建控制器
    pub fn with_config(config: WorkflowConfig) -> Result<Self, ProcessingError> {
        let mut registry = ComponentRegistry::new();
        register_default_factories(&mut registry)?;
        
        let registry = Arc::new(registry);
        let strategy_controller = Arc::new(StrategyController::new(registry.clone()));
        let workflow_controller = Arc::new(WorkflowController::with_config(
            registry.clone(),
            strategy_controller.clone(),
            config,
        ));
        let config_manager = Arc::new(ConfigManager::new());
        
        Ok(Self {
            registry,
            strategy_controller,
            workflow_controller,
            config_manager,
        })
    }
    
    /// 自动模式处理
    pub fn process_automatic(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        user_config: Option<&Value>,
    ) -> Result<Vec<Peak>, ProcessingError> {
        println!("开始自动模式峰处理");
        
        // 设置自动模式
        let mut strategy_controller = StrategyController::new(self.registry.clone());
        strategy_controller.set_mode(ProcessingMode::Automatic {
            fallback_strategy: ProcessingStrategy::new(
                "default".to_string(),
                "默认策略".to_string()
            ),
        });
        
        // 合并配置
        let config = self.merge_configs(user_config)?;
        
        // 执行工作流
        self.workflow_controller.execute_workflow(peaks, curve, &config)
    }
    
    /// 手动模式处理
    pub fn process_manual(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        strategy: ProcessingStrategy,
        user_config: Option<&Value>,
    ) -> Result<Vec<Peak>, ProcessingError> {
        println!("开始手动模式峰处理，策略: {}", strategy.name);
        
        // 设置手动模式
        let mut strategy_controller = StrategyController::new(self.registry.clone());
        strategy_controller.set_mode(ProcessingMode::Manual {
            strategy: strategy.clone(),
            allow_override: true,
        });
        
        // 合并配置
        let mut config = self.merge_configs(user_config)?;
        
        // 将策略配置合并到用户配置中
        if let Some(config_obj) = config.as_object_mut() {
            if let Some(strategy_config) = strategy.configuration.as_object() {
                for (key, value) in strategy_config {
                    config_obj.insert(format!("strategy_{}", key), value.clone());
                }
            }
        }
        
        // 执行工作流
        self.workflow_controller.execute_workflow(peaks, curve, &config)
    }
    
    /// 混合模式处理
    pub fn process_hybrid(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        manual_overrides: std::collections::HashMap<String, String>,
        user_config: Option<&Value>,
    ) -> Result<Vec<Peak>, ProcessingError> {
        println!("开始混合模式峰处理");
        
        // 设置混合模式
        let mut strategy_controller = StrategyController::new(self.registry.clone());
        strategy_controller.set_mode(ProcessingMode::Hybrid {
            auto_strategy: ProcessingStrategy::new(
                "auto_hybrid".to_string(),
                "自动混合策略".to_string()
            ),
            manual_overrides,
        });
        
        // 合并配置
        let config = self.merge_configs(user_config)?;
        
        // 执行工作流
        self.workflow_controller.execute_workflow(peaks, curve, &config)
    }
    
    /// 使用预定义策略处理
    pub fn process_with_predefined_strategy(
        &self,
        peaks: &[Peak],
        curve: &Curve,
        strategy_name: &str,
        user_config: Option<&Value>,
    ) -> Result<Vec<Peak>, ProcessingError> {
        println!("使用预定义策略处理: {}", strategy_name);
        
        // 获取预定义策略
        let strategy = self.strategy_controller.get_predefined_strategy(strategy_name)
            .ok_or_else(|| ProcessingError::ConfigError(
                format!("未找到预定义策略: {}", strategy_name)
            ))?;
        
        self.process_manual(peaks, curve, strategy.clone(), user_config)
    }
    
    /// 获取可用的预定义策略列表
    pub fn get_available_strategies(&self) -> Vec<&str> {
        self.strategy_controller.list_predefined_strategies()
            .iter()
            .map(|s| s.name.as_str())
            .collect()
    }
    
    /// 获取组件信息
    pub fn get_component_info(&self, component_type: &ComponentType, name: &str) -> Option<&super::component_registry::ComponentDescriptor> {
        self.registry.get_descriptor(component_type, name)
    }
    
    /// 列出所有可用组件
    pub fn list_available_components(&self) -> Vec<&super::component_registry::ComponentDescriptor> {
        self.registry.list_components()
    }
    
    /// 验证配置
    pub fn validate_config(&self, config_name: &str, config: &Value) -> Result<(), ProcessingError> {
        self.config_manager.validate_config(config_name, config)
    }
    
    /// 获取配置架构
    pub fn get_config_schema(&self, config_name: &str) -> Option<Value> {
        self.config_manager.get_config_schema(config_name)
    }
    
    /// 合并配置
    fn merge_configs(&self, user_config: Option<&Value>) -> Result<Value, ProcessingError> {
        let mut merged = json!({});
        
        // 添加默认配置
        match self.config_manager.get_merged_config(&[
            "peak_detection".to_string(),
            "overlap_processing".to_string(),
            "fitting".to_string(),
            "optimization".to_string(),
            "workflow".to_string(),
        ]) {
            Ok(default_configs) => {
                merged = self.merge_config_values(merged, default_configs);
            },
            Err(_) => {
                // 使用默认配置
                merged = json!({
                    "peak_detection": {"method": "advanced_analyzer"},
                    "overlap_processing": {"method": "auto"},
                    "fitting": {"method": "gaussian"},
                    "optimization": {"algorithm": "levenberg_marquardt"},
                    "workflow": {"quality_threshold": 0.8}
                });
            }
        }
        
        // 添加用户配置
        if let Some(user_cfg) = user_config {
            merged = self.merge_config_values(merged, user_cfg.clone());
        }
        
        Ok(merged)
    }
    
    /// 合并配置值
    fn merge_config_values(&self, mut base: Value, override_config: Value) -> Value {
        if let (Some(base_obj), Some(override_obj)) = (base.as_object_mut(), override_config.as_object()) {
            for (key, value) in override_obj {
                if let Some(existing) = base_obj.get_mut(key) {
                    if existing.is_object() && value.is_object() {
                        *existing = self.merge_config_values(existing.clone(), value.clone());
                    } else {
                        *existing = value.clone();
                    }
                } else {
                    base_obj.insert(key.clone(), value.clone());
                }
            }
        }
        
        base
    }
}

impl Default for PeakProcessingController {
    fn default() -> Self {
        Self::new().expect("Failed to create default PeakProcessingController")
    }
}
