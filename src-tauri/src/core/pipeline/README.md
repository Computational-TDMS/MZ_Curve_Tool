# 峰分析流水线模块

这个模块提供了独立的流水线处理功能，整合各种处理器而不改变现有架构。

## 核心组件

### 1. SerializableDataContainer
专门用于前后端数据传递的可序列化数据容器，不包含不可序列化的字段（如spectra）。

### 2. PipelineManager
流水线管理器，负责协调各种处理器的执行，提供统一的流水线接口。

### 3. Pipeline Commands
提供流水线处理的Tauri命令接口，包括：
- `detect_peaks` - 峰检测流水线
- `fit_peaks` - 峰拟合流水线
- `enhance_peaks` - 峰增强流水线
- `reconstruct_curves` - 曲线还原流水线
- `baseline_correction_pipeline` - 基线校正流水线
- `execute_pipeline` - 完整流水线执行

## 使用示例

### 基本流水线使用

```rust
use crate::core::pipeline::{PipelineManager, SerializableDataContainer};

// 创建流水线管理器
let pipeline = PipelineManager::new()
    .add_peak_detection("cwt", detection_config)
    .add_peak_fitting("gaussian", fitting_config)
    .add_peak_enhancement("default", enhancement_config);

// 执行流水线
let result = pipeline.execute(container).await?;
```

### Tauri命令使用

```typescript
// 前端调用示例
import { invoke } from '@tauri-apps/api/core';

// 峰检测
const containerWithPeaks = await invoke('detect_peaks', {
  container: currentContainer,
  params: {
    method: 'cwt',
    sensitivity: 0.7,
    threshold_multiplier: 3.0,
    min_peak_width: 0.1,
    max_peak_width: 10.0
  }
});

// 峰拟合
const containerWithFittedPeaks = await invoke('fit_peaks', {
  container: containerWithPeaks,
  params: {
    method: 'gaussian',
    min_peak_width: 0.1,
    max_peak_width: 10.0,
    fit_quality_threshold: 0.8
  }
});

// 峰增强
const containerWithEnhancedPeaks = await invoke('enhance_peaks', {
  container: containerWithFittedPeaks,
  params: {
    quality_threshold: 0.5,
    boundary_method: 'adaptive',
    separation_analysis: true
  }
});
```

### 完整流水线执行

```typescript
// 执行完整的流水线
const pipelineResult = await invoke('execute_pipeline', {
  container: initialContainer,
  params: {
    steps: [
      {
        step_type: 'detection',
        method: 'cwt',
        config: {
          sensitivity: 0.7,
          threshold_multiplier: 3.0
        }
      },
      {
        step_type: 'fitting',
        method: 'gaussian',
        config: {
          min_peak_width: 0.1,
          max_peak_width: 10.0
        }
      },
      {
        step_type: 'enhancement',
        method: 'default',
        config: {
          quality_threshold: 0.5
        }
      }
    ]
  }
});
```

## 架构优势

1. **独立性**: 流水线模块独立于现有架构，不破坏现有功能
2. **可扩展性**: 易于添加新的处理步骤和处理器
3. **可测试性**: 每个组件都可以独立测试
4. **类型安全**: 使用Rust的类型系统确保数据安全
5. **序列化友好**: 专门的数据容器确保前后端数据传递的可靠性

## 与现有架构的集成

流水线模块通过以下方式与现有架构集成：

1. **复用现有处理器**: 使用现有的`PeakAnalyzer`、`BaselineProcessor`等
2. **保持API兼容性**: 现有的`analyze_peaks`等命令继续可用
3. **统一数据格式**: 使用`SerializableDataContainer`确保数据一致性
4. **渐进式迁移**: 可以逐步将现有功能迁移到流水线架构

## 测试

运行测试：
```bash
cargo test --package mz_curve_gui_lib --lib core::pipeline::tests
```

测试覆盖：
- 数据容器创建和转换
- 流水线管理器配置
- 元数据操作
- 数据清空功能
