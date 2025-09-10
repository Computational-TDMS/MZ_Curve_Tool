# MZ Curve GUI

质谱数据处理与分析工具 - 基于Tauri + Vue的桌面应用程序

## 项目概述

MZ Curve GUI是一个专业的质谱数据处理工具，支持离子迁移谱（IMS）数据、质谱数据和色谱数据的处理与分析。该工具提供了完整的峰检测、峰拟合、基线校正和可视化功能。

## 核心功能

### 🔬 数据处理
- **多格式支持**: mzML, mzXML, mzData等质谱数据格式
- **分步处理**: 文件加载 → 曲线提取 → 峰分析 → 结果导出
- **批量处理**: 支持多文件同时处理
- **实时状态**: 处理进度和状态实时显示

### 📊 峰分析算法
- **峰检测**: CWT、简单阈值、峰查找器
- **峰拟合**: 高斯、洛伦兹、EMG、Pseudo-Voigt等9种方法
- **重叠峰处理**: FBF、CWT锐化、EMG NLLS等
- **基线校正**: 线性、多项式、移动平均、非对称最小二乘法

### 🎨 可视化
- **交互式图表**: 基于Plotly的实时数据可视化
- **峰标注**: 自动峰识别和标注
- **拟合显示**: 峰拟合曲线和阴影显示
- **多曲线对比**: 支持多条曲线同时显示

## 技术架构

### 后端 (Rust)
```
src-tauri/src/
├── lib.rs              # Tauri命令和API接口
├── state.rs            # 应用状态管理
└── core/                # 数据处理核心库
    ├── data/           # 数据结构定义
    ├── loaders/        # 数据加载器
    ├── processors/     # 数据处理模块
    ├── exporters/      # 数据导出模块
    └── utils/          # 工具函数
```

### 前端 (Vue 3 + TypeScript)
```
src/
├── components/         # Vue组件
│   ├── FileSelector.vue      # 文件选择
│   ├── ProcessingPanel.vue   # 处理参数配置
│   ├── DataVisualization.vue # 数据可视化
│   └── StatusPanel.vue       # 状态显示
├── stores/             # 状态管理
│   └── appStore.ts     # 应用状态
├── types/              # TypeScript类型定义
│   └── api.ts          # API接口类型
└── utils/              # 工具函数
    └── api.ts          # API调用封装
```

