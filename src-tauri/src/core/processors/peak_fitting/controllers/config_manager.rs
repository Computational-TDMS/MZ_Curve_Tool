//! 配置管理器
//! 
//! 负责配置的加载、验证、合并和管理

use std::collections::HashMap;
use std::path::Path;
use crate::core::data::ProcessingError;
use serde_json::{Value, json};

/// 配置源类型
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigSource {
    /// 文件配置
    File(String),
    /// 内存配置
    Memory(Value),
    /// 环境变量
    Environment(String),
    /// 默认配置
    Default,
}

/// 配置管理器
#[derive(Debug)]
pub struct ConfigManager {
    configs: HashMap<String, Value>,
    config_sources: HashMap<String, ConfigSource>,
    validation_rules: HashMap<String, Box<dyn ConfigValidator>>,
}

/// 配置验证器trait
pub trait ConfigValidator: Send + Sync + std::fmt::Debug {
    fn validate(&self, config: &Value) -> Result<(), ProcessingError>;
    fn get_schema(&self) -> Value;
}

impl ConfigManager {
    pub fn new() -> Self {
        let mut manager = Self {
            configs: HashMap::new(),
            config_sources: HashMap::new(),
            validation_rules: HashMap::new(),
        };
        
        manager.initialize_default_configs();
        manager.initialize_validation_rules();
        manager
    }
    
    /// 加载配置文件
    pub fn load_config_file(&mut self, name: String, path: &Path) -> Result<(), ProcessingError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ProcessingError::ConfigError(format!("读取配置文件失败: {}", e)))?;
        
        let config: Value = serde_json::from_str(&content)
            .map_err(|e| ProcessingError::ConfigError(format!("解析配置文件失败: {}", e)))?;
        
        self.configs.insert(name.clone(), config);
        self.config_sources.insert(name, ConfigSource::File(path.to_string_lossy().to_string()));
        
        Ok(())
    }
    
    /// 设置内存配置
    pub fn set_config(&mut self, name: String, config: Value) {
        self.configs.insert(name.clone(), config);
        self.config_sources.insert(name, ConfigSource::Memory(json!({})));
    }
    
    /// 从环境变量加载配置
    pub fn load_from_env(&mut self, name: String, env_var: &str) -> Result<(), ProcessingError> {
        let value = std::env::var(env_var)
            .map_err(|_| ProcessingError::ConfigError(format!("环境变量 {} 未设置", env_var)))?;
        
        let config: Value = serde_json::from_str(&value)
            .map_err(|e| ProcessingError::ConfigError(format!("解析环境变量配置失败: {}", e)))?;
        
        self.configs.insert(name.clone(), config);
        self.config_sources.insert(name, ConfigSource::Environment(env_var.to_string()));
        
        Ok(())
    }
    
    /// 获取配置
    pub fn get_config(&self, name: &str) -> Option<&Value> {
        self.configs.get(name)
    }
    
    /// 获取合并后的配置
    pub fn get_merged_config(&self, names: &[String]) -> Result<Value, ProcessingError> {
        let mut merged = json!({});
        
        for name in names {
            if let Some(config) = self.configs.get(name) {
                merged = self.merge_configs(merged, config.clone());
            }
        }
        
        Ok(merged)
    }
    
    /// 合并配置
    fn merge_configs(&self, mut base: Value, override_config: Value) -> Value {
        if let (Some(base_obj), Some(override_obj)) = (base.as_object_mut(), override_config.as_object()) {
            for (key, value) in override_obj {
                if let Some(existing) = base_obj.get_mut(key) {
                    if existing.is_object() && value.is_object() {
                        *existing = self.merge_configs(existing.clone(), value.clone());
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
    
    /// 验证配置
    pub fn validate_config(&self, name: &str, config: &Value) -> Result<(), ProcessingError> {
        if let Some(validator) = self.validation_rules.get(name) {
            validator.validate(config)?;
        }
        
        Ok(())
    }
    
    /// 注册配置验证器
    pub fn register_validator(&mut self, name: String, validator: Box<dyn ConfigValidator>) {
        self.validation_rules.insert(name, validator);
    }
    
    /// 获取配置架构
    pub fn get_config_schema(&self, name: &str) -> Option<Value> {
        self.validation_rules.get(name).map(|v| v.get_schema())
    }
    
    /// 列出所有配置
    pub fn list_configs(&self) -> Vec<&String> {
        self.configs.keys().collect()
    }
    
    /// 获取配置源信息
    pub fn get_config_source(&self, name: &str) -> Option<&ConfigSource> {
        self.config_sources.get(name)
    }
    
    /// 初始化默认配置
    fn initialize_default_configs(&mut self) {
        // 默认峰检测配置
        let peak_detection_config = json!({
            "threshold": 0.1,
            "min_distance": 0.5,
            "window_size": 3.0,
            "smoothing": true,
            "noise_level": 0.05
        });
        
        // 默认重叠峰处理配置
        let overlap_processing_config = json!({
            "method": "fbf",
            "sharpen_strength": 1.0,
            "cwt_scales": [1, 20],
            "noise_threshold": 0.1,
            "max_iterations": 100
        });
        
        // 默认拟合配置
        let fitting_config = json!({
            "method": "multi_peak",
            "max_iterations": 100,
            "convergence_threshold": 1e-6,
            "damping_factor": 0.1,
            "parameter_bounds": {
                "amplitude": [0.0, 10000.0],
                "center": [-1000.0, 1000.0],
                "width": [0.1, 100.0]
            }
        });
        
        // 默认优化配置
        let optimization_config = json!({
            "algorithm": "levenberg_marquardt",
            "max_iterations": 100,
            "convergence_threshold": 1e-6,
            "damping_factor": 0.1,
            "parameter_tolerance": 1e-8,
            "function_tolerance": 1e-8
        });
        
        // 默认高级算法配置
        let advanced_algorithm_config = json!({
            "emg": {
                "tau_range": [0.1, 10.0],
                "initial_tau": 1.0,
                "max_iterations": 200
            },
            "bi_gaussian": {
                "asymmetry_range": [0.1, 5.0],
                "initial_asymmetry": 1.0,
                "max_iterations": 150
            }
        });
        
        // 默认工作流配置
        let workflow_config = json!({
            "stages": [
                "peak_detection",
                "overlap_analysis", 
                "overlap_processing",
                "peak_shape_analysis",
                "fitting",
                "parameter_optimization",
                "post_processing",
                "validation"
            ],
            "parallel_execution": false,
            "error_handling": "stop_on_error",
            "quality_threshold": 0.8,
            "max_iterations": 100
        });
        
        self.configs.insert("peak_detection".to_string(), peak_detection_config);
        self.configs.insert("overlap_processing".to_string(), overlap_processing_config);
        self.configs.insert("fitting".to_string(), fitting_config);
        self.configs.insert("optimization".to_string(), optimization_config);
        self.configs.insert("advanced_algorithm".to_string(), advanced_algorithm_config);
        self.configs.insert("workflow".to_string(), workflow_config);
        
        // 标记为默认配置
        for name in ["peak_detection", "overlap_processing", "fitting", "optimization", "advanced_algorithm", "workflow"] {
            self.config_sources.insert(name.to_string(), ConfigSource::Default);
        }
    }
    
    /// 初始化验证规则
    fn initialize_validation_rules(&mut self) {
        // 峰检测配置验证器
        self.register_validator("peak_detection".to_string(), Box::new(PeakDetectionConfigValidator));
        
        // 拟合配置验证器
        self.register_validator("fitting".to_string(), Box::new(FittingConfigValidator));
        
        // 优化配置验证器
        self.register_validator("optimization".to_string(), Box::new(OptimizationConfigValidator));
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

/// 峰检测配置验证器
#[derive(Debug)]
struct PeakDetectionConfigValidator;

impl ConfigValidator for PeakDetectionConfigValidator {
    fn validate(&self, config: &Value) -> Result<(), ProcessingError> {
        if let Some(threshold) = config.get("threshold") {
            if let Some(t) = threshold.as_f64() {
                if t <= 0.0 || t >= 1.0 {
                    return Err(ProcessingError::ConfigError("threshold 必须在 0.0 到 1.0 之间".to_string()));
                }
            }
        }
        
        if let Some(min_distance) = config.get("min_distance") {
            if let Some(d) = min_distance.as_f64() {
                if d <= 0.0 {
                    return Err(ProcessingError::ConfigError("min_distance 必须大于 0".to_string()));
                }
            }
        }
        
        Ok(())
    }
    
    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "threshold": {
                    "type": "number",
                    "minimum": 0.0,
                    "maximum": 1.0,
                    "description": "峰检测阈值"
                },
                "min_distance": {
                    "type": "number",
                    "minimum": 0.0,
                    "description": "最小峰间距"
                },
                "window_size": {
                    "type": "number",
                    "minimum": 1.0,
                    "description": "检测窗口大小"
                }
            },
            "required": ["threshold", "min_distance"]
        })
    }
}

/// 拟合配置验证器
#[derive(Debug)]
struct FittingConfigValidator;

impl ConfigValidator for FittingConfigValidator {
    fn validate(&self, config: &Value) -> Result<(), ProcessingError> {
        if let Some(max_iterations) = config.get("max_iterations") {
            if let Some(i) = max_iterations.as_u64() {
                if i == 0 || i > 10000 {
                    return Err(ProcessingError::ConfigError("max_iterations 必须在 1 到 10000 之间".to_string()));
                }
            }
        }
        
        if let Some(convergence_threshold) = config.get("convergence_threshold") {
            if let Some(t) = convergence_threshold.as_f64() {
                if t <= 0.0 {
                    return Err(ProcessingError::ConfigError("convergence_threshold 必须大于 0".to_string()));
                }
            }
        }
        
        Ok(())
    }
    
    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "method": {
                    "type": "string",
                    "enum": ["multi_peak", "single_peak"],
                    "description": "拟合方法"
                },
                "max_iterations": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 10000,
                    "description": "最大迭代次数"
                },
                "convergence_threshold": {
                    "type": "number",
                    "minimum": 0.0,
                    "description": "收敛阈值"
                }
            },
            "required": ["method", "max_iterations"]
        })
    }
}

/// 优化配置验证器
#[derive(Debug)]
struct OptimizationConfigValidator;

impl ConfigValidator for OptimizationConfigValidator {
    fn validate(&self, config: &Value) -> Result<(), ProcessingError> {
        if let Some(algorithm) = config.get("algorithm") {
            if let Some(alg) = algorithm.as_str() {
                let valid_algorithms = ["levenberg_marquardt", "gradient_descent", "simulated_annealing", "grid_search"];
                if !valid_algorithms.contains(&alg) {
                    return Err(ProcessingError::ConfigError(
                        format!("不支持的优化算法: {}，支持的算法: {:?}", alg, valid_algorithms)
                    ));
                }
            }
        }
        
        Ok(())
    }
    
    fn get_schema(&self) -> Value {
        json!({
            "type": "object",
            "properties": {
                "algorithm": {
                    "type": "string",
                    "enum": ["levenberg_marquardt", "gradient_descent", "simulated_annealing", "grid_search"],
                    "description": "优化算法"
                },
                "max_iterations": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 10000,
                    "description": "最大迭代次数"
                },
                "convergence_threshold": {
                    "type": "number",
                    "minimum": 0.0,
                    "description": "收敛阈值"
                }
            },
            "required": ["algorithm", "max_iterations"]
        })
    }
}
