# MZ Curve API 使用指南

## 快速开始

### 基本使用 - 新的Pipeline架构

```rust
use mz_curve_gui_lib::core::pipeline::{PipelineManager, SerializableDataContainer};
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. 创建数据容器
    let mut container = SerializableDataContainer::new();
    // ... 添加数据到容器

    // 2. 创建流水线
    let pipeline = PipelineManager::new()
        .add_peak_detection("cwt", json!({
            "sensitivity": 0.7,
            "threshold_multiplier": 3.0
        }))
        .add_peak_fitting("gaussian", json!({
            "fit_quality_threshold": 0.8
        }));

    // 3. 执行流水线
    let result = pipeline.execute(container).await?;
    
    println!("处理完成: {} 条曲线, {} 个峰值", 
             result.container.curve_count(), result.container.peak_count());

    Ok(())
}
```

### 高级峰分析

```rust
use mz_curve::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 使用高级峰分析功能
    let result = process_file_with_peak_analysis(
        "data.mzML",           // 文件路径
        "100.0-200.0",         // m/z范围
        "0.0-60.0",            // 保留时间范围
        1,                     // MS级别
        "cwt",                 // 检测方法
        "gaussian",            // 拟合方法
        Some("fbf"),           // 重叠峰处理方法
    ).await?;

    println!("峰分析完成: {} 个峰值", result.peak_count());
    
    // 导出结果
    let export_results = export_results(
        &result,
        &["tsv".to_string(), "plotly".to_string()],
        Some("output"),
    ).await?;

    println!("导出了 {} 个文件", export_results.len());

    Ok(())
}
```

## 核心API

### 数据处理函数

#### `process_file(request: ProcessingRequest) -> Result<ProcessingResult, ProcessingError>`
基本文件处理函数，支持DT提取等基本功能。

**参数:**
- `request`: 处理请求，包含文件路径、m/z范围、保留时间范围等

**返回:**
- `ProcessingResult`: 包含处理后的曲线和峰数据

#### `process_file_with_peak_analysis(...) -> Result<ProcessingResult, ProcessingError>`
高级峰分析函数，支持完整的峰检测、拟合和重叠峰处理。

**参数:**
- `file_path`: 数据文件路径
- `mz_range`: m/z范围字符串 (格式: "min-max")
- `rt_range`: 保留时间范围字符串 (格式: "min-max")
- `ms_level`: MS级别
- `detection_method`: 峰检测方法 ("cwt", "simple", "peak_finder")
- `fitting_method`: 峰拟合方法 ("gaussian", "lorentzian", "emg", 等)
- `overlapping_method`: 重叠峰处理方法 (可选)

### 数据导出

#### `export_results(result, formats, output_dir) -> Result<Vec<ExportResult>, ProcessingError>`
批量导出处理结果到多种格式。

**参数:**
- `result`: 处理结果
- `formats`: 导出格式列表 (["tsv", "plotly"])
- `output_dir`: 输出目录 (可选)

## 数据结构

### ProcessingRequest
```rust
pub struct ProcessingRequest {
    pub file_path: String,     // 文件路径
    pub mz_range: String,      // m/z范围
    pub rt_range: String,      // 保留时间范围
    pub ms_level: u8,          // MS级别
    pub mode: String,          // 处理模式
}
```

### ProcessingResult
```rust
pub struct ProcessingResult {
    pub curves: Vec<Curve>,    // 曲线数据
    pub peaks: Vec<Peak>,      // 峰数据
    pub metadata: HashMap<String, Value>, // 元数据
}
```

### Curve
```rust
pub struct Curve {
    pub id: String,                    // 曲线ID
    pub curve_type: String,            // 曲线类型
    pub x_values: Vec<f64>,            // X轴数据
    pub y_values: Vec<f64>,            // Y轴数据
    pub x_label: String,               // X轴标签
    pub y_label: String,               // Y轴标签
    pub signal_to_noise_ratio: f64,    // 信噪比
    pub quality_score: f64,            // 质量评分
    // ... 更多字段
}
```

### Peak
```rust
pub struct Peak {
    pub id: String,                    // 峰ID
    pub center: f64,                   // 峰中心
    pub amplitude: f64,                // 峰振幅
    pub area: f64,                     // 峰面积
    pub fwhm: f64,                     // 半高全宽
    pub rsquared: f64,                 // R²值
    pub peak_type: PeakType,           // 峰类型
    pub detection_algorithm: DetectionAlgorithm, // 检测算法
    // ... 更多字段
}
```

## 峰分析配置

### 检测方法
- `"cwt"`: 连续小波变换，适合复杂峰形
- `"simple"`: 简单阈值检测，快速但精度较低
- `"peak_finder"`: 基于导数的峰查找算法

### 拟合方法
- `"gaussian"`: 高斯拟合，适合对称峰
- `"lorentzian"`: 洛伦兹拟合，适合宽峰
- `"pseudo_voigt"`: Pseudo-Voigt拟合，高斯和洛伦兹的混合
- `"emg"`: 指数修正高斯，适合拖尾峰
- `"bi_gaussian"`: 双高斯拟合，适合不对称峰
- `"voigt_exponential_tail"`: Voigt+指数尾，适合XPS/XRD数据
- `"pearson_iv"`: Pearson-IV拟合，适合复杂峰形
- `"nlc"`: 非线性色谱拟合
- `"gmg_bayesian"`: GMG贝叶斯拟合，适合重叠峰

### 重叠峰处理
- `"fbf"`: FBF预处理
- `"sharpen_cwt"`: CWT锐化预处理
- `"emg_nlls"`: EMG非线性最小二乘
- `"extreme_overlap"`: 极度重叠处理器
- `"auto"`: 自动选择最佳方法

## 导出格式

### TSV导出
```rust
let config = serde_json::json!({
    "include_header": true,
    "decimal_precision": 6,
    "include_metadata": true,
    "export_format": "combined"  // "peaks_only", "curves_only", "combined", "summary"
});
```

### Plotly导出
```rust
let config = serde_json::json!({
    "chart_type": "combined",     // "line", "scatter", "bar", "combined"
    "show_peaks": true,
    "show_fit": false,
    "title": "IMS Data Visualization",
    "width": 800,
    "height": 600
});
```

## 峰分析策略

### 自动策略选择
```rust
use mz_curve::PeakAnalysisStrategySelector;

let selector = PeakAnalysisStrategySelector::new();
let strategy = selector.select_strategy(&curve, Some("chromatography"));

println!("推荐策略: {}", strategy.description);
println!("检测方法: {}", strategy.detection_method);
println!("拟合方法: {}", strategy.fitting_method);
println!("置信度: {}", strategy.confidence);
```

### 自定义策略
```rust
use mz_curve::{PeakAnalysisStrategy, PeakAnalysisStrategySelector};

let mut selector = PeakAnalysisStrategySelector::new();

let custom_strategy = PeakAnalysisStrategy {
    detection_method: "cwt".to_string(),
    fitting_method: "emg".to_string(),
    overlapping_method: "fbf".to_string(),
    confidence: 0.9,
    description: "自定义色谱策略".to_string(),
    use_cases: vec!["HPLC".to_string(), "复杂混合物".to_string()],
};

selector.add_custom_strategy("custom_hplc".to_string(), custom_strategy);
```

## 错误处理

```rust
use mz_curve::ProcessingError;

match result {
    Ok(processing_result) => {
        println!("处理成功: {} 个峰", processing_result.peak_count());
    }
    Err(ProcessingError::ConfigError(msg)) => {
        eprintln!("配置错误: {}", msg);
    }
    Err(ProcessingError::DataError(msg)) => {
        eprintln!("数据错误: {}", msg);
    }
    Err(ProcessingError::ProcessError(msg)) => {
        eprintln!("处理错误: {}", msg);
    }
    Err(ProcessingError::MzDataError(msg)) => {
        eprintln!("mzData错误: {}", msg);
    }
}
```

## 完整示例

```rust
use mz_curve::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 初始化日志
    env_logger::init();

    // 1. 加载数据
    println!("加载数据文件...");
    let container = DataLoader::load_from_file("20250103-ZMT-NSP-IMS-VTP1-3.mzML")?;
    println!("加载了 {} 个光谱", container.spectra.len());

    // 2. 创建峰分析器
    let analyzer = PeakAnalyzer::new_with_overlapping_processing(
        "cwt",
        "gaussian", 
        Some("auto")
    )?;

    // 3. 配置分析参数
    let config = serde_json::json!({
        "mz_range": "100.0-500.0",
        "rt_range": "0.0-60.0",
        "ms_level": 1,
        "sensitivity": 0.7,
        "threshold_multiplier": 3.0,
        "min_peak_width": 0.1,
        "max_peak_width": 10.0
    });

    // 4. 执行峰分析
    println!("开始峰分析...");
    let result = analyzer.process(container, config).await?;
    println!("分析完成: {} 条曲线, {} 个峰", 
             result.curve_count(), result.peak_count());

    // 5. 导出结果
    println!("导出结果...");
    let export_results = export_results(
        &result,
        &["tsv".to_string(), "plotly".to_string()],
        Some("output"),
    ).await?;

    for export_result in export_results {
        println!("导出文件: {} ({} bytes)", 
                 export_result.filename, export_result.data.len());
    }

    // 6. 显示峰信息
    for peak in &result.peaks {
        println!("峰 {}: 中心={:.3}, 振幅={:.1}, 面积={:.1}, R²={:.3}", 
                 peak.id, peak.center, peak.amplitude, peak.area, peak.rsquared);
    }

    Ok(())
}
```

## 性能优化建议

1. **大数据文件**: 使用流式处理，避免一次性加载所有数据
2. **并行处理**: 对于多个文件，可以使用并行处理
3. **内存管理**: 及时释放不需要的数据结构
4. **缓存策略**: 对于重复分析，可以缓存中间结果

## 常见问题

### Q: 如何处理大文件？
A: 使用数据过滤功能，只加载需要的m/z和保留时间范围。

### Q: 如何提高峰检测精度？
A: 调整敏感度参数，使用CWT检测方法，或尝试不同的拟合方法。

### Q: 如何处理重叠峰？
A: 使用重叠峰处理功能，如FBF预处理或EMG NLLS拟合。

### Q: 如何自定义导出格式？
A: 实现Exporter trait，创建自定义导出器。
