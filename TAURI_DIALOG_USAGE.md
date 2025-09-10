# Tauri Dialog 功能使用说明

## 概述

本项目已成功集成了 Tauri 2.x Dialog 插件，提供了完整的文件对话框功能，包括文件选择、保存、消息提示等。用户可以通过 Windows 原生对话框来选择文件或保存结果。

## 版本信息

- **Tauri 版本**: 2.x
- **Dialog 插件版本**: 2.x
- **配置方式**: 使用 capabilities 权限系统

## 功能特性

### 1. 文件选择对话框
- **位置**: 参数配置面板的"选择"按钮
- **支持格式**: 
  - 质谱数据文件: `.mzml`, `.mzxml`, `.mzdata`, `.raw`, `.d`
  - 所有文件类型
- **功能**: 选择要处理的质谱数据文件

### 2. 文件保存对话框
- **位置**: 导出结果时自动弹出
- **支持格式**:
  - TSV 文件 (`.tsv`)
  - JSON 文件 (`.json`)
  - 所有文件类型
- **功能**: 选择导出结果的保存位置和文件名

### 3. 消息对话框
- **类型**: 信息提示、警告、错误消息
- **功能**: 显示处理状态和结果信息

## 使用方法

### 选择文件
1. 在左侧参数配置面板中，点击"数据文件"输入框右侧的"选择"按钮
2. 在弹出的 Windows 文件选择对话框中浏览并选择目标文件
3. 点击"打开"确认选择
4. 文件路径将自动填入输入框

### 导出结果
1. 完成数据处理后，点击"导出结果"按钮
2. 在弹出的 Windows 保存对话框中：
   - 选择保存位置
   - 输入文件名
   - 选择文件格式 (TSV 或 JSON)
3. 点击"保存"开始导出

## 技术实现

### 插件安装
```bash
# 安装 Tauri 2.x dialog 插件
pnpm tauri add dialog
pnpm add @tauri-apps/plugin-dialog
```

### 权限配置
```json
// src-tauri/capabilities/default.json
{
  "permissions": [
    "core:default",
    "opener:default",
    "dialog:default"
  ]
}
```

### 插件初始化
```rust
// src-tauri/src/lib.rs
::tauri::Builder::default()
    .plugin(tauri_plugin_dialog::init())
    // ... 其他配置
```

### 类型定义
```typescript
// src/types/tauri.d.ts
interface Window {
  __TAURI__: {
    dialog: {
      open: (options?) => Promise<string | string[] | null>
      save: (options?) => Promise<string | null>
      message: (message: string, options?) => Promise<void>
      ask: (message: string, options?) => Promise<boolean>
      confirm: (message: string, options?) => Promise<boolean>
    }
  }
}
```

### 前端调用示例
```typescript
// 文件选择
import { open } from '@tauri-apps/plugin-dialog'

const selected = await open({
  title: '选择质谱数据文件',
  filters: [{
    name: '质谱数据文件',
    extensions: ['mzml', 'mzxml', 'mzdata', 'raw', 'd']
  }],
  multiple: false
});

// 文件保存
import { save } from '@tauri-apps/plugin-dialog'

const savePath = await save({
  title: '保存导出结果',
  filters: [{
    name: 'TSV文件',
    extensions: ['tsv']
  }],
  defaultPath: 'export.tsv'
});
```

## 测试验证

### 测试页面
项目根目录下的 `test-dialog.html` 文件提供了完整的对话框功能测试页面，包括：
- 文件选择对话框测试
- 文件保存对话框测试
- 消息对话框测试
- 确认对话框测试
- 询问对话框测试

### 运行测试
1. 在 Tauri 应用中打开 `test-dialog.html`
2. 点击各个测试按钮验证功能
3. 检查控制台输出和结果显示

## 错误处理

### 常见问题
1. **对话框不显示**: 检查 Tauri 配置是否正确
2. **文件选择失败**: 确认文件路径和权限
3. **类型错误**: 检查 TypeScript 类型定义

### 调试方法
```typescript
try {
  const result = await window.__TAURI__.dialog.open(options);
  console.log('对话框结果:', result);
} catch (error) {
  console.error('对话框错误:', error);
}
```

## 注意事项

1. **环境要求**: 必须在 Tauri 桌面应用中运行，浏览器环境不支持
2. **权限**: 确保应用有文件系统访问权限
3. **路径格式**: Windows 路径使用反斜杠，需要正确处理
4. **异步操作**: 所有对话框操作都是异步的，需要使用 await

## 更新日志

- **v1.0.0**: 初始实现，支持基本的文件选择和保存功能
- **v1.1.0**: 添加了完整的类型定义和错误处理
- **v1.2.0**: 增加了测试页面和使用说明文档
- **v2.0.0**: 升级到 Tauri 2.x，使用新的插件系统和权限配置

## 技术支持

如果遇到问题，请检查：
1. Tauri 版本兼容性
2. 插件配置是否正确
3. 文件权限设置
4. 控制台错误信息

---

*最后更新: 2024年12月*
