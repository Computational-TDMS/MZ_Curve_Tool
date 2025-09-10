# 峰分析使用指南

## 概述

本指南介绍如何使用新的峰分析整合系统，该系统提供了智能策略选择、多方法比较和统一的结果接口。

## 核心概念

### 1. 峰分析策略 (PeakAnalysisStrategy)
峰分析策略定义了用于峰检测、拟合和重叠峰处理的具体方法组合。

**预定义策略:**
- `high_quality`: 高质量数据策略 (CWT检测 + 高斯拟合)
- `chromatography`: 色谱数据策略 (简单检测 + EMG拟合 + FBF重叠处理)
- `xps_xrd`: XPS/XRD数据策略 (峰查找器 + Voigt+指数尾拟合)
- `complex_overlap`: 复杂重叠峰策略 (CWT检测 + GMG贝叶斯拟合)
- `asymmetric`: 不对称峰策略 (简单检测 + Pearson-IV拟合)
- `nonlinear`: 非线性峰策略 (CWT检测 + NLC拟合)

### 2. 峰信息整合 (PeakPostProcessor)
峰信息整合器负责:
- 质量评估和评分
- 信息整合和优化
- 统一的结果格式

### 3. 峰分析整合器 (PeakAnalysisIntegrator)
提供统一的峰分析接口，支持:
- 智能策略选择
- 多策略比较
- 结果整合和导出

## 使用方法

### 方法1: 使用新的Pipeline架构 (推荐)

```rust
use crate::core::pipeline::{PipelineManager, SerializableDataContainer};
use serde_json::json;

// 创建流水线管理器
let pipeline = PipelineManager::new()
    .add_peak_detection("cwt", json!({
        "sensitivity": 0.7,
        "threshold_multiplier": 3.0
    }))
    .add_peak_fitting("gaussian", json!({
        "fit_quality_threshold": 0.8
    }));

// 执行流水线
let result = pipeline.execute(container).await?;

// 获取结果
println!("检测到 {} 个峰", result.container.peak_count());
println!("整体质量评分: {:.2}", result.overall_quality);
```

### 方法2: 指定策略分析

```rust
// 创建自定义策略
let strategy = PeakAnalysisStrategy {
    detection_method: "cwt".to_string(),
    fitting_method: "emg".to_string(),
    overlapping_method: "fbf".to_string(),
    confidence: 0.85,
    description: "自定义策略".to_string(),
    use_cases: vec!["质谱".to_string()],
};

// 使用指定策略
let result = integrator.analyze_peaks_with_strategy(data_container, strategy).await?;
```

### 方法3: 多策略比较

```rust
// 获取可用策略
let strategies = integrator.get_available_strategies();

// 选择要比较的策略
let strategies_to_compare = vec![
    strategies[0].clone(),
    strategies[1].clone(),
];

// 执行比较分析
let comparison = integrator.analyze_peaks_compare_strategies(
    data_container, 
    strategies_to_compare
).await?;

// 获取最佳结果
let best_result = &comparison.best_result;
println!("最佳策略: {}", best_result.strategy.description);
```

## 峰信息查询

### 质量筛选
```rust
// 获取高质量峰 (质量评分 ≥ 0.8)
let high_quality_peaks = result.get_high_quality_peaks(0.8);

// 获取优秀峰 (质量评分 ≥ 0.9)
let excellent_peaks = result.get_high_quality_peaks(0.9);
```

### 按峰类型筛选
```rust
use crate::data::PeakType;

// 获取EMG峰
let emg_peaks = result.get_peaks_by_type(&PeakType::EMG);

// 获取Voigt+指数尾峰
let voigt_peaks = result.get_peaks_by_type(&PeakType::VoigtExponentialTail);
```

### 峰面积和峰宽信息
```rust
// 获取峰面积摘要
let area_summary = result.get_area_summary();
println!("总峰面积: {:.2}", area_summary.total_area);
println!("平均峰面积: {:.2}", area_summary.average_area);

// 获取峰宽摘要
let fwhm_summary = result.get_fwhm_summary();
println!("平均FWHM: {:.2}", fwhm_summary.average_fwhm);
```

## 峰信息详解

### ProcessedPeak 结构
每个处理后的峰包含以下信息:

```rust
pub struct ProcessedPeak {
    pub id: String,                    // 峰ID
    pub peak_type: PeakType,           // 峰类型
    pub center: f64,                   // 中心位置
    pub amplitude: f64,                // 振幅
    pub area: f64,                     // 峰面积 (优先使用拟合结果)
    pub fwhm: f64,                     // 半峰宽 (优先使用拟合结果)
    pub sigma: f64,                    // 峰宽参数
    pub quality_score: f64,            // 质量评分 (0.0-1.0)
    pub confidence: f64,               // 置信度 (0.0-1.0)
    pub boundaries: PeakBoundaries,    // 峰边界信息
    pub shape_parameters: ShapeParameters, // 形状参数
    pub fit_info: FitInformation,      // 拟合信息
    pub metadata: HashMap<String, Value>, // 元数据
}
```

### 形状参数 (ShapeParameters)
```rust
pub struct ShapeParameters {
    pub asymmetry_factor: f64,  // 不对称因子
    pub skewness: f64,          // 偏度
    pub kurtosis: f64,          // 峰度
    pub tailing_factor: f64,    // 拖尾因子
    pub resolution: f64,        // 分辨率
}
```

### 拟合信息 (FitInformation)
```rust
pub struct FitInformation {
    pub r_squared: f64,              // R²值
    pub residual_sum_squares: f64,   // 残差平方和
    pub parameter_count: usize,      // 参数数量
    pub parameter_errors: Vec<f64>,  // 参数误差
    pub fit_quality: FitQuality,     // 拟合质量等级
}
```

## 质量评估

### 质量评分计算
峰质量评分基于以下因素:
- 信噪比 (权重: 30%)
- 拟合质量 (权重: 25%)
- 对称性 (权重: 15%)
- 分辨率 (权重: 15%)
- 基线分离 (权重: 15%)

### 质量等级
- **A级** (0.8-1.0): 优秀质量
- **B级** (0.6-0.8): 良好质量
- **C级** (0.4-0.6): 一般质量
- **D级** (0.0-0.4): 较差质量

## 数据导出

### CSV导出
```rust
let csv_data = result.export_to_csv();
println!("{}", csv_data);
```

CSV格式包含以下列:
- ID, Type, Center, Amplitude, Area, FWHM, Sigma
- Quality, Confidence, Asymmetry, Skewness, Kurtosis, R_Squared

### JSON导出
```rust
let json_data = serde_json::to_string_pretty(&result)?;
println!("{}", json_data);
```

## 最佳实践

### 1. 策略选择建议
- **质谱数据**: 使用 `high_quality` 策略
- **色谱数据**: 使用 `chromatography` 策略
- **XPS/XRD数据**: 使用 `xps_xrd` 策略
- **复杂重叠峰**: 使用 `complex_overlap` 策略
- **不确定时**: 使用智能分析或多策略比较

### 2. 质量筛选建议
- 对于定量分析: 使用质量评分 ≥ 0.8 的峰
- 对于定性分析: 使用质量评分 ≥ 0.6 的峰
- 对于探索性分析: 使用质量评分 ≥ 0.4 的峰

### 3. 结果验证
- 检查整体质量评分
- 查看质量报告和建议
- 验证峰数量是否合理
- 检查峰面积分布

## 常见问题

### Q: 如何选择合适的拟合方法？
A: 系统会根据数据特征自动选择，也可以使用多策略比较来找到最佳方法。

### Q: 峰面积和峰宽应该使用哪个值？
A: 系统会优先使用拟合结果，如果没有拟合结果则使用原始测量值。

### Q: 如何处理重叠峰？
A: 系统会自动检测重叠峰并应用相应的处理方法，如FBF、EMG-NLLS等。

### Q: 如何提高峰质量？
A: 可以尝试不同的策略，或者使用多策略比较找到最适合的方法。

## 示例代码

完整的使用示例请参考 `src/examples/peak_analysis_usage.rs` 文件。
