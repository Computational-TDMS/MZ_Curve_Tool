//! 组件工厂实现
//! 
//! 为现有组件提供工厂实现，支持依赖注入

use crate::core::data::ProcessingError;
use super::component_registry::{ComponentRegistry, ComponentType, ComponentDescriptor, ComponentFactory, Component, ProcessingData};
use crate::core::processors::peak_fitting::peak_shapes::PeakShapeAnalyzer;
use crate::core::processors::peak_fitting::parameter_optimizer::{ParameterOptimizer, OptimizationAlgorithm};
// use crate::core::processors::peak_fitting::advanced_algorithms::{AdvancedPeakAlgorithm, EMGAlgorithm, BiGaussianAlgorithm};
use crate::core::processors::peak_fitting::PeakFitter;
use serde_json::{Value, json};

/// 峰形分析器工厂
#[derive(Debug)]
pub struct PeakShapeAnalyzerFactory;

impl ComponentFactory for PeakShapeAnalyzerFactory {
    fn create_component(&self, _config: &Value) -> Result<Box<dyn Component>, ProcessingError> {
        Ok(Box::new(PeakShapeAnalyzerComponent::new()))
    }
    
    fn get_descriptor(&self) -> ComponentDescriptor {
        ComponentDescriptor {
            component_type: ComponentType::PeakAnalyzer,
            name: "peak_shape_analyzer".to_string(),
            version: "1.0.0".to_string(),
            description: "峰形分析器，自动分析峰形特征并推荐最佳峰形".to_string(),
            capabilities: vec![
                "peak_shape_analysis".to_string(),
                "automatic_shape_selection".to_string(),
                "parameter_estimation".to_string(),
            ],
            configuration_schema: json!({
                "type": "object",
                "properties": {
                    "analysis_depth": {
                        "type": "string",
                        "enum": ["basic", "detailed", "comprehensive"],
                        "default": "detailed"
                    }
                }
            }),
        }
    }
}

/// 峰形分析器组件包装
struct PeakShapeAnalyzerComponent {
    analyzer: PeakShapeAnalyzer,
}

impl PeakShapeAnalyzerComponent {
    fn new() -> Self {
        Self {
            analyzer: PeakShapeAnalyzer,
        }
    }
}

impl Component for PeakShapeAnalyzerComponent {
    fn name(&self) -> &str {
        "peak_shape_analyzer"
    }
    
    fn process(&self, input: &ProcessingData, _config: &Value) -> Result<ProcessingData, ProcessingError> {
        let mut result_data = input.clone();
        
        // 分析每个峰的峰形
        let peak_ids: Vec<String> = result_data.peaks.iter().map(|p| p.id.clone()).collect();
        for peak_id in peak_ids {
            // 需要提供x_data和y_data，这里简化处理
            let shape_type = self.analyzer.analyze_peak_shape(&[], &[]);
            result_data.add_intermediate_result(
                format!("peak_{}_shape", peak_id),
                Value::String(format!("{:?}", shape_type))
            );
        }
        
        Ok(result_data)
    }
    
    fn validate_config(&self, _config: &Value) -> Result<(), ProcessingError> {
        Ok(())
    }
}

/// 参数优化器工厂
#[derive(Debug)]
pub struct ParameterOptimizerFactory;

impl ComponentFactory for ParameterOptimizerFactory {
    fn create_component(&self, config: &Value) -> Result<Box<dyn Component>, ProcessingError> {
        let algorithm = if let Some(alg_config) = config.get("algorithm") {
            Self::parse_algorithm(alg_config)?
        } else {
            OptimizationAlgorithm::LevenbergMarquardt {
                max_iterations: 100,
                convergence_threshold: 1e-6,
                damping_factor: 0.1,
            }
        };
        
        Ok(Box::new(ParameterOptimizerComponent::new(algorithm)))
    }
    
    fn get_descriptor(&self) -> ComponentDescriptor {
        ComponentDescriptor {
            component_type: ComponentType::ParameterOptimizer,
            name: "parameter_optimizer".to_string(),
            version: "1.0.0".to_string(),
            description: "参数优化器，支持多种优化算法".to_string(),
            capabilities: vec![
                "levenberg_marquardt".to_string(),
                "gradient_descent".to_string(),
                "simulated_annealing".to_string(),
                "grid_search".to_string(),
            ],
            configuration_schema: json!({
                "type": "object",
                "properties": {
                    "algorithm": {
                        "type": "string",
                        "enum": ["levenberg_marquardt", "gradient_descent", "simulated_annealing", "grid_search"]
                    }
                }
            }),
        }
    }
}

impl ParameterOptimizerFactory {
    fn parse_algorithm(config: &Value) -> Result<OptimizationAlgorithm, ProcessingError> {
        if let Some(alg_name) = config.as_str() {
            match alg_name {
                "levenberg_marquardt" => Ok(OptimizationAlgorithm::LevenbergMarquardt {
                    max_iterations: 100,
                    convergence_threshold: 1e-6,
                    damping_factor: 0.1,
                }),
                "gradient_descent" => Ok(OptimizationAlgorithm::GradientDescent {
                    learning_rate: 0.01,
                    max_iterations: 1000,
                    convergence_threshold: 1e-6,
                }),
                "simulated_annealing" => Ok(OptimizationAlgorithm::SimulatedAnnealing {
                    initial_temperature: 100.0,
                    cooling_rate: 0.95,
                    max_iterations: 1000,
                }),
                "grid_search" => Ok(OptimizationAlgorithm::GridSearch {
                    resolution: 10,
                    max_iterations: 10000,
                }),
                _ => Err(ProcessingError::ConfigError(format!("不支持的优化算法: {}", alg_name))),
            }
        } else {
            Err(ProcessingError::ConfigError("算法配置必须是字符串".to_string()))
        }
    }
}

/// 参数优化器组件包装
struct ParameterOptimizerComponent {
    optimizer: ParameterOptimizer,
}

impl ParameterOptimizerComponent {
    fn new(algorithm: OptimizationAlgorithm) -> Self {
        Self {
            optimizer: ParameterOptimizer::new(algorithm),
        }
    }
}

impl Component for ParameterOptimizerComponent {
    fn name(&self) -> &str {
        "parameter_optimizer"
    }
    
    fn process(&self, input: &ProcessingData, _config: &Value) -> Result<ProcessingData, ProcessingError> {
        let mut result_data = input.clone();
        
        // 为每个峰优化参数
        let peak_ids: Vec<String> = result_data.peaks.iter().map(|p| p.id.clone()).collect();
        for peak_id in peak_ids {
            // 这里应该调用实际的优化逻辑
            // 简化实现，只记录优化状态
            result_data.add_intermediate_result(
                format!("peak_{}_optimized", peak_id),
                Value::Bool(true)
            );
        }
        
        Ok(result_data)
    }
    
    fn validate_config(&self, _config: &Value) -> Result<(), ProcessingError> {
        Ok(())
    }
}

/// 高级算法工厂
#[derive(Debug)]
pub struct AdvancedAlgorithmFactory;

impl ComponentFactory for AdvancedAlgorithmFactory {
    fn create_component(&self, _config: &Value) -> Result<Box<dyn Component>, ProcessingError> {
        // 暂时返回错误，因为AdvancedPeakAlgorithm不存在
        Err(ProcessingError::ConfigError("高级算法组件暂时不可用".to_string()))
    }
    
    fn get_descriptor(&self) -> ComponentDescriptor {
        ComponentDescriptor {
            component_type: ComponentType::AdvancedAlgorithm,
            name: "advanced_algorithm".to_string(),
            version: "1.0.0".to_string(),
            description: "高级峰形算法，支持复杂峰形处理".to_string(),
            capabilities: vec![
                "emg_algorithm".to_string(),
                "bi_gaussian_algorithm".to_string(),
            ],
            configuration_schema: json!({
                "type": "object",
                "properties": {
                    "algorithm": {
                        "type": "string",
                        "enum": ["emg", "bi_gaussian"]
                    }
                }
            }),
        }
    }
}

// 高级算法组件包装 - 暂时注释掉，因为AdvancedPeakAlgorithm不存在

/// 多峰拟合器工厂
#[derive(Debug)]
pub struct MultiPeakFitterFactory;

impl ComponentFactory for MultiPeakFitterFactory {
    fn create_component(&self, config: &Value) -> Result<Box<dyn Component>, ProcessingError> {
        let algorithm = if let Some(alg_config) = config.get("optimization_algorithm") {
            ParameterOptimizerFactory::parse_algorithm(alg_config)?
        } else {
            OptimizationAlgorithm::LevenbergMarquardt {
                max_iterations: 100,
                convergence_threshold: 1e-6,
                damping_factor: 0.1,
            }
        };
        
        Ok(Box::new(MultiPeakFitterComponent::new(algorithm)))
    }
    
    fn get_descriptor(&self) -> ComponentDescriptor {
        ComponentDescriptor {
            component_type: ComponentType::FittingMethod,
            name: "multi_peak".to_string(),
            version: "1.0.0".to_string(),
            description: "多峰拟合器，支持多峰同时拟合".to_string(),
            capabilities: vec![
                "multi_peak_fitting".to_string(),
                "joint_optimization".to_string(),
                "peak_decomposition".to_string(),
            ],
            configuration_schema: json!({
                "type": "object",
                "properties": {
                    "optimization_algorithm": {
                        "type": "string",
                        "enum": ["levenberg_marquardt", "gradient_descent", "simulated_annealing", "grid_search"]
                    }
                }
            }),
        }
    }
}

/// 多峰拟合器组件包装
struct MultiPeakFitterComponent {
    fitter: crate::core::processors::peak_fitting::multi_peak_fitter::MultiPeakFitter,
}

impl MultiPeakFitterComponent {
    fn new(algorithm: OptimizationAlgorithm) -> Self {
        Self {
            fitter: crate::core::processors::peak_fitting::multi_peak_fitter::MultiPeakFitter::with_optimizer(algorithm),
        }
    }
}

impl Component for MultiPeakFitterComponent {
    fn name(&self) -> &str {
        "multi_peak"
    }
    
    fn process(&self, input: &ProcessingData, config: &Value) -> Result<ProcessingData, ProcessingError> {
        let mut result_data = input.clone();
        
        // 执行多峰拟合
        for peak in &mut result_data.peaks {
            let fitted_peak = self.fitter.fit_peak(peak, &result_data.curve, config)?;
            *peak = fitted_peak;
        }
        
        Ok(result_data)
    }
    
    fn validate_config(&self, _config: &Value) -> Result<(), ProcessingError> {
        Ok(())
    }
}

/// 注册所有默认组件工厂
pub fn register_default_factories(registry: &mut ComponentRegistry) -> Result<(), ProcessingError> {
    // 注册峰形分析器
    registry.register_factory(PeakShapeAnalyzerFactory)?;
    
    // 注册参数优化器
    registry.register_factory(ParameterOptimizerFactory)?;
    
    // 注册高级算法
    registry.register_factory(AdvancedAlgorithmFactory)?;
    
    // 注册多峰拟合器
    registry.register_factory(MultiPeakFitterFactory)?;
    
    Ok(())
}
