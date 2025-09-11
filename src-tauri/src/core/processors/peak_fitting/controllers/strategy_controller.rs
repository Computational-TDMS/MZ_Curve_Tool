//! 策略控制器
//! 
//! 负责策略选择、规则管理和策略执行

use std::collections::HashMap;
use std::sync::Arc;
use crate::core::data::{Curve, Peak, ProcessingError};
use super::component_registry::ComponentRegistry;
use serde_json::Value;

/// 处理模式
#[derive(Debug, Clone, PartialEq)]
pub enum ProcessingMode {
    /// 自动模式 - 智能策略选择
    Automatic {
        fallback_strategy: ProcessingStrategy,
    },
    /// 手动模式 - 用户指定策略
    Manual {
        strategy: ProcessingStrategy,
        allow_override: bool,
    },
    /// 混合模式 - 自动选择但允许手动调整
    Hybrid {
        auto_strategy: ProcessingStrategy,
        manual_overrides: HashMap<String, String>,
    },
}

/// 处理策略
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessingStrategy {
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

impl ProcessingStrategy {
    pub fn new(name: String, description: String) -> Self {
        Self {
            name,
            description,
            peak_detection: "default".to_string(),
            overlap_processing: "none".to_string(),
            fitting_method: "multi_peak".to_string(),
            optimization_algorithm: "levenberg_marquardt".to_string(),
            advanced_algorithm: None,
            post_processing: None,
            configuration: Value::Object(serde_json::Map::new()),
        }
    }
    
    pub fn with_peak_detection(mut self, method: String) -> Self {
        self.peak_detection = method;
        self
    }
    
    pub fn with_overlap_processing(mut self, method: String) -> Self {
        self.overlap_processing = method;
        self
    }
    
    pub fn with_fitting_method(mut self, method: String) -> Self {
        self.fitting_method = method;
        self
    }
    
    pub fn with_optimization_algorithm(mut self, algorithm: String) -> Self {
        self.optimization_algorithm = algorithm;
        self
    }
    
    pub fn with_advanced_algorithm(mut self, algorithm: String) -> Self {
        self.advanced_algorithm = Some(algorithm);
        self
    }
    
    pub fn with_post_processing(mut self, method: String) -> Self {
        self.post_processing = Some(method);
        self
    }
    
    pub fn with_configuration(mut self, config: Value) -> Self {
        self.configuration = config;
        self
    }
}

/// 策略规则trait
pub trait StrategyRule: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn evaluate(&self, context: &ProcessingContext) -> f64;
    fn get_recommended_strategy(&self, context: &ProcessingContext) -> ProcessingStrategy;
}

/// 处理上下文
#[derive(Debug, Clone)]
pub struct ProcessingContext {
    pub peaks: Vec<Peak>,
    pub curve: Curve,
    pub peak_count: usize,
    pub overlap_ratio: f64,
    pub signal_to_noise_ratio: f64,
    pub peak_complexity: f64,
    pub data_quality: f64,
    pub user_preferences: HashMap<String, Value>,
}

impl ProcessingContext {
    pub fn new(peaks: Vec<Peak>, curve: Curve) -> Self {
        let peak_count = peaks.len();
        let overlap_ratio = Self::calculate_overlap_ratio(&peaks);
        let signal_to_noise_ratio = Self::calculate_snr(&peaks, &curve);
        let peak_complexity = Self::calculate_peak_complexity(&peaks);
        let data_quality = Self::calculate_data_quality(&curve);
        
        Self {
            peaks,
            curve,
            peak_count,
            overlap_ratio,
            signal_to_noise_ratio,
            peak_complexity,
            data_quality,
            user_preferences: HashMap::new(),
        }
    }
    
    fn calculate_overlap_ratio(peaks: &[Peak]) -> f64 {
        if peaks.len() < 2 {
            return 0.0;
        }
        
        let mut total_overlap = 0.0;
        let mut comparisons = 0;
        
        for i in 0..peaks.len() {
            for j in (i + 1)..peaks.len() {
                let distance = (peaks[i].center - peaks[j].center).abs();
                let avg_width = (peaks[i].fwhm + peaks[j].fwhm) / 2.0;
                let overlap = (avg_width - distance).max(0.0) / avg_width;
                total_overlap += overlap;
                comparisons += 1;
            }
        }
        
        if comparisons > 0 {
            total_overlap / comparisons as f64
        } else {
            0.0
        }
    }
    
    fn calculate_snr(peaks: &[Peak], curve: &Curve) -> f64 {
        if peaks.is_empty() {
            return 0.0;
        }
        
        let max_amplitude = peaks.iter().map(|p| p.amplitude).fold(0.0f64, f64::max);
        let noise_level = Self::estimate_noise_level(curve);
        
        if noise_level > 0.0 {
            max_amplitude / noise_level
        } else {
            100.0 // 假设无噪声
        }
    }
    
    fn estimate_noise_level(curve: &Curve) -> f64 {
        // 简化的噪声估计：使用数据的最小值作为噪声水平
        curve.y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b))
    }
    
    fn calculate_peak_complexity(peaks: &[Peak]) -> f64 {
        if peaks.is_empty() {
            return 0.0;
        }
        
        let width_variance = Self::calculate_width_variance(peaks);
        let asymmetry = Self::calculate_average_asymmetry(peaks);
        
        (width_variance + asymmetry) / 2.0
    }
    
    fn calculate_width_variance(peaks: &[Peak]) -> f64 {
        if peaks.len() < 2 {
            return 0.0;
        }
        
        let mean_width = peaks.iter().map(|p| p.fwhm).sum::<f64>() / peaks.len() as f64;
        let variance = peaks.iter()
            .map(|p| (p.fwhm - mean_width).powi(2))
            .sum::<f64>() / peaks.len() as f64;
        
        variance.sqrt() / mean_width
    }
    
    fn calculate_average_asymmetry(peaks: &[Peak]) -> f64 {
        // 简化的不对称性计算
        peaks.iter().map(|p| p.asymmetry_factor).sum::<f64>() / peaks.len() as f64
    }
    
    fn calculate_data_quality(curve: &Curve) -> f64 {
        // 简化的数据质量评估
        if curve.y_values.is_empty() {
            return 0.0;
        }
        
        let max_val = curve.y_values.iter().fold(0.0f64, |a, &b| a.max(b));
        let min_val = curve.y_values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
        
        if max_val > min_val {
            (max_val - min_val) / max_val
        } else {
            0.0
        }
    }
}

/// 策略控制器
#[derive(Debug)]
pub struct StrategyController {
    registry: Arc<ComponentRegistry>,
    predefined_strategies: HashMap<String, ProcessingStrategy>,
    mode: ProcessingMode,
}

impl StrategyController {
    pub fn new(registry: Arc<ComponentRegistry>) -> Self {
        let mut controller = Self {
            registry,
            predefined_strategies: HashMap::new(),
            mode: ProcessingMode::Automatic {
                fallback_strategy: ProcessingStrategy::new(
                    "default".to_string(),
                    "默认策略".to_string()
                ),
            },
        };
        
        controller.initialize_predefined_strategies();
        controller
    }
    
    /// 设置处理模式
    pub fn set_mode(&mut self, mode: ProcessingMode) {
        self.mode = mode;
    }
    
    /// 添加策略规则
    pub fn add_strategy_rule(&mut self, rule: Box<dyn StrategyRule>) {
        // 暂时注释掉，因为Automatic模式不再包含strategy_rules字段
        // if let ProcessingMode::Automatic { strategy_rules, .. } = &mut self.mode {
        //     strategy_rules.push(rule);
        // }
    }
    
    /// 选择处理策略
    pub fn select_strategy(&self, context: &ProcessingContext) -> Result<ProcessingStrategy, ProcessingError> {
        match &self.mode {
            ProcessingMode::Automatic { fallback_strategy } => {
                // 暂时直接返回fallback_strategy，因为strategy_rules字段已被移除
                Ok(fallback_strategy.clone())
            },
            ProcessingMode::Manual { strategy, .. } => {
                Ok(strategy.clone())
            },
            ProcessingMode::Hybrid { auto_strategy, manual_overrides } => {
                self.select_hybrid_strategy(context, auto_strategy, manual_overrides)
            },
        }
    }
    
    /// 自动策略选择
    fn select_automatic_strategy(
        &self,
        context: &ProcessingContext,
        rules: &[Box<dyn StrategyRule>],
        fallback: &ProcessingStrategy,
    ) -> Result<ProcessingStrategy, ProcessingError> {
        if rules.is_empty() {
            return Ok(fallback.clone());
        }
        
        let mut best_score = 0.0;
        let mut best_strategy = fallback.clone();
        
        for rule in rules {
            let score = rule.evaluate(context);
            if score > best_score {
                best_score = score;
                best_strategy = rule.get_recommended_strategy(context);
            }
        }
        
        Ok(best_strategy)
    }
    
    /// 混合策略选择
    fn select_hybrid_strategy(
        &self,
        context: &ProcessingContext,
        auto_strategy: &ProcessingStrategy,
        manual_overrides: &HashMap<String, String>,
    ) -> Result<ProcessingStrategy, ProcessingError> {
        let mut strategy = auto_strategy.clone();
        
        // 应用手动覆盖
        for (key, value) in manual_overrides {
            match key.as_str() {
                "peak_detection" => strategy.peak_detection = value.clone(),
                "overlap_processing" => strategy.overlap_processing = value.clone(),
                "fitting_method" => strategy.fitting_method = value.clone(),
                "optimization_algorithm" => strategy.optimization_algorithm = value.clone(),
                "advanced_algorithm" => strategy.advanced_algorithm = Some(value.clone()),
                "post_processing" => strategy.post_processing = Some(value.clone()),
                _ => {
                    // 添加到配置中
                    if let Some(config) = strategy.configuration.as_object_mut() {
                        config.insert(key.clone(), Value::String(value.clone()));
                    }
                }
            }
        }
        
        Ok(strategy)
    }
    
    /// 初始化预定义策略
    fn initialize_predefined_strategies(&mut self) {
        // 简单峰策略
        let simple_strategy = ProcessingStrategy::new(
            "simple_peaks".to_string(),
            "简单峰处理策略".to_string()
        )
        .with_peak_detection("basic_analyzer".to_string())
        .with_overlap_processing("none".to_string())
        .with_fitting_method("multi_peak".to_string())
        .with_optimization_algorithm("levenberg_marquardt".to_string());
        
        // 重叠峰策略
        let overlap_strategy = ProcessingStrategy::new(
            "overlapping_peaks".to_string(),
            "重叠峰处理策略".to_string()
        )
        .with_peak_detection("advanced_analyzer".to_string())
        .with_overlap_processing("fbf_preprocessor".to_string())
        .with_fitting_method("multi_peak".to_string())
        .with_optimization_algorithm("levenberg_marquardt".to_string());
        
        // 复杂峰策略
        let complex_strategy = ProcessingStrategy::new(
            "complex_peaks".to_string(),
            "复杂峰处理策略".to_string()
        )
        .with_peak_detection("advanced_analyzer".to_string())
        .with_overlap_processing("extreme_overlap".to_string())
        .with_fitting_method("multi_peak".to_string())
        .with_optimization_algorithm("simulated_annealing".to_string())
        .with_advanced_algorithm("emg_algorithm".to_string());
        
        // 高精度策略
        let high_precision_strategy = ProcessingStrategy::new(
            "high_precision".to_string(),
            "高精度处理策略".to_string()
        )
        .with_peak_detection("advanced_analyzer".to_string())
        .with_overlap_processing("sharpen_cwt".to_string())
        .with_fitting_method("multi_peak".to_string())
        .with_optimization_algorithm("levenberg_marquardt".to_string())
        .with_advanced_algorithm("bi_gaussian".to_string())
        .with_post_processing("quality_validation".to_string());
        
        self.predefined_strategies.insert("simple_peaks".to_string(), simple_strategy);
        self.predefined_strategies.insert("overlapping_peaks".to_string(), overlap_strategy);
        self.predefined_strategies.insert("complex_peaks".to_string(), complex_strategy);
        self.predefined_strategies.insert("high_precision".to_string(), high_precision_strategy);
    }
    
    /// 获取预定义策略
    pub fn get_predefined_strategy(&self, name: &str) -> Option<&ProcessingStrategy> {
        self.predefined_strategies.get(name)
    }
    
    /// 列出所有预定义策略
    pub fn list_predefined_strategies(&self) -> Vec<&ProcessingStrategy> {
        self.predefined_strategies.values().collect()
    }
}
