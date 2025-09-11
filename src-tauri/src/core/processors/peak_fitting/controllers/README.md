# 峰拟合控制器架构

## 概述

这是一个全新的控制器架构，提供了高度解耦、可扩展的峰处理系统。支持自动策略选择、手动策略配置和混合模式处理。

## 核心组件

### 1. 组件注册器 (ComponentRegistry)
- 管理所有处理组件的注册和检索
- 支持动态组件加载和配置验证
- 提供组件描述符和功能查询

### 2. 策略控制器 (StrategyController)
- 支持三种处理模式：自动、手动、混合
- 智能策略选择基于峰特征分析
- 预定义策略和自定义策略支持

### 3. 工作流控制器 (WorkflowController)
- 管理多阶段处理流程
- 支持并行执行和错误处理
- 质量评估和结果验证

### 4. 配置管理器 (ConfigManager)
- 统一的配置加载和验证
- 支持文件、内存、环境变量配置源
- 配置合并和架构验证

### 5. 策略构建器 (StrategyBuilder)
- 灵活的策略构建工具
- 预定义策略模板
- 自定义策略规则支持

## 使用方式

### 基本使用

```rust
use crate::core::processors::peak_fitting::PeakProcessingController;

// 创建控制器
let controller = PeakProcessingController::new()?;

// 自动模式处理
let result = controller.process_automatic(&peaks, &curve, None)?;

// 手动模式处理
let strategy = ProcessingStrategy::new("custom".to_string(), "自定义策略".to_string())
    .with_peak_detection("advanced_analyzer".to_string())
    .with_fitting_method("multi_peak".to_string());
let result = controller.process_manual(&peaks, &curve, strategy, None)?;

// 使用预定义策略
let result = controller.process_with_predefined_strategy(
    &peaks, &curve, "high_precision", None
)?;
```

### 自定义策略构建

```rust
use crate::core::processors::peak_fitting::{StrategyBuilder, PredefinedStrategyBuilder};

// 使用构建器创建策略
let strategy = StrategyBuilder::new("my_strategy".to_string(), "我的策略".to_string())
    .with_peak_detection("advanced_analyzer".to_string(), json!({
        "threshold": 0.05
    }))
    .with_fitting_method("multi_peak".to_string(), json!({
        "max_iterations": 200
    }))
    .build()?;

// 使用预定义策略
let strategy = PredefinedStrategyBuilder::build_complex_peaks_strategy()?;
```

### 混合模式处理

```rust
use std::collections::HashMap;

let mut overrides = HashMap::new();
overrides.insert("optimization_algorithm".to_string(), "simulated_annealing".to_string());
overrides.insert("advanced_algorithm".to_string(), "emg_algorithm".to_string());

let result = controller.process_hybrid(&peaks, &curve, overrides, None)?;
```

## 架构优势

1. **高度解耦**: 组件间通过接口交互，易于测试和替换
2. **策略灵活**: 支持自动、手动、混合三种处理模式
3. **配置统一**: 统一的配置管理和验证机制
4. **扩展性强**: 易于添加新的组件和策略
5. **质量保证**: 内置质量评估和结果验证

## 预定义策略

- `simple_peaks`: 简单峰处理策略
- `overlapping_peaks`: 重叠峰处理策略  
- `complex_peaks`: 复杂峰处理策略
- `high_precision`: 高精度处理策略

## 组件类型

- `PeakAnalyzer`: 峰形分析器
- `ParameterOptimizer`: 参数优化器
- `AdvancedAlgorithm`: 高级算法
- `OverlapProcessor`: 重叠峰处理器
- `FittingMethod`: 拟合方法
- `PeakDetector`: 峰检测器
- `PostProcessor`: 后处理器
