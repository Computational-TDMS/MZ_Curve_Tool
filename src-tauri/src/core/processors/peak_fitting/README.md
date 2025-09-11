# 多峰拟合架构重构总结

## 概述

根据用户需求，我们删除了所有单峰拟合器，只保留多峰拟合功能。新的架构专注于多峰拟合和峰拆分场景，提供了更灵活和强大的峰拟合能力。

## 新架构组件

### 1. 峰形定义器 (`peak_shapes.rs`)

**功能**：
- 定义各种峰形类型（高斯、洛伦兹、伪Voigt、EMG、双高斯、不对称峰）
- 提供峰形参数管理和边界约束
- 实现峰形计算器trait，支持函数值和导数计算
- 自动峰形分析，根据峰特征推荐最佳峰形

**主要结构**：
- `PeakShapeType`: 峰形类型枚举
- `PeakShapeParams`: 峰形参数管理
- `PeakShapeCalculator`: 峰形计算器trait
- `PeakShapeAnalyzer`: 峰形分析器

### 2. 参数优化器 (`parameter_optimizer.rs`)

**功能**：
- 支持多种优化算法：网格搜索、梯度下降、Levenberg-Marquardt、模拟退火
- 统一的优化接口，支持自定义目标函数
- 参数边界约束和误差估计
- 收敛性检测和迭代控制

**主要结构**：
- `OptimizationAlgorithm`: 优化算法枚举
- `ParameterOptimizer`: 参数优化器
- `OptimizationResult`: 优化结果

### 3. 复杂峰形算法 (`advanced_algorithms.rs`)

**功能**：
- 针对特殊峰形的专门算法实现
- EMG（指数修正高斯）算法
- 双高斯峰算法
- 可扩展的算法框架

**主要结构**：
- `AdvancedPeakAlgorithm`: 复杂峰形算法trait
- `EMGAlgorithm`: EMG专门算法
- `BiGaussianAlgorithm`: 双高斯专门算法
- `AdvancedAlgorithmFactory`: 算法工厂

### 4. 多峰拟合器 (`multi_peak_fitter.rs`)

**功能**：
- 多峰同时拟合和峰拆分
- 自动峰检测和峰形分析
- 支持单峰和多峰两种模式
- 联合优化和参数分离
- 拟合质量评估

**主要结构**：
- `MultiPeakFitter`: 多峰拟合器
- `PeakCandidate`: 峰候选
- `MultiPeakOptimizationResult`: 多峰优化结果

## 架构优势

### 1. 模块化设计
- 每个组件职责单一，易于维护和扩展
- 清晰的接口定义，支持组件替换
- 松耦合设计，便于测试和调试

### 2. 算法灵活性
- 支持多种优化算法，可根据需求选择
- 峰形类型可扩展，支持自定义峰形
- 参数化配置，适应不同应用场景

### 3. 性能优化
- 针对不同峰形使用专门算法
- 智能峰形分析，减少不必要的计算
- 参数边界约束，提高收敛速度

### 4. 代码复用
- 统一的峰形计算接口
- 通用的参数优化框架
- 可重用的峰形分析器

## 使用方式

### 基本使用

```rust
use crate::core::processors::peak_fitting::{create_fitter, PeakFitter};

// 创建多峰拟合器
let fitter = create_fitter("multi_peak")?;

// 配置拟合参数
let config = json!({
    "fit_window_size": 3.0,
    "peak_threshold": 0.1,
    "min_peak_distance": 0.5
});

// 执行拟合
let fitted_peak = fitter.fit_peak(&peak, &curve, &config)?;
```

### 高级使用

```rust
use crate::core::processors::peak_fitting::{
    create_fitter_with_optimizer, 
    parameter_optimizer::OptimizationAlgorithm
};

// 使用自定义优化算法
let algorithm = OptimizationAlgorithm::LevenbergMarquardt {
    max_iterations: 100,
    convergence_threshold: 1e-6,
    damping_factor: 0.1,
};

let fitter = create_fitter_with_optimizer("multi_peak", algorithm)?;
```

## 配置参数

### 多峰拟合器配置

- `fit_window_size`: 拟合窗口大小（默认：3.0）
- `peak_threshold`: 峰检测阈值（默认：0.1）
- `min_peak_distance`: 最小峰间距（默认：0.5）

### 优化算法配置

- **网格搜索**: `resolution`（分辨率）, `max_iterations`（最大迭代次数）
- **梯度下降**: `learning_rate`（学习率）, `max_iterations`, `convergence_threshold`（收敛阈值）
- **Levenberg-Marquardt**: `max_iterations`, `convergence_threshold`, `damping_factor`（阻尼因子）
- **模拟退火**: `initial_temperature`（初始温度）, `cooling_rate`（冷却率）, `max_iterations`

## 扩展指南

### 添加新的峰形类型

1. 在 `PeakShapeType` 枚举中添加新类型
2. 在 `PeakShapeParams::new()` 中添加参数定义
3. 实现对应的 `PeakShapeCalculator`
4. 在 `PeakShapeCalculatorFactory` 中注册

### 添加新的优化算法

1. 在 `OptimizationAlgorithm` 枚举中添加新算法
2. 在 `ParameterOptimizer::optimize()` 中添加实现
3. 实现对应的优化逻辑

### 添加新的复杂峰形算法

1. 实现 `AdvancedPeakAlgorithm` trait
2. 在 `AdvancedAlgorithmFactory` 中注册
3. 在多峰拟合器中集成使用

## 性能考虑

1. **内存使用**: 多峰拟合需要更多内存存储多个峰参数
2. **计算复杂度**: 联合优化比单峰拟合计算量更大
3. **收敛性**: 复杂峰形可能需要更多迭代次数
4. **参数初始化**: 良好的初始参数估计对收敛很重要

## 测试和验证

使用 `usage_example.rs` 中的示例代码进行测试：

```rust
use crate::core::processors::peak_fitting::usage_example::run_all_examples;

// 运行所有示例
run_all_examples()?;
```

## 总结

新的多峰拟合架构成功实现了：

1. ✅ 删除了所有单峰拟合器
2. ✅ 保留了多峰拟合和峰拆分功能
3. ✅ 提供了灵活的峰形定义和参数优化
4. ✅ 支持复杂峰形的专门算法
5. ✅ 实现了模块化和可扩展的设计
6. ✅ 提供了完整的使用示例和文档

这个架构为质谱数据的多峰拟合和峰拆分提供了强大而灵活的工具，同时保持了代码的清晰性和可维护性。
