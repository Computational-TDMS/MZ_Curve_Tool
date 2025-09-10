# MZ Curve - 质谱数据处理与分析项目

## 项目概述

MZ Curve 是一个用 Rust 编写的质谱数据处理与分析库，专门用于处理离子迁移谱（IMS）数据、质谱数据和色谱数据。项目提供了完整的峰检测、峰拟合、基线校正和可视化功能。

## 核心功能

### 1. 数据加载与处理
- **支持格式**: mzML, mzXML, mzData 等质谱数据格式
- **数据提取**: DT曲线、TIC曲线、XIC曲线等
- **数据过滤**: 按m/z范围、保留时间、MS级别过滤

### 2. 峰分析功能
- **峰检测算法**:
  - CWT (连续小波变换)
  - 简单阈值检测
  - PeakFinder算法
- **峰拟合方法**:
  - 高斯拟合
  - 洛伦兹拟合
  - Pseudo-Voigt拟合
  - EMG (指数修正高斯)
  - Bi-Gaussian拟合
  - Voigt+指数尾拟合
  - Pearson-IV拟合
  - NLC (非线性色谱)
  - GMG贝叶斯拟合

### 3. 重叠峰处理
- **FBF预处理**
- **CWT锐化预处理**
- **EMG NLLS拟合**
- **极度重叠处理器**

### 4. 基线校正
- **非对称最小二乘法**
- **线性基线**
- **移动平均基线**
- **多项式基线**

### 5. 数据导出与可视化
- **TSV格式导出**: 完整的峰和曲线数据
- **Plotly JSON导出**: 交互式可视化
- **批量导出**: 支持多种格式同时导出

## 项目架构

### 流水线峰分析架构

```
前端 (Vue.js + TypeScript)
    ↓ (Tauri Commands)
后端 (Rust) - 流水线处理
    ↓ (DataContainer + Peak对象)
峰分析流水线
    ↓ (逐步增强的Peak对象)
前端可视化 (Plotly)
```

#### 核心流水线设计

```
1. 文件加载阶段:
   前端: FileSelector → 选择mzML等支持的格式的文件
   后端: load_file() → 返回FileInfo
   
2. 数据提取阶段:
   前端: ProcessingPanel → 设置提取参数
   后端: extract_curve() → 返回DataContainer
   └── DataContainer: { curves: [Curve], peaks: [], metadata: {} }
   
3. 峰检测流水线:
   前端: ProcessingPanel → 设置检测参数
   后端: detect_peaks(DataContainer) → 返回DataContainer
   └── DataContainer: { curves: [Curve], peaks: [Peak(center, amplitude)], metadata: {} }
   └── 可视化: 显示原始曲线 + 检测到的峰位置
   
4. 峰拟合流水线:
   前端: ProcessingPanel → 设置拟合参数
   后端: fit_peaks(DataContainer) → 返回DataContainer
   └── DataContainer: { curves: [Curve], peaks: [Peak(area, fwhm, 拟合参数)], metadata: {} }
   └── 可视化: 显示原始曲线 + 峰信息 + 拟合质量
   
5. 峰增强流水线:
   前端: ProcessingPanel → 设置增强参数
   后端: enhance_peaks(DataContainer) → 返回DataContainer
   └── DataContainer: { curves: [Curve], peaks: [Peak(质量评分, 边界, 分离度)], metadata: {} }
   └── 可视化: 显示完整的峰分析结果
   
6. 曲线还原流水线:
   前端: ProcessingPanel → 设置还原参数
   后端: reconstruct_curves(DataContainer) → 返回DataContainer
   └── DataContainer: { curves: [Curve, FittedCurve], peaks: [Peak], metadata: {} }
   └── 可视化: 显示原始曲线 + 拟合曲线 + 单个峰曲线
```

#### 数据传递优化

**统一数据容器 (DataContainer)**:
```rust
// 后端核心数据结构
pub struct DataContainer {
    pub metadata: HashMap<String, Value>,  // 元数据
    pub curves: Vec<Curve>,               // 曲线数据
    pub peaks: Vec<Peak>,                 // 峰数据
    pub spectra: Vec<Spectrum>,           // 原始光谱数据
}
```

**前端类型定义**:
```typescript
// 前端对应的数据结构
interface DataContainer {
  metadata: Record<string, any>;
  curves: CurveData[];
  peaks: PeakInfo[];
  // spectra 不直接传递到前端
}
```

### 流水线数据流架构

```
原始数据文件 (mzML/mzXML) 
    ↓
数据加载器 (DataLoader)
    ↓
数据容器 (DataContainer) ← 核心数据载体
    ↓
峰分析流水线 (PeakAnalysisPipeline)
    ├── 峰检测器 (PeakDetector) ← 添加位置+峰高
    ├── 峰拟合器 (PeakFitter) ← 添加面积+半峰宽+拟合参数
    ├── 峰增强器 (PeakEnhancer) ← 添加质量评分+边界信息
    └── 曲线还原器 (CurveReconstructor) ← 生成拟合曲线
    ↓
增强的DataContainer (包含完整Peak对象)
    ↓
导出器 (Exporter)
    ├── TSV导出器
    └── Plotly导出器
```

#### 流水线工具设计

**每个工具都操作DataContainer，逐步增强其中的Peak对象**:

```rust
// 峰检测器 - 流水线第一步
pub trait PeakDetector {
    fn detect_peaks(&self, container: &mut DataContainer, config: &Value) -> Result<(), ProcessingError>;
    // 功能: 从container.curves中检测峰，添加到container.peaks
    // 输出: Peak对象包含 center, amplitude, detection_method
}

// 峰拟合器 - 流水线第二步  
pub trait PeakFitter {
    fn fit_peaks(&self, container: &mut DataContainer, config: &Value) -> Result<(), ProcessingError>;
    // 功能: 对container.peaks中的峰进行拟合，增强峰信息
    // 输出: Peak对象添加 area, fwhm, sigma, rsquared, 拟合参数
}

// 峰增强器 - 流水线第三步
pub trait PeakEnhancer {
    fn enhance_peaks(&self, container: &mut DataContainer, config: &Value) -> Result<(), ProcessingError>;
    // 功能: 增强container.peaks中的峰信息
    // 输出: Peak对象添加 quality_score, boundaries, separation, 质量指标
}

// 曲线还原器 - 流水线第四步
pub trait CurveReconstructor {
    fn reconstruct_curves(&self, container: &mut DataContainer, config: &Value) -> Result<(), ProcessingError>;
    // 功能: 根据container.peaks的拟合参数生成拟合曲线，添加到container.curves
    // 输出: 添加FittedCurve到container.curves
}
```

### 核心数据结构

#### DataContainer
- 统一的数据容器，装载一条曲线及其对应的众多`Peak`对象
- 支持元数据管理
- 提供数据查询和过滤功能

**DataContainer结构**:
```rust
pub struct DataContainer {
    pub metadata: HashMap<String, Value>,  // 元数据
    pub curves: Vec<Curve>,               // 曲线数据 (原始曲线 + 拟合曲线)
    pub peaks: Vec<Peak>,                 // 峰数据 (逐步增强的Peak对象)
    pub spectra: Vec<Spectrum>,           // 原始光谱数据 (不传递到前端)
}
```

**流水线处理过程**:
```rust
// 初始状态
DataContainer {
    curves: [原始曲线],
    peaks: [],
    metadata: {}
}

// 峰检测后
DataContainer {
    curves: [原始曲线],
    peaks: [Peak1(center, amplitude), Peak2(center, amplitude), ...],
    metadata: { detection_method: "cwt", ... }
}

// 峰拟合后
DataContainer {
    curves: [原始曲线],
    peaks: [Peak1(area, fwhm, 拟合参数), Peak2(area, fwhm, 拟合参数), ...],
    metadata: { fitting_method: "gaussian", ... }
}

// 曲线还原后
DataContainer {
    curves: [原始曲线, 拟合曲线],
    peaks: [Peak1(完整信息), Peak2(完整信息), ...],
    metadata: { reconstructed: true, ... }
} 每个峰都有各自的曲线信息, 从而也能够展示其各自的形状!
```

#### Curve
- 完整的曲线数据结构
- 包含科学参数：信噪比、基线、质量评分等
- 支持多种曲线类型：DT、TIC、XIC、EIC等

#### Peak
- 高精度峰数据结构
- 包含拟合参数、质量指标、置信度等
- 支持多种峰类型和检测算法

## 模块组织

### 核心模块
- `data/`: 数据结构定义
- `loaders/`: 数据加载器
- `processors/`: 数据处理模块
- `exporters/`: 数据导出模块
- `utils/`: 工具函数

### 处理器模块
- `peak_detection/`: 峰检测算法
- `peak_fitting/`: 峰拟合方法
- `overlapping_peaks/`: 重叠峰处理
- `baseline_correction/`: 基线校正

## 流水线API接口设计

### Tauri Commands 接口 - 流水线架构

```rust
// 1. 文件操作
#[tauri::command]
pub async fn load_file(file_path: String) -> Result<FileInfo, String>

#[tauri::command] 
pub async fn validate_file(file_path: String) -> Result<ValidationResult, String>

// 2. 数据提取
#[tauri::command]
pub async fn extract_curve(params: CurveExtractionParams) -> Result<DataContainer, String>

// 3. 峰检测流水线 - 添加位置和峰高
#[tauri::command]
pub async fn detect_peaks(
    container: DataContainer,           // 输入: 包含曲线的DataContainer
    params: PeakDetectionParams
) -> Result<DataContainer, String>      // 输出: 包含检测峰的DataContainer
// Peak对象增强: center, amplitude, detection_method

// 4. 峰拟合流水线 - 添加面积、半峰宽、拟合参数
#[tauri::command]
pub async fn fit_peaks(
    container: DataContainer,           // 输入: 包含检测峰的DataContainer
    params: PeakFittingParams
) -> Result<DataContainer, String>      // 输出: 包含拟合结果的DataContainer
// Peak对象增强: area, fwhm, sigma, rsquared, 拟合参数
// 功能: 支持曲线还原用于可视化

// 5. 峰增强流水线 - 添加质量评分、边界信息
#[tauri::command]
pub async fn enhance_peaks(
    container: DataContainer,           // 输入: 包含拟合峰的DataContainer
    params: PeakEnhancementParams
) -> Result<DataContainer, String>      // 输出: 包含增强峰的DataContainer
// Peak对象增强: quality_score, boundaries, separation, 质量指标

// 6. 曲线还原 - 生成拟合曲线用于可视化
#[tauri::command]
pub async fn reconstruct_curves(
    container: DataContainer,           // 输入: 包含拟合峰的DataContainer
    params: CurveReconstructionParams
) -> Result<DataContainer, String>      // 输出: 包含拟合曲线的DataContainer
// 功能: 根据拟合参数生成拟合曲线数据点

// 7. 基线校正
#[tauri::command]
pub async fn baseline_correction(
    container: DataContainer,           // 输入: DataContainer
    params: BaselineCorrectionParams
) -> Result<DataContainer, String>      // 输出: 处理后的DataContainer

// 8. 数据导出
#[tauri::command]
pub async fn export_tsv(container: DataContainer, params: ExportParams) -> Result<ExportResultInfo, String>

#[tauri::command]
pub async fn export_plotly(container: DataContainer, params: ExportParams) -> Result<ExportResultInfo, String>
```

### 前端API调用 - 流水线处理

```typescript
// 1. 文件加载
const fileInfo = await MZCurveAPI.loadFile(filePath);

// 2. 数据提取
const container = await MZCurveAPI.extractCurve({
  file_path: filePath,
  mz_range: "100.0-200.0",
  rt_range: "0.0-60.0",
  ms_level: 1,
  mode: "dt"
});

// 3. 峰检测流水线 - 在DataContainer中添加峰对象
const containerWithPeaks = await MZCurveAPI.detectPeaks(container, {
  detection_method: "cwt",
  sensitivity: 0.7,
  threshold_multiplier: 3.0,
  min_peak_width: 0.1,
  max_peak_width: 10.0
});
// 结果: DataContainer { curves: [原始曲线], peaks: [Peak(center, amplitude)], metadata: {} }
// 可视化: 显示原始曲线 + 检测到的峰位置

// 4. 峰拟合流水线 - 增强DataContainer中的峰对象
const containerWithFittedPeaks = await MZCurveAPI.fitPeaks(containerWithPeaks, {
  fitting_method: "gaussian",
  overlapping_method: "auto",
  fit_quality_threshold: 0.8
});
// 结果: DataContainer { curves: [原始曲线], peaks: [Peak(area, fwhm, 拟合参数)], metadata: {} }
// 可视化: 显示原始曲线 + 峰信息 + 拟合质量

// 5. 峰增强流水线 - 进一步增强DataContainer中的峰对象
const containerWithEnhancedPeaks = await MZCurveAPI.enhancePeaks(containerWithFittedPeaks, {
  quality_threshold: 0.5,
  boundary_method: "adaptive",
  separation_analysis: true
});
// 结果: DataContainer { curves: [原始曲线], peaks: [Peak(质量评分, 边界, 分离度)], metadata: {} }
// 可视化: 显示完整的峰分析结果

// 6. 曲线还原 - 在DataContainer中添加拟合曲线
const containerWithReconstructedCurves = await MZCurveAPI.reconstructCurves(containerWithEnhancedPeaks, {
  resolution: 1000,
  include_baseline: true,
  include_individual_peaks: true
});
// 结果: DataContainer { curves: [原始曲线, 拟合曲线], peaks: [Peak(完整信息)], metadata: {} }
// 可视化: 显示原始曲线 + 拟合曲线 + 单个峰曲线

// 7. 基线校正 (可选步骤)
const correctedContainer = await MZCurveAPI.baselineCorrection(containerWithReconstructedCurves, {
  method: "asymmetric_least_squares",
  parameters: { lambda: 1000, p: 0.01 }
});

// 8. 数据导出
const exportResult = await MZCurveAPI.exportTsv(correctedContainer, {
  output_path: "./results.tsv",
  include_curves: true,
  include_peaks: true,
  include_fitted_curves: true
});
```

## 使用示例

### 基本使用流程

```rust
use mz_curve::*;

// 1. 创建处理请求
let request = ProcessingRequest {
    file_path: "data.mzML".to_string(),
    mz_range: "100.0-200.0".to_string(),
    rt_range: "0.0-60.0".to_string(),
    ms_level: 1,
    mode: "dt".to_string(),
};

// 2. 处理数据
let result = process_file(request).await?;

// 3. 导出结果
let export_manager = ExportManager::new();
let export_config = serde_json::json!({
    "include_curves": true,
    "include_peaks": true
});

let export_result = export_manager.export(
    "plotly",
    &result,
    export_config
).await?;
```

### 峰分析示例

```rust
use mz_curve::processors::PeakAnalyzer;

// 创建峰分析器
let analyzer = PeakAnalyzer::new_with_overlapping_processing(
    "cwt",           // 检测方法
    "gaussian",      // 拟合方法
    Some("fbf")      // 重叠峰处理
)?;

// 配置参数
let config = serde_json::json!({
    "sensitivity": 0.7,
    "threshold_multiplier": 3.0,
    "min_peak_width": 0.1
});

// 执行分析
let result = analyzer.process(data_container, config).await?;
```

## 流水线架构优势

### 核心设计理念

**1. 流水线工具化**:
- 每个处理方法都是流水线上的一道工具
- 操作`DataContainer`对象，逐步增强其中的`Peak`对象
- 支持独立测试、调试和优化

**2. 逐步增强DataContainer**:
```
DataContainer { curves: [Curve], peaks: [], metadata: {} }
    ↓ 峰检测器
DataContainer { curves: [Curve], peaks: [Peak(center, amplitude)], metadata: {} }
    ↓ 峰拟合器  
DataContainer { curves: [Curve], peaks: [Peak(area, fwhm, 拟合参数)], metadata: {} }
    ↓ 峰增强器
DataContainer { curves: [Curve], peaks: [Peak(质量评分, 边界, 分离度)], metadata: {} }
    ↓ 曲线还原器
DataContainer { curves: [Curve, FittedCurve], peaks: [Peak], metadata: {} }
```

**3. 可视化追踪**:
- 每一步都可以独立可视化
- 支持参数调整和实时预览
- 完整的处理历史记录

### 流水线优势

**1. 模块化设计**:
```rust
// 每个工具都操作DataContainer，可以单独测试和优化
pub trait PeakDetector {
    fn detect_peaks(&self, container: &mut DataContainer, config: &Value) -> Result<(), ProcessingError>;
    // 从container.curves检测峰，添加到container.peaks
}

pub trait PeakFitter {
    fn fit_peaks(&self, container: &mut DataContainer, config: &Value) -> Result<(), ProcessingError>;
    // 对container.peaks中的峰进行拟合，增强峰信息
}

pub trait CurveReconstructor {
    fn reconstruct_curves(&self, container: &mut DataContainer, config: &Value) -> Result<(), ProcessingError>;
    // 根据container.peaks生成拟合曲线，添加到container.curves
}
```

**2. 参数组合实验**:
- 用户可以尝试不同的检测方法 + 拟合方法组合
- 支持A/B测试和参数优化
- 便于找到最佳参数组合

**3. 实时可视化反馈**:
- 峰检测后：显示检测到的峰位置
- 峰拟合后：显示拟合结果和质量指标
- 峰增强后：显示完整的分析结果
- 曲线还原后：显示拟合曲线对比

**4. 数据完整性**:
- `DataContainer` 作为统一数据载体，装载一条曲线及其对应的众多`Peak`对象
- `Peak` 对象在`DataContainer`中逐步增强，保持所有处理历史
- 支持处理链的追溯和调试
- 每一步都返回完整的`DataContainer`，便于前端可视化

### 性能优势

**1. 减少数据转换**:
```rust
// 优化前: 多次转换
CurveData → DataContainer → Curve → PeakAnalysisResult

// 优化后: 直接传递
DataContainer → PeakAnalysisResult
```

**2. 保持数据完整性**:
- 元数据在整个处理流程中保持一致
- 处理历史记录在 `DataContainer.metadata` 中
- 支持处理链的追溯和调试

**3. 简化API设计**:
```typescript
// 所有处理函数都使用相同的签名模式
async function processData(
  container: DataContainer, 
  params: ProcessingParams
): Promise<DataContainer | ProcessingResult>
```

### 流水线用户交互流程示例

```typescript
// 完整的流水线用户交互流程
class PeakAnalysisPipeline {
  private currentContainer: DataContainer;
  
  // 1. 加载文件
  async loadFile(filePath: string) {
    const fileInfo = await MZCurveAPI.loadFile(filePath);
    return fileInfo;
  }
  
  // 2. 提取曲线
  async extractCurve(params: CurveExtractionParams) {
    this.currentContainer = await MZCurveAPI.extractCurve(params);
    this.visualizeCurrentState(); // 可视化原始曲线
    return this.currentContainer;
  }
  
  // 3. 峰检测流水线 (可重复调用调整参数)
  async detectPeaks(params: PeakDetectionParams) {
    this.currentContainer = await MZCurveAPI.detectPeaks(this.currentContainer, params);
    // this.currentContainer.peaks 现在包含检测到的峰
    this.visualizePeakDetection(); // 可视化检测结果
    return this.currentContainer;
  }
  
  // 4. 峰拟合流水线 (可重复调用调整参数)
  async fitPeaks(params: PeakFittingParams) {
    this.currentContainer = await MZCurveAPI.fitPeaks(this.currentContainer, params);
    // this.currentContainer.peaks 中的峰现在包含拟合信息
    this.visualizePeakFitting(); // 可视化拟合结果
    return this.currentContainer;
  }
  
  // 5. 峰增强流水线 (可重复调用调整参数)
  async enhancePeaks(params: PeakEnhancementParams) {
    this.currentContainer = await MZCurveAPI.enhancePeaks(this.currentContainer, params);
    // this.currentContainer.peaks 中的峰现在包含增强信息
    this.visualizePeakEnhancement(); // 可视化增强结果
    return this.currentContainer;
  }
  
  // 6. 曲线还原 (在DataContainer中添加拟合曲线)
  async reconstructCurves(params: CurveReconstructionParams) {
    this.currentContainer = await MZCurveAPI.reconstructCurves(this.currentContainer, params);
    // this.currentContainer.curves 现在包含原始曲线和拟合曲线
    this.visualizeReconstructedCurves(); // 可视化拟合曲线
    return this.currentContainer;
  }
  
  // 7. 基线校正 (可选步骤)
  async correctBaseline(params: BaselineCorrectionParams) {
    this.currentContainer = await MZCurveAPI.baselineCorrection(this.currentContainer, params);
    this.visualizeCurrentState(); // 可视化校正结果
    return this.currentContainer;
  }
  
  // 8. 导出结果
  async exportResults(params: ExportParams) {
    return await MZCurveAPI.exportTsv(this.currentContainer, params);
  }
  
  // 可视化方法 - 分步显示流水线结果
  private visualizeCurrentState() {
    this.updatePlotlyChart(this.currentContainer);
  }
  
  private visualizePeakDetection() {
    // 显示DataContainer中的原始曲线 + 检测到的峰位置
    // this.currentContainer.curves[0] - 原始曲线
    // this.currentContainer.peaks - 检测到的峰 (center, amplitude)
    this.updatePlotlyChart({
      ...this.currentContainer,
      visualization_mode: "peak_detection"
    });
  }
  
  private visualizePeakFitting() {
    // 显示DataContainer中的原始曲线 + 峰信息 (面积、半峰宽等)
    // this.currentContainer.curves[0] - 原始曲线
    // this.currentContainer.peaks - 拟合后的峰 (area, fwhm, 拟合参数)
    this.updatePlotlyChart({
      ...this.currentContainer,
      visualization_mode: "peak_fitting"
    });
  }
  
  private visualizePeakEnhancement() {
    // 显示DataContainer中的完整峰分析结果 (质量评分、边界等)
    // this.currentContainer.curves[0] - 原始曲线
    // this.currentContainer.peaks - 增强后的峰 (质量评分, 边界, 分离度)
    this.updatePlotlyChart({
      ...this.currentContainer,
      visualization_mode: "peak_enhancement"
    });
  }
  
  private visualizeReconstructedCurves() {
    // 显示DataContainer中的原始曲线 + 拟合曲线 + 单个峰曲线
    // this.currentContainer.curves[0] - 原始曲线
    // this.currentContainer.curves[1] - 拟合曲线
    // this.currentContainer.peaks - 完整的峰信息
    this.updatePlotlyChart({
      ...this.currentContainer,
      visualization_mode: "reconstructed_curves"
    });
  }
}
```

## 可视化功能

### Plotly集成
- 交互式图表生成
- 支持多种图表类型：线图、散点图、柱状图
- 峰标注和拟合曲线显示
- 可自定义样式和布局

### 导出格式
- **TSV**: 完整的数值数据，适合进一步分析
- **Plotly JSON**: 交互式可视化，适合报告和展示

## 性能特性

- **异步处理**: 支持大规模数据处理
- **内存优化**: 流式处理，减少内存占用
- **并行计算**: 支持多线程处理
- **类型安全**: Rust类型系统保证数据安全

## 扩展性

- **插件架构**: 支持自定义处理器和导出器
- **配置驱动**: 通过JSON配置控制处理流程
- **Python绑定**: 支持Python接口调用

## 测试覆盖

项目包含完整的测试套件：
- 单元测试：各模块功能测试
- 集成测试：端到端流程测试
- 性能测试：大数据量处理测试
- 基准测试：算法性能对比

## 依赖项

- `mzdata`: 质谱数据格式支持
- `serde`: 序列化/反序列化
- `async-trait`: 异步trait支持
- `uuid`: 唯一标识符生成
- `plotly`: 可视化支持

## 许可证

MIT License

## 贡献指南

1. Fork 项目
2. 创建功能分支
3. 提交更改
4. 创建 Pull Request

## 联系方式

项目维护者：[您的姓名]
邮箱：[您的邮箱]
