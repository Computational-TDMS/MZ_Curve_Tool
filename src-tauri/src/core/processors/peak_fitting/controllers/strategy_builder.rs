//! 策略构建器
//! 
//! 提供灵活的策略构建和自定义功能

use std::collections::HashMap;
use crate::core::data::ProcessingError;
use super::strategy_controller::{ProcessingStrategy, StrategyRule, ProcessingContext};
use super::component_registry::ComponentType;
use serde_json::{Value, json};

/// 策略构建器
#[derive(Debug)]
pub struct StrategyBuilder {
    name: String,
    description: String,
    components: HashMap<String, ComponentDescriptor>,
    configuration: Value,
    rules: Vec<Box<dyn StrategyRule>>,
}

/// 组件描述符
#[derive(Debug, Clone)]
pub struct ComponentDescriptor {
    pub component_type: ComponentType,
    pub name: String,
    pub configuration: Value,
    pub dependencies: Vec<String>,
    pub output_mapping: HashMap<String, String>,
}

impl StrategyBuilder {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            components: HashMap::new(),
            configuration: json!({}),
            rules: Vec::new(),
        }
    }
    
    /// 添加峰检测组件
    pub fn with_peak_detection(mut self, name: String, config: Value) -> Self {
        self.components.insert("peak_detection".to_string(), ComponentDescriptor {
            component_type: ComponentType::PeakDetector,
            name,
            configuration: config,
            dependencies: Vec::new(),
            output_mapping: HashMap::new(),
        });
        self
    }
    
    /// 添加重叠峰处理组件
    pub fn with_overlap_processing(mut self, name: String, config: Value) -> Self {
        self.components.insert("overlap_processing".to_string(), ComponentDescriptor {
            component_type: ComponentType::OverlapProcessor,
            name,
            configuration: config,
            dependencies: vec!["peak_detection".to_string()],
            output_mapping: HashMap::new(),
        });
        self
    }
    
    /// 添加拟合方法组件
    pub fn with_fitting_method(mut self, name: String, config: Value) -> Self {
        self.components.insert("fitting_method".to_string(), ComponentDescriptor {
            component_type: ComponentType::FittingMethod,
            name,
            configuration: config,
            dependencies: vec!["overlap_processing".to_string()],
            output_mapping: HashMap::new(),
        });
        self
    }
    
    /// 添加参数优化组件
    pub fn with_parameter_optimizer(mut self, name: String, config: Value) -> Self {
        self.components.insert("parameter_optimizer".to_string(), ComponentDescriptor {
            component_type: ComponentType::ParameterOptimizer,
            name,
            configuration: config,
            dependencies: vec!["fitting_method".to_string()],
            output_mapping: HashMap::new(),
        });
        self
    }
    
    /// 添加高级算法组件
    pub fn with_advanced_algorithm(mut self, name: String, config: Value) -> Self {
        self.components.insert("advanced_algorithm".to_string(), ComponentDescriptor {
            component_type: ComponentType::AdvancedAlgorithm,
            name,
            configuration: config,
            dependencies: vec!["parameter_optimizer".to_string()],
            output_mapping: HashMap::new(),
        });
        self
    }
    
    /// 添加后处理组件
    pub fn with_post_processing(mut self, name: String, config: Value) -> Self {
        self.components.insert("post_processing".to_string(), ComponentDescriptor {
            component_type: ComponentType::PostProcessor,
            name,
            configuration: config,
            dependencies: vec!["advanced_algorithm".to_string()],
            output_mapping: HashMap::new(),
        });
        self
    }
    
    /// 添加策略规则
    pub fn with_rule(mut self, rule: Box<dyn StrategyRule>) -> Self {
        self.rules.push(rule);
        self
    }
    
    /// 设置全局配置
    pub fn with_global_config(mut self, config: Value) -> Self {
        self.configuration = config;
        self
    }
    
    /// 构建策略
    pub fn build(self) -> Result<ProcessingStrategy, ProcessingError> {
        // 验证组件依赖
        self.validate_dependencies()?;
        
        // 构建策略
        let name = self.name.clone();
        let description = self.description.clone();
        let mut strategy = ProcessingStrategy::new(name, description);
        
        // 设置组件
        if let Some(peak_detection) = self.components.get("peak_detection") {
            strategy.peak_detection = peak_detection.name.clone();
        }
        
        if let Some(overlap_processing) = self.components.get("overlap_processing") {
            strategy.overlap_processing = overlap_processing.name.clone();
        }
        
        if let Some(fitting_method) = self.components.get("fitting_method") {
            strategy.fitting_method = fitting_method.name.clone();
        }
        
        if let Some(parameter_optimizer) = self.components.get("parameter_optimizer") {
            strategy.optimization_algorithm = parameter_optimizer.name.clone();
        }
        
        if let Some(advanced_algorithm) = self.components.get("advanced_algorithm") {
            strategy.advanced_algorithm = Some(advanced_algorithm.name.clone());
        }
        
        if let Some(post_processing) = self.components.get("post_processing") {
            strategy.post_processing = Some(post_processing.name.clone());
        }
        
        // 合并配置
        let merged_config = self.merge_component_configs();
        strategy.configuration = merged_config;
        
        Ok(strategy)
    }
    
    /// 验证组件依赖
    fn validate_dependencies(&self) -> Result<(), ProcessingError> {
        for (component_name, descriptor) in &self.components {
            for dependency in &descriptor.dependencies {
                if !self.components.contains_key(dependency) {
                    return Err(ProcessingError::ConfigError(
                        format!("组件 {} 依赖的组件 {} 未定义", component_name, dependency)
                    ));
                }
            }
        }
        Ok(())
    }
    
    /// 合并组件配置
    fn merge_component_configs(&self) -> Value {
        let mut merged = self.configuration.clone();
        
        for descriptor in self.components.values() {
            if let (Some(merged_obj), Some(component_config)) = (merged.as_object_mut(), descriptor.configuration.as_object()) {
                for (key, value) in component_config {
                    merged_obj.insert(format!("{}_{}", descriptor.name, key), value.clone());
                }
            }
        }
        
        merged
    }
}

/// 预定义策略构建器
pub struct PredefinedStrategyBuilder;

impl PredefinedStrategyBuilder {
    /// 构建简单峰策略
    pub fn build_simple_peaks_strategy() -> Result<ProcessingStrategy, ProcessingError> {
        StrategyBuilder::new(
            "simple_peaks".to_string(),
            "简单峰处理策略".to_string()
        )
        .with_peak_detection("basic_analyzer".to_string(), json!({
            "threshold": 0.1,
            "min_distance": 0.5
        }))
        .with_overlap_processing("none".to_string(), json!({}))
        .with_fitting_method("multi_peak".to_string(), json!({
            "max_iterations": 50
        }))
        .with_parameter_optimizer("levenberg_marquardt".to_string(), json!({
            "max_iterations": 100,
            "convergence_threshold": 1e-6
        }))
        .build()
    }
    
    /// 构建重叠峰策略
    pub fn build_overlapping_peaks_strategy() -> Result<ProcessingStrategy, ProcessingError> {
        StrategyBuilder::new(
            "overlapping_peaks".to_string(),
            "重叠峰处理策略".to_string()
        )
        .with_peak_detection("advanced_analyzer".to_string(), json!({
            "threshold": 0.05,
            "min_distance": 0.3
        }))
        .with_overlap_processing("fbf_preprocessor".to_string(), json!({
            "sharpen_strength": 1.2,
            "max_iterations": 100
        }))
        .with_fitting_method("multi_peak".to_string(), json!({
            "max_iterations": 100
        }))
        .with_parameter_optimizer("levenberg_marquardt".to_string(), json!({
            "max_iterations": 150,
            "convergence_threshold": 1e-7
        }))
        .build()
    }
    
    /// 构建复杂峰策略
    pub fn build_complex_peaks_strategy() -> Result<ProcessingStrategy, ProcessingError> {
        StrategyBuilder::new(
            "complex_peaks".to_string(),
            "复杂峰处理策略".to_string()
        )
        .with_peak_detection("advanced_analyzer".to_string(), json!({
            "threshold": 0.03,
            "min_distance": 0.2
        }))
        .with_overlap_processing("extreme_overlap".to_string(), json!({
            "sharpen_strength": 1.5,
            "cwt_scales": [1, 30],
            "max_iterations": 200
        }))
        .with_fitting_method("multi_peak".to_string(), json!({
            "max_iterations": 200
        }))
        .with_parameter_optimizer("simulated_annealing".to_string(), json!({
            "max_iterations": 500,
            "initial_temperature": 100.0,
            "cooling_rate": 0.95
        }))
        .with_advanced_algorithm("emg_algorithm".to_string(), json!({
            "tau_range": [0.1, 10.0],
            "max_iterations": 200
        }))
        .build()
    }
    
    /// 构建高精度策略
    pub fn build_high_precision_strategy() -> Result<ProcessingStrategy, ProcessingError> {
        StrategyBuilder::new(
            "high_precision".to_string(),
            "高精度处理策略".to_string()
        )
        .with_peak_detection("advanced_analyzer".to_string(), json!({
            "threshold": 0.01,
            "min_distance": 0.1
        }))
        .with_overlap_processing("sharpen_cwt".to_string(), json!({
            "sharpen_strength": 2.0,
            "cwt_scales": [1, 50],
            "noise_threshold": 0.05
        }))
        .with_fitting_method("multi_peak".to_string(), json!({
            "max_iterations": 300
        }))
        .with_parameter_optimizer("levenberg_marquardt".to_string(), json!({
            "max_iterations": 500,
            "convergence_threshold": 1e-9,
            "damping_factor": 0.01
        }))
        .with_advanced_algorithm("bi_gaussian".to_string(), json!({
            "asymmetry_range": [0.1, 5.0],
            "max_iterations": 300
        }))
        .with_post_processing("quality_validation".to_string(), json!({
            "quality_threshold": 0.95,
            "validation_method": "comprehensive"
        }))
        .build()
    }
    
    /// 构建自定义策略
    pub fn build_custom_strategy(
        name: String,
        description: String,
        components: HashMap<String, (String, Value)>,
        global_config: Option<Value>,
    ) -> Result<ProcessingStrategy, ProcessingError> {
        let mut builder = StrategyBuilder::new(name, description);
        
        // 添加组件
        if let Some((comp_name, config)) = components.get("peak_detection") {
            builder = builder.with_peak_detection(comp_name.clone(), config.clone());
        }
        
        if let Some((comp_name, config)) = components.get("overlap_processing") {
            builder = builder.with_overlap_processing(comp_name.clone(), config.clone());
        }
        
        if let Some((comp_name, config)) = components.get("fitting_method") {
            builder = builder.with_fitting_method(comp_name.clone(), config.clone());
        }
        
        if let Some((comp_name, config)) = components.get("parameter_optimizer") {
            builder = builder.with_parameter_optimizer(comp_name.clone(), config.clone());
        }
        
        if let Some((comp_name, config)) = components.get("advanced_algorithm") {
            builder = builder.with_advanced_algorithm(comp_name.clone(), config.clone());
        }
        
        if let Some((comp_name, config)) = components.get("post_processing") {
            builder = builder.with_post_processing(comp_name.clone(), config.clone());
        }
        
        // 设置全局配置
        if let Some(config) = global_config {
            builder = builder.with_global_config(config);
        }
        
        builder.build()
    }
}

/// 策略规则构建器
pub struct StrategyRuleBuilder;

impl StrategyRuleBuilder {
    /// 构建重叠度规则
    pub fn build_overlap_rule() -> Box<dyn StrategyRule> {
        Box::new(OverlapStrategyRule)
    }
    
    /// 构建复杂度规则
    pub fn build_complexity_rule() -> Box<dyn StrategyRule> {
        Box::new(ComplexityStrategyRule)
    }
    
    /// 构建信噪比规则
    pub fn build_snr_rule() -> Box<dyn StrategyRule> {
        Box::new(SNRStrategyRule)
    }
    
    /// 构建数据质量规则
    pub fn build_quality_rule() -> Box<dyn StrategyRule> {
        Box::new(DataQualityStrategyRule)
    }
}

/// 重叠度策略规则
#[derive(Debug)]
pub struct OverlapStrategyRule;

impl StrategyRule for OverlapStrategyRule {
    fn name(&self) -> &str {
        "overlap_rule"
    }
    
    fn evaluate(&self, context: &ProcessingContext) -> f64 {
        if context.overlap_ratio < 0.1 {
            0.1 // 低重叠度，使用简单策略
        } else if context.overlap_ratio < 0.5 {
            0.5 // 中等重叠度，使用重叠峰策略
        } else {
            1.0 // 高重叠度，使用复杂策略
        }
    }
    
    fn get_recommended_strategy(&self, context: &ProcessingContext) -> ProcessingStrategy {
        if context.overlap_ratio < 0.1 {
            PredefinedStrategyBuilder::build_simple_peaks_strategy().unwrap()
        } else if context.overlap_ratio < 0.5 {
            PredefinedStrategyBuilder::build_overlapping_peaks_strategy().unwrap()
        } else {
            PredefinedStrategyBuilder::build_complex_peaks_strategy().unwrap()
        }
    }
}

/// 复杂度策略规则
#[derive(Debug)]
pub struct ComplexityStrategyRule;

impl StrategyRule for ComplexityStrategyRule {
    fn name(&self) -> &str {
        "complexity_rule"
    }
    
    fn evaluate(&self, context: &ProcessingContext) -> f64 {
        context.peak_complexity
    }
    
    fn get_recommended_strategy(&self, context: &ProcessingContext) -> ProcessingStrategy {
        if context.peak_complexity < 0.3 {
            PredefinedStrategyBuilder::build_simple_peaks_strategy().unwrap()
        } else if context.peak_complexity < 0.7 {
            PredefinedStrategyBuilder::build_overlapping_peaks_strategy().unwrap()
        } else {
            PredefinedStrategyBuilder::build_complex_peaks_strategy().unwrap()
        }
    }
}

/// 信噪比策略规则
#[derive(Debug)]
pub struct SNRStrategyRule;

impl StrategyRule for SNRStrategyRule {
    fn name(&self) -> &str {
        "snr_rule"
    }
    
    fn evaluate(&self, context: &ProcessingContext) -> f64 {
        if context.signal_to_noise_ratio > 100.0 {
            1.0 // 高信噪比，可以使用高精度策略
        } else if context.signal_to_noise_ratio > 10.0 {
            0.5 // 中信噪比，使用标准策略
        } else {
            0.1 // 低信噪比，使用简单策略
        }
    }
    
    fn get_recommended_strategy(&self, context: &ProcessingContext) -> ProcessingStrategy {
        if context.signal_to_noise_ratio > 100.0 {
            PredefinedStrategyBuilder::build_high_precision_strategy().unwrap()
        } else if context.signal_to_noise_ratio > 10.0 {
            PredefinedStrategyBuilder::build_overlapping_peaks_strategy().unwrap()
        } else {
            PredefinedStrategyBuilder::build_simple_peaks_strategy().unwrap()
        }
    }
}

/// 数据质量策略规则
#[derive(Debug)]
pub struct DataQualityStrategyRule;

impl StrategyRule for DataQualityStrategyRule {
    fn name(&self) -> &str {
        "quality_rule"
    }
    
    fn evaluate(&self, context: &ProcessingContext) -> f64 {
        context.data_quality
    }
    
    fn get_recommended_strategy(&self, context: &ProcessingContext) -> ProcessingStrategy {
        if context.data_quality > 0.8 {
            PredefinedStrategyBuilder::build_high_precision_strategy().unwrap()
        } else if context.data_quality > 0.5 {
            PredefinedStrategyBuilder::build_overlapping_peaks_strategy().unwrap()
        } else {
            PredefinedStrategyBuilder::build_simple_peaks_strategy().unwrap()
        }
    }
}
