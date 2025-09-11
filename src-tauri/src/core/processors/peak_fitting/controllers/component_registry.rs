//! 组件注册器
//! 
//! 负责注册、管理和检索各种处理组件

use std::collections::HashMap;
use crate::core::data::{Curve, Peak, ProcessingError};
use serde_json::Value;

/// 组件类型枚举
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComponentType {
    /// 峰形分析器
    PeakAnalyzer,
    /// 参数优化器
    ParameterOptimizer,
    /// 高级算法
    AdvancedAlgorithm,
    /// 重叠峰处理器
    OverlapProcessor,
    /// 拟合方法
    FittingMethod,
    /// 峰检测器
    PeakDetector,
    /// 后处理器
    PostProcessor,
}

/// 组件描述符
#[derive(Debug, Clone)]
pub struct ComponentDescriptor {
    pub component_type: ComponentType,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub configuration_schema: Value,
}

/// 组件工厂trait
pub trait ComponentFactory: Send + Sync + std::fmt::Debug {
    fn create_component(&self, config: &Value) -> Result<Box<dyn Component>, ProcessingError>;
    fn get_descriptor(&self) -> ComponentDescriptor;
}

/// 组件trait
pub trait Component: Send + Sync {
    fn name(&self) -> &str;
    fn process(&self, input: &ProcessingData, config: &Value) -> Result<ProcessingData, ProcessingError>;
    fn validate_config(&self, config: &Value) -> Result<(), ProcessingError>;
}

/// 处理数据
#[derive(Debug, Clone)]
pub struct ProcessingData {
    pub peaks: Vec<Peak>,
    pub curve: Curve,
    pub metadata: HashMap<String, Value>,
    pub intermediate_results: HashMap<String, Value>,
}

impl ProcessingData {
    pub fn new(peaks: Vec<Peak>, curve: Curve) -> Self {
        Self {
            peaks,
            curve,
            metadata: HashMap::new(),
            intermediate_results: HashMap::new(),
        }
    }
    
    pub fn with_metadata(mut self, metadata: HashMap<String, Value>) -> Self {
        self.metadata = metadata;
        self
    }
    
    pub fn add_intermediate_result(&mut self, key: String, value: Value) {
        self.intermediate_results.insert(key, value);
    }
    
    pub fn get_intermediate_result(&self, key: &str) -> Option<&Value> {
        self.intermediate_results.get(key)
    }
}

/// 组件注册器
#[derive(Debug)]
pub struct ComponentRegistry {
    factories: HashMap<(ComponentType, String), Box<dyn ComponentFactory>>,
    descriptors: HashMap<(ComponentType, String), ComponentDescriptor>,
}

impl ComponentRegistry {
    pub fn new() -> Self {
        Self {
            factories: HashMap::new(),
            descriptors: HashMap::new(),
        }
    }
    
    /// 注册组件工厂
    pub fn register_factory<F>(&mut self, factory: F) -> Result<(), ProcessingError>
    where
        F: ComponentFactory + 'static,
    {
        let descriptor = factory.get_descriptor();
        let key = (descriptor.component_type.clone(), descriptor.name.clone());
        
        self.factories.insert(key.clone(), Box::new(factory));
        self.descriptors.insert(key, descriptor);
        
        Ok(())
    }
    
    /// 获取组件实例
    pub fn get_component(
        &self,
        component_type: &ComponentType,
        name: &str,
        config: &Value,
    ) -> Result<Box<dyn Component>, ProcessingError> {
        let key = (component_type.clone(), name.to_string());
        
        let factory = self.factories
            .get(&key)
            .ok_or_else(|| ProcessingError::ConfigError(
                format!("未找到组件: {:?} - {}", component_type, name)
            ))?;
        
        factory.create_component(config)
    }
    
    /// 获取组件描述符
    pub fn get_descriptor(
        &self,
        component_type: &ComponentType,
        name: &str,
    ) -> Option<&ComponentDescriptor> {
        let key = (component_type.clone(), name.to_string());
        self.descriptors.get(&key)
    }
    
    /// 列出所有组件
    pub fn list_components(&self) -> Vec<&ComponentDescriptor> {
        self.descriptors.values().collect()
    }
    
    /// 按类型列出组件
    pub fn list_components_by_type(&self, component_type: &ComponentType) -> Vec<&ComponentDescriptor> {
        self.descriptors
            .values()
            .filter(|desc| desc.component_type == *component_type)
            .collect()
    }
    
    /// 验证组件配置
    pub fn validate_component_config(
        &self,
        component_type: &ComponentType,
        name: &str,
        config: &Value,
    ) -> Result<(), ProcessingError> {
        let component = self.get_component(component_type, name, config)?;
        component.validate_config(config)
    }
}

impl Default for ComponentRegistry {
    fn default() -> Self {
        Self::new()
    }
}
