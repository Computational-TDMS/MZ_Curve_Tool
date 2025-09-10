<template>
  <div id="app">
    <el-container class="main-container">
      <!-- 左侧参数配置面板 -->
      <el-aside width="300px" class="left-panel">
        <ParameterPanel 
          :data-ranges="dataRanges"
          @load-file="handleLoadFile"
          @extract-curve="handleExtractCurve"
          @detect-peaks="handleDetectPeaks"
          @fit-peaks="handleFitPeaks"
          @run-pipeline="handleRunPipeline"
          @export-results="handleExportResults"
          @export-spectro-data="handleExportSpectroData"
        />
      </el-aside>

      <!-- 中间图像面板 -->
      <el-main class="center-panel">
        <PlotPanel 
          :container="currentContainer"
          :plot-mode="plotMode"
          @plot-mode-changed="handlePlotModeChanged"
        />
      </el-main>

      <!-- 右侧信息输出面板 -->
      <el-aside width="300px" class="right-panel">
        <InfoPanel 
          :file-info="currentFileInfo"
          :status="processingStatus"
          :logs="logs"
          :curve-data="curveData"
          @export-curves="handleExportCurvesToFolder"
        />
      </el-aside>
    </el-container>

    <!-- 底部进度条 -->
    <el-footer height="60px" class="progress-footer">
      <ProgressBar 
        :current="progressCurrent"
        :total="progressTotal"
        :message="progressMessage"
      />
    </el-footer>

    <!-- 加载遮罩 -->
    <el-loading 
      v-loading="isProcessing"
      element-loading-text="处理中..."
      element-loading-spinner="el-icon-loading"
      element-loading-background="rgba(0, 0, 0, 0.8)"
    />
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import ParameterPanel from './components/ParameterPanel.vue'
import PlotPanel from './components/PlotPanel.vue'
import InfoPanel from './components/InfoPanel.vue'
import ProgressBar from './components/ProgressBar.vue'
import type { SerializableDataContainer, FileInfo, LogMessage, ProgressUpdate, ProcessingStatus, CurveDisplayData, DataRanges } from './types/data'

// 响应式数据
const currentContainer = ref<SerializableDataContainer | null>(null)
const currentFileInfo = ref<FileInfo | null>(null)
const processingStatus = ref<ProcessingStatus>('idle')
const isProcessing = ref(false)
const plotMode = ref('original')
const logs = ref<LogMessage[]>([])
const progressCurrent = ref(0)
const progressTotal = ref(0)
const progressMessage = ref('就绪')
const dataRanges = ref<DataRanges | undefined>(undefined)

const curveData = ref<CurveDisplayData[]>([])

// 事件监听器
let statusListener: (() => void) | null = null
let logListener: (() => void) | null = null
let progressListener: (() => void) | null = null

// 生命周期
onMounted(async () => {
  await initializeApp()
  await setupEventListeners()
})

onUnmounted(() => {
  // 清理事件监听器
  if (statusListener) statusListener()
  if (logListener) logListener()
  if (progressListener) progressListener()
})

// 初始化应用
async function initializeApp() {
  addLog('info', '系统启动', 'MZ Curve GUI 已就绪')
}

// 设置事件监听器
async function setupEventListeners() {
  try {
    const { listen } = await import('@tauri-apps/api/event')
    
    // 监听状态更新事件
    statusListener = await listen<ProcessingStatus>('status-updated', (event) => {
      console.log('状态更新:', event.payload)
      processingStatus.value = event.payload
      
      // 根据状态更新处理状态
      if (event.payload === 'loading' || event.payload === 'extracting' || 
          event.payload === 'analyzing' || event.payload === 'exporting') {
        isProcessing.value = true
      } else {
        isProcessing.value = false
      }
      
      // 显示状态消息
      if (event.payload === 'success') {
        ElMessage.success('操作完成')
      } else if (event.payload === 'error') {
        ElMessage.error('操作失败')
      }
    })
    
    // 监听日志消息事件
    logListener = await listen<LogMessage>('log-message', (event) => {
      console.log('日志消息:', event.payload)
      addLog(event.payload.level, event.payload.title, event.payload.content)
    })
    
    // 监听进度更新事件
    progressListener = await listen<ProgressUpdate>('progress-updated', (event) => {
      console.log('进度更新:', event.payload)
      progressCurrent.value = event.payload.current
      progressTotal.value = event.payload.total
      progressMessage.value = event.payload.message
    })
    
    console.log('事件监听器设置完成')
  } catch (error) {
    console.error('设置事件监听器失败:', error)
  }
}

// 事件处理函数
async function handleLoadFile(filePath: string) {
  try {
    setProcessing(true, '加载文件')
    // 移除手动添加的日志，让后端事件处理
    
    const { invoke } = await import('@tauri-apps/api/core')
    const fileInfo = await invoke('load_file', { filePath })
    currentFileInfo.value = fileInfo
    
    // 获取数据范围并更新
    if (fileInfo && fileInfo.data_ranges) {
      console.log('接收到数据范围:', fileInfo.data_ranges)
      dataRanges.value = fileInfo.data_ranges
      console.log('设置数据范围:', dataRanges.value)
    }
    
    // 移除手动添加的成功日志，让后端事件处理
    ElMessage.success('文件加载成功')
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    // 保留错误日志，因为后端可能没有发送错误事件
    addLog('error', '文件加载失败', errorMessage)
    ElMessage.error('文件加载失败: ' + errorMessage)
  } finally {
    setProcessing(false)
  }
}

async function handleExtractCurve(params: any) {
  try {
    setProcessing(true, '提取曲线')
    console.log('开始提取曲线，参数:', params)
    
    const { invoke } = await import('@tauri-apps/api/core')
    const result = await invoke('extract_curve', { params })
    console.log('曲线提取结果:', result)
    
    currentContainer.value = result
    plotMode.value = 'original'
    
    // 获取曲线数据用于显示
    await loadCurveData()
    
    ElMessage.success(`曲线提取完成 (${params.curve_type})`)
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    // 保留错误日志，因为后端可能没有发送错误事件
    addLog('error', '曲线提取失败', errorMessage)
    ElMessage.error('曲线提取失败: ' + errorMessage)
  } finally {
    setProcessing(false)
  }
}

async function loadCurveData() {
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const data = await invoke('get_curve_data_for_display')
    curveData.value = data as CurveDisplayData[]
    console.log('加载的曲线数据:', curveData.value)
  } catch (error) {
    console.error('获取曲线数据失败:', error)
    curveData.value = []
  }
}

async function handleDetectPeaks(params: any) {
  if (!currentContainer.value) {
    ElMessage.warning('请先提取曲线')
    return
  }

  try {
    setProcessing(true, '峰检测')
    // 移除手动添加的日志，让后端事件处理
    
    const { invoke } = await import('@tauri-apps/api/core')
    currentContainer.value = await invoke('detect_peaks', { 
      container: currentContainer.value, 
      params 
    })
    plotMode.value = 'peaks'
    
    // 移除手动添加的成功日志，让后端事件处理
    ElMessage.success('峰检测完成')
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    // 保留错误日志，因为后端可能没有发送错误事件
    addLog('error', '峰检测失败', errorMessage)
    ElMessage.error('峰检测失败: ' + errorMessage)
  } finally {
    setProcessing(false)
  }
}

async function handleFitPeaks(params: any) {
  if (!currentContainer.value || currentContainer.value.peaks.length === 0) {
    ElMessage.warning('请先进行峰检测')
    return
  }

  try {
    setProcessing(true, '峰拟合')
    // 移除手动添加的日志，让后端事件处理
    
    const { invoke } = await import('@tauri-apps/api/core')
    currentContainer.value = await invoke('fit_peaks', { 
      container: currentContainer.value, 
      params 
    })
    plotMode.value = 'fitted'
    
    // 移除手动添加的成功日志，让后端事件处理
    ElMessage.success('峰拟合完成')
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    // 保留错误日志，因为后端可能没有发送错误事件
    addLog('error', '峰拟合失败', errorMessage)
    ElMessage.error('峰拟合失败: ' + errorMessage)
  } finally {
    setProcessing(false)
  }
}

async function handleRunPipeline(params: any) {
  try {
    setProcessing(true, '自动处理')
    // 移除手动添加的日志，让后端事件处理
    
    // 1. 提取曲线
    await handleExtractCurve(params.extraction)
    if (!currentContainer.value) return

    // 2. 峰检测
    await handleDetectPeaks(params.detection)
    if (!currentContainer.value || currentContainer.value.peaks.length === 0) return

    // 3. 峰拟合
    await handleFitPeaks(params.fitting)

    // 移除手动添加的成功日志，让后端事件处理
    ElMessage.success('自动处理完成')
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    // 保留错误日志，因为后端可能没有发送错误事件
    addLog('error', '自动处理失败', errorMessage)
    ElMessage.error('自动处理失败: ' + errorMessage)
  } finally {
    setProcessing(false)
  }
}

function handlePlotModeChanged(mode: string) {
  plotMode.value = mode
}

async function handleExportResults() {
  if (!currentContainer.value) {
    ElMessage.warning('没有可导出的数据')
    return
  }

  try {
    setProcessing(true, '导出结果')
    
    // 使用新的曲线导出功能
    await handleExportCurvesToFolder()
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    addLog('error', '导出失败', errorMessage)
    ElMessage.error('导出失败: ' + errorMessage)
  } finally {
    setProcessing(false)
  }
}

async function handleExportCurvesToFolder() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const folderPath = await open({
      title: '选择导出文件夹',
      directory: true,
      multiple: false
    })
    
    if (!folderPath) return
    
    setProcessing(true, '导出曲线数据')
    
    const { invoke } = await import('@tauri-apps/api/core')
    const result = await invoke('export_curves_to_folder', { 
      outputFolder: folderPath,
      container: currentContainer.value
    })
    
    ElMessage.success(`曲线数据导出完成: ${result.message}`)
    addLog('success', '导出完成', `曲线数据已导出到: ${folderPath}`)
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    addLog('error', '导出失败', errorMessage)
    ElMessage.error('导出失败: ' + errorMessage)
  } finally {
    setProcessing(false)
  }
}

async function handleExportSpectroData(params: any) {
  try {
    console.log('开始导出光谱数据，参数:', params)
    
    const { save } = await import('@tauri-apps/plugin-dialog')
    const filePath = await save({
      title: '保存光谱数据文件',
      filters: [{
        name: 'TSV文件',
        extensions: ['tsv']
      }],
      defaultPath: 'spectro_data.tsv'
    })
    
    if (!filePath) {
      console.log('用户取消了文件保存')
      return
    }
    
    console.log('选择的文件路径:', filePath)
    
    setProcessing(true, '导出光谱数据')
    
    const exportParams = {
      ...params,
      output_path: filePath
    }
    
    console.log('发送到后端的参数:', exportParams)
    
    const { invoke } = await import('@tauri-apps/api/core')
    const result = await invoke('export_spectro_tsv', { params: exportParams })
    
    console.log('后端返回结果:', result)
    
    ElMessage.success(`光谱数据导出完成: ${result.message}`)
    addLog('success', '导出完成', `光谱数据已导出到: ${filePath}`)
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    console.error('导出失败:', error)
    addLog('error', '导出失败', errorMessage)
    ElMessage.error('导出失败: ' + errorMessage)
  } finally {
    setProcessing(false)
  }
}

// 工具函数
function setProcessing(processing: boolean, message = '处理中...') {
  isProcessing.value = processing
  processingStatus.value = processing ? 'loading' : 'idle'
  progressMessage.value = processing ? message : '就绪'
}

function addLog(level: string, title: string, content: string) {
  const log: LogMessage = {
    id: Date.now().toString(),
    level,
    title,
    content,
    timestamp: new Date().toLocaleTimeString()
  }
  logs.value.push(log)
  
  // 限制日志数量
  if (logs.value.length > 100) {
    logs.value.shift()
  }
}
</script>

<style scoped>
.main-container {
  height: calc(100vh - 60px);
}

.left-panel {
  background: #fff;
  border-right: 1px solid #e4e7ed;
  overflow-y: auto;
}

.center-panel {
  background: #fff;
  padding: 0;
}

.right-panel {
  background: #fff;
  border-left: 1px solid #e4e7ed;
  overflow-y: auto;
}

.progress-footer {
  background: #f5f7fa;
  border-top: 1px solid #e4e7ed;
  padding: 0 20px;
  display: flex;
  align-items: center;
}
</style>
