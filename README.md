# MZ Curve GUI

质谱数据处理与分析工具 - 基于Tauri + Vue的桌面应用程序

## 项目概述

MZ Curve GUI是一个简单的数据处理，支持离子迁移谱（IMS）数据、质谱数据和色谱数据的处理与分析。

其基于rust mzData 库开发, 采用OpenMSUtils的逻辑进行数据处理

前端基于vue以及plotly进行数据处理

暂时不支持批量的数据处理, 如果需要, 请移步当前还在streamlit进行技术验证阶段的项目: 

### 特性:

- 能够封装为macOS, Linux等版本, 实现多平台的兼容(tauri的自带特性)
- 主要基于rust进行计算密集型数据处理,相比于python有较大的提升
- 小巧轻便
- 在电脑安装 .Net 8.0 运行时的情况下, 可以读取Thermo Raw数据, 可以方便没有安装Thermo付费软件的用户进行运行 (MZData库的特性!)

## 核心功能

### 🔬 数据处理
- **多格式支持**: mzML, MGF, Thermo Raw数据格式
- **分步处理**: 文件加载 → 曲线提取 → 峰分析 → 结果导出

### 🎨 可视化
- **交互式图表**: 基于Plotly的实时数据可视化

### 峰分析
当前并不完善, 之后有空才会接着开发

并且将在之后移植scipy的专业峰分析算法



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

