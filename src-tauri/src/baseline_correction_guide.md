# 基线校准模块使用指南

## 概述

基线校准模块提供了多种算法来校正质谱数据中的基线漂移，提高数据质量和信噪比。该模块支持线性、多项式、移动平均和非对称最小二乘法等多种基线校准方法。

## 功能特性

- **多种算法支持**: 线性、多项式、移动平均、非对称最小二乘法
- **自动参数选择**: 支持自适应参数选择
- **灵活配置**: 丰富的配置选项和自定义参数
- **高质量输出**: 提供详细的统计信息和质量评估
- **易于集成**: 与现有处理器架构完全兼容

## 支持的算法

### 1. 线性基线校准 (Linear)

最简单的基线校准方法，使用最小二乘法拟合线性基线。

```rust
let config = json!({
    "method": "linear",
    "preserve_original": true,
    "output_baseline": false
});
```

**适用场景**: 基线漂移相对简单，呈线性趋势的数据。

### 2. 多项式基线校准 (Polynomial)

使用多项式拟合基线，可以处理更复杂的基线形状。

```rust
let config = json!({
    "method": "polynomial",
    "degree": 3,  // 多项式次数 (0-10)
    "preserve_original": true,
    "output_baseline": true
});
```

**参数说明**:
- `degree`: 多项式次数，范围0-10，推荐2-4次

**适用场景**: 基线具有明显的非线性特征。

### 3. 移动平均基线校准 (Moving Average)

使用移动平均方法估计基线，支持多种算法变体。

```rust
let config = json!({
    "method": "moving_average",
    "window_size": 21,  // 窗口大小 (必须为奇数)
    "preserve_original": true,
    "output_baseline": true,
    "custom_params": {
        "algorithm": "weighted"  // "simple", "weighted", "adaptive"
    }
});
```

**参数说明**:
- `window_size`: 移动窗口大小，必须≥3且为奇数
- `algorithm`: 算法类型
  - `simple`: 简单移动平均
  - `weighted`: 加权移动平均（高斯权重）
  - `adaptive`: 自适应移动平均

**适用场景**: 基线变化较为平滑，噪声适中的数据。

### 4. 非对称最小二乘法 (Asymmetric Least Squares)

高级基线校准算法，对正负残差给予不同权重。

```rust
let config = json!({
    "method": "asymmetric_least_squares",
    "lambda": 0.0,      // 平滑参数 (0为自动选择)
    "p": 0.0,           // 非对称参数 (0为自动选择)
    "max_iterations": 100,
    "preserve_original": true,
    "output_baseline": true
});
```

**参数说明**:
- `lambda`: 平滑参数，控制基线平滑度，0表示自动选择
- `p`: 非对称参数，控制正负残差的权重差异，0表示自动选择
- `max_iterations`: 最大迭代次数

**适用场景**: 复杂的基线形状，需要高精度校准的数据。

## 使用方法

### 基本使用

```rust
use mz_curve::processors::baseline_correction::BaselineProcessor;
use serde_json::json;

// 创建处理器
let processor = BaselineProcessor::new();

// 配置参数
let config = json!({
    "method": "linear",
    "preserve_original": true,
    "output_baseline": false
});

// 执行基线校准
let result = processor.process(input_data, config).await?;
```

### 快速基线校准

```rust
use mz_curve::processors::baseline_correction::quick_baseline_correction;

// 快速线性基线校准
let result = quick_baseline_correction(input_data, "linear").await?;
```

### 批量处理

```rust
let methods = vec!["linear", "polynomial", "moving_average", "asymmetric_least_squares"];

for method in methods {
    let config = json!({
        "method": method,
        "preserve_original": true,
        "output_baseline": true
    });
    
    let result = processor.process(input_data.clone(), config).await?;
    // 处理结果...
}
```

## 配置参数详解

### 通用参数

| 参数 | 类型 | 默认值 | 说明 |
|------|------|--------|------|
| `method` | string | "linear" | 基线校准方法 |
| `preserve_original` | boolean | true | 是否保留原始数据 |
| `output_baseline` | boolean | false | 是否输出基线曲线 |
| `custom_params` | object | {} | 自定义参数 |

### 方法特定参数

#### 多项式方法
- `degree`: 多项式次数 (0-10)

#### 移动平均方法
- `window_size`: 窗口大小 (≥3的奇数)

#### 非对称最小二乘法
- `lambda`: 平滑参数 (≥0，0为自动)
- `p`: 非对称参数 (0-1，0为自动)
- `max_iterations`: 最大迭代次数 (≥1)

## 输出结果

### 处理结果结构

```rust
pub struct ProcessingResult {
    pub data: DataContainer,        // 处理后的数据
    pub metadata: Value,           // 元数据信息
}
```

### 统计信息

每个处理结果包含详细的统计信息：

```json
{
    "original_baseline": 100.5,      // 原始基线强度
    "corrected_baseline": 0.2,       // 校准后基线强度
    "baseline_offset": 100.3,        // 基线偏移量
    "quality_score": 0.95,           // 质量评分 (0-1)
    "method_used": "Linear",         // 使用的方法
    "processing_time_ms": 15         // 处理时间(毫秒)
}
```

### 质量评估

- **质量评分**: 基于RMSE和信号强度的综合评分 (0-1)
- **信噪比改善**: 校准前后信噪比的变化
- **基线偏移**: 基线强度的变化量

## 最佳实践

### 1. 方法选择指南

- **线性漂移**: 使用线性基线校准
- **简单非线性**: 使用2-3次多项式
- **复杂基线**: 使用非对称最小二乘法
- **平滑基线**: 使用移动平均方法

### 2. 参数调优建议

- **多项式次数**: 从2次开始，逐步增加直到效果不再改善
- **移动平均窗口**: 约为数据点数的1-5%
- **非对称最小二乘法**: 优先使用自动参数选择

### 3. 质量检查

- 检查质量评分是否>0.8
- 确认信噪比有所改善
- 验证基线曲线是否合理

## 错误处理

### 常见错误及解决方案

1. **InsufficientData**: 数据点不足
   - 确保数据点数量满足算法要求
   - 线性方法需要≥2个点
   - 多项式方法需要≥(degree+1)个点

2. **InvalidWindowSize**: 窗口大小无效
   - 确保窗口大小≥3且为奇数
   - 窗口大小不能超过数据点总数

3. **ConvergenceFailed**: 收敛失败
   - 增加最大迭代次数
   - 调整lambda和p参数
   - 尝试其他方法

## 性能优化

### 处理速度对比

1. **线性**: 最快，适合实时处理
2. **移动平均**: 较快，适合批量处理
3. **多项式**: 中等，适合离线分析
4. **非对称最小二乘法**: 较慢，适合高精度需求

### 内存使用

- 所有方法的内存使用量都与数据点数量成正比
- 非对称最小二乘法需要额外的矩阵存储空间

## 示例代码

完整的使用示例请参考 `src/examples/baseline_correction_usage.rs` 文件。

## 扩展开发

### 添加新的基线校准算法

1. 实现 `BaselineAlgorithm` trait
2. 在 `BaselineProcessor` 中注册新算法
3. 更新配置schema和文档

### 自定义参数验证

```rust
impl BaselineAlgorithm for YourAlgorithm {
    fn validate_config(&self, config: &BaselineConfig) -> Result<(), BaselineError> {
        // 实现参数验证逻辑
        Ok(())
    }
}
```
