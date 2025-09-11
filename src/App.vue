<template>
  <div id="app">
    <el-container class="main-container">
      <!-- 左侧面板 -->
      <el-aside width="350px" class="left-panel">
        <!-- 文件列表管理 -->
        <FileListPanel 
          ref="fileListRef"
          @file-selected="handleFileSelected"
          @file-processed="handleFileProcessed"
          @process-all="handleProcessAll"
          @compare-files="handleCompareFiles"
          @file-checked="handleFileChecked"
          @single-file-load="handleSingleFileLoad"
          @single-file-analyze="handleSingleFileAnalyze"
        />
        
        <!-- 参数配置面板 -->
        <ParameterPanel 
          :data-ranges="dataRanges"
          :selected-file="selectedFile"
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
          :multi-curve-data="multiCurveData"
          :is-comparing="isComparing"
          @plot-mode-changed="handlePlotModeChanged"
        />
      </el-main>

      <!-- 右侧信息输出面板 -->
      <el-aside width="300px" class="right-panel">
        <!-- 拟合质量评估面板 -->
        <FittingQualityPanel 
          v-if="currentContainer && currentContainer.peaks && currentContainer.peaks.length > 0"
          :peaks="currentContainer.peaks"
          @export-report="handleExportQualityReport"
          @optimize-parameters="handleOptimizeParameters"
        />
        
        <InfoPanel 
          :file-info="currentFileInfo"
          :status="processingStatus"
          :logs="logs"
          :curve-data="curveData"
          :multi-curve-data="multiCurveData"
          :is-comparing="isComparing"
          @export-curves="handleExportCurvesToFolder"
          @exit-comparison="handleExitComparison"
          @export-comparison="handleExportComparison"
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
import FileListPanel from './components/FileListPanel.vue'
import FittingQualityPanel from './components/FittingQualityPanel.vue'
import type { SerializableDataContainer, FileInfo, LogMessage, ProgressUpdate, ProcessingStatus, CurveDisplayData, DataRanges } from './types/data'
import { peakProcessingWorkflow } from './services/peak-processing-workflow'

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

// 文件管理相关状态
const fileListRef = ref()
const selectedFile = ref<{id: string, path: string} | null>(null)
const multiCurveData = ref<Array<{fileId: string, fileName: string, curves: any[]}>>([])
const isComparing = ref(false)

// Plotly曲线管理
const plotlyCurves = ref<Map<string, any>>(new Map()) // 存储已显示的曲线
const plotlyRefs = ref<{original: any, peaks: any, fitted: any}>({
  original: null,
  peaks: null,
  fitted: null
})

// 事件监听器
let statusListener: (() => void) | null = null
let logListener: (() => void) | null = null
let progressListener: (() => void) | null = null

// 生命周期
onMounted(async () => {
  await initializeApp()
  await setupEventListeners()
  await initializePeakProcessingWorkflow()
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

// 初始化峰处理工作流管理器
async function initializePeakProcessingWorkflow() {
  try {
    await peakProcessingWorkflow.initialize()
    addLog('info', '峰处理工作流', '峰处理工作流管理器初始化成功')
    
    // 设置工作流事件监听
    peakProcessingWorkflow.on('processing_started', (event) => {
      addLog('info', '峰处理开始', '开始处理峰数据')
    })
    
    peakProcessingWorkflow.on('processing_completed', (event) => {
      addLog('info', '峰处理完成', '峰数据处理完成')
    })
    
    peakProcessingWorkflow.on('processing_failed', (event) => {
      addLog('error', '峰处理失败', event.data?.error || '未知错误')
    })
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    addLog('error', '峰处理工作流初始化失败', errorMessage)
    ElMessage.error('峰处理工作流初始化失败: ' + errorMessage)
  }
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
    
    // 使用峰处理工作流管理器进行峰拟合
    const request = peakProcessingWorkflow.createAutomaticRequest(
      currentContainer.value.peaks,
      currentContainer.value.curves[0],
      {
        fitting_method: params.method,
        overlapping_method: params.overlapping_method,
        optimization_algorithm: params.optimization_algorithm,
        max_iterations: params.max_iterations,
        convergence_threshold: params.convergence_threshold,
        learning_rate: params.learning_rate,
        advanced_algorithm: params.advanced_algorithm,
        fit_quality_threshold: params.fit_quality_threshold
      }
    )
    
    const response = await peakProcessingWorkflow.processPeaks(request)
    
    if (response.success) {
      currentContainer.value = {
        ...currentContainer.value,
        peaks: response.peaks
      }
      plotMode.value = 'fitted'
      
      addLog('info', '峰拟合完成', `成功拟合 ${response.peaks.length} 个峰`)
      ElMessage.success('峰拟合完成')
    } else {
      throw new Error(response.error || '峰拟合失败')
    }
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
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

// 导出拟合质量报告
async function handleExportQualityReport() {
  if (!currentContainer.value || !currentContainer.value.peaks || currentContainer.value.peaks.length === 0) {
    ElMessage.warning('没有可导出的拟合质量数据')
    return
  }

  try {
    setProcessing(true, '导出质量报告')
    
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('export_fitting_quality_report', {
      container: currentContainer.value,
      includeDetails: true,
      format: 'json'
    })
    
    addLog('info', '质量报告导出', '拟合质量报告导出成功')
    ElMessage.success('拟合质量报告导出成功')
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    addLog('error', '质量报告导出失败', errorMessage)
    ElMessage.error('质量报告导出失败: ' + errorMessage)
  } finally {
    setProcessing(false)
  }
}

// 优化拟合参数
async function handleOptimizeParameters() {
  if (!currentContainer.value || !currentContainer.value.peaks || currentContainer.value.peaks.length === 0) {
    ElMessage.warning('没有可优化的峰数据')
    return
  }

  try {
    setProcessing(true, '优化参数')
    
    // 使用峰处理工作流管理器进行参数优化
    const request = peakProcessingWorkflow.createAutomaticRequest(
      currentContainer.value.peaks,
      currentContainer.value.curves[0],
      {
        optimization_mode: 'adaptive',
        quality_threshold: 0.9,
        max_iterations: 200
      }
    )
    
    const response = await peakProcessingWorkflow.processPeaks(request)
    
    if (response.success) {
      currentContainer.value = {
        ...currentContainer.value,
        peaks: response.peaks
      }
      
      addLog('info', '参数优化', '峰拟合参数优化完成')
      ElMessage.success('峰拟合参数优化完成')
    } else {
      throw new Error(response.error || '参数优化失败')
    }
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    addLog('error', '参数优化失败', errorMessage)
    ElMessage.error('参数优化失败: ' + errorMessage)
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

// 文件管理方法
function handleFileSelected(fileId: string) {
  const fileList = fileListRef.value?.getFileList() || []
  const file = fileList.find((f: any) => f.id === fileId)
  if (file) {
    selectedFile.value = { id: file.id, path: file.path }
    addLog('info', '文件选择', `已选择文件: ${file.path}`)
  }
}

async function handleFileProcessed(fileId: string) {
  const fileList = fileListRef.value?.getFileList() || []
  const file = fileList.find((f: any) => f.id === fileId)
  if (!file) return

  try {
    // 更新文件状态为处理中
    fileListRef.value?.updateFileStatus(fileId, 'processing', 0)
    
    // 1. 加载文件
    await handleLoadFile(file.path)
    
    // 2. 运行完整流水线
    const params = {
      extraction: {
        file_path: file.path,
        mz_range: `${dataRanges.value?.mz_min || 100}-${dataRanges.value?.mz_max || 200}`,
        rt_range: `${dataRanges.value?.rt_min || 0}-${dataRanges.value?.rt_max || 60}`,
        ms_level: 1,
        curve_type: 'dt'
      },
      detection: {
        method: 'cwt',
        sensitivity: 0.7,
        threshold_multiplier: 3.0,
        min_peak_width: 0.1,
        max_peak_width: 10.0
      },
      fitting: {
        method: 'gaussian',
        overlapping_method: 'auto',
        fit_quality_threshold: 0.8,
        max_iterations: 100
      }
    }
    
    await handleRunPipeline(params)
    
    // 更新文件状态为完成
    const result = {
      curvesCount: currentContainer.value?.curves.length || 0,
      peaksCount: currentContainer.value?.peaks.length || 0,
      processingTime: Date.now()
    }
    
    fileListRef.value?.updateFileResult(fileId, result)
    addLog('success', '文件处理完成', `成功处理: ${file.path}`)
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    fileListRef.value?.updateFileStatus(fileId, 'failed', undefined, errorMessage)
    addLog('error', '文件处理失败', `处理失败: ${file.path} - ${errorMessage}`)
  }
}


function handleCompareFiles(fileIds: string[]) {
  const fileList = fileListRef.value?.getFileList() || []
  const completedFiles = fileList.filter((f: any) => 
    fileIds.includes(f.id) && (f.status === 'analyzed' || f.status === 'completed')
  )
  
  if (completedFiles.length === 0) {
    ElMessage.warning('没有可对比的文件')
    return
  }
  
  isComparing.value = true
  multiCurveData.value = completedFiles.map(file => ({
    fileId: file.id,
    fileName: file.path.split('\\').pop() || file.path.split('/').pop() || file.path,
    curves: file.result?.curves || [] // 使用实际的曲线数据
  }))
  
  addLog('info', '开始对比', `正在对比 ${completedFiles.length} 个文件的曲线`)
  ElMessage.success(`开始对比 ${completedFiles.length} 个文件`)
}

// 单文件加载处理
async function handleSingleFileLoad(fileId: string) {
  const fileList = fileListRef.value?.getFileList() || []
  const file = fileList.find((f: any) => f.id === fileId)
  if (!file) return

  try {
    // 更新状态为加载中
    fileListRef.value?.updateFileStatus(fileId, 'loading', 0, undefined, '正在加载文件...')
    
    // 加载文件
    await handleLoadFile(file.path)
    
    // 更新状态为加载完成
    fileListRef.value?.updateFileStatus(fileId, 'loaded', 100, undefined, '文件加载完成')
    
    addLog('success', '文件加载完成', `成功加载: ${file.path}`)
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    fileListRef.value?.updateFileStatus(fileId, 'failed', undefined, errorMessage)
    addLog('error', '文件加载失败', `加载失败: ${file.path} - ${errorMessage}`)
  }
}

// 单文件分析处理
async function handleSingleFileAnalyze(fileId: string) {
  const fileList = fileListRef.value?.getFileList() || []
  const file = fileList.find((f: any) => f.id === fileId)
  if (!file) return

  try {
    // 更新状态为分析中
    fileListRef.value?.updateFileStatus(fileId, 'analyzing', 0, undefined, '正在分析数据...')
    
    // 运行分析流水线
    const params = {
      extraction: {
        file_path: file.path,
        mz_range: `${dataRanges.value?.mz_min || 100}-${dataRanges.value?.mz_max || 200}`,
        rt_range: `${dataRanges.value?.rt_min || 0}-${dataRanges.value?.rt_max || 60}`,
        ms_level: 1,
        curve_type: 'dt'
      },
      detection: {
        method: 'cwt',
        sensitivity: 0.7,
        threshold_multiplier: 3.0,
        min_peak_width: 0.1,
        max_peak_width: 10.0
      },
      fitting: {
        method: 'gaussian',
        overlapping_method: 'auto',
        fit_quality_threshold: 0.8,
        max_iterations: 100
      }
    }
    
    await handleRunPipeline(params)
    
    // 更新状态为分析完成
    const result = {
      curvesCount: currentContainer.value?.curves.length || 0,
      peaksCount: currentContainer.value?.peaks.length || 0,
      processingTime: Date.now(),
      curves: currentContainer.value?.curves || []
    }
    
    fileListRef.value?.updateFileResult(fileId, result)
    addLog('success', '文件分析完成', `成功分析: ${file.path}`)
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    fileListRef.value?.updateFileStatus(fileId, 'failed', undefined, errorMessage)
    addLog('error', '文件分析失败', `分析失败: ${file.path} - ${errorMessage}`)
  }
}

// 文件勾选状态变化处理
function handleFileChecked(fileId: string, checked: boolean) {
  const fileList = fileListRef.value?.getFileList() || []
  const file = fileList.find((f: any) => f.id === fileId)
  if (!file) return

  if (checked) {
    // 勾选：在plotly中显示曲线
    addCurveToPlotly(file)
    addLog('info', '曲线显示', `已显示文件 ${file.path} 的曲线`)
  } else {
    // 取消勾选：从plotly中移除曲线
    removeCurveFromPlotly(fileId)
    addLog('info', '曲线隐藏', `已隐藏文件 ${file.path} 的曲线`)
  }
}

function handleExitComparison() {
  isComparing.value = false
  multiCurveData.value = []
  addLog('info', '退出对比', '已退出多曲线对比模式')
}

async function handleExportComparison() {
  if (multiCurveData.value.length === 0) {
    ElMessage.warning('没有可导出的对比数据')
    return
  }
  
  try {
    const { save } = await import('@tauri-apps/plugin-dialog')
    const filePath = await save({
      title: '保存对比结果',
      filters: [{
        name: 'JSON文件',
        extensions: ['json']
      }],
      defaultPath: 'curve_comparison_results.json'
    })
    
    if (!filePath) return
    
    const exportData = {
      timestamp: new Date().toISOString(),
      comparisonType: 'multi_curve',
      fileCount: multiCurveData.value.length,
      files: multiCurveData.value.map(file => ({
        fileId: file.fileId,
        fileName: file.fileName,
        curveCount: file.curves.length
      }))
    }
    
    // 使用 Tauri 的文件系统 API
    const { writeTextFile } = await import('@tauri-apps/api/fs')
    await writeTextFile(filePath, JSON.stringify(exportData, null, 2))
    
    ElMessage.success('对比结果已导出')
    addLog('success', '导出完成', `对比结果已导出到: ${filePath}`)
    
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    ElMessage.error('导出失败: ' + errorMessage)
    addLog('error', '导出失败', errorMessage)
  }
}

// Plotly曲线管理方法
function addCurveToPlotly(file: any) {
  if (!file.result?.curves) return
  
  const fileName = file.path.split('\\').pop() || file.path.split('/').pop() || file.path
  const curveData = file.result.curves.map((curve: any, index: number) => ({
    x: curve.x_values || [],
    y: curve.y_values || [],
    type: 'scatter',
    mode: 'lines',
    name: `${fileName}_${index + 1}`,
    line: {
      color: getCurveColor(file.id, index),
      width: 2
    },
    visible: true
  }))
  
  plotlyCurves.value.set(file.id, {
    fileName,
    curves: curveData,
    fileId: file.id
  })
  
  // 更新所有三个plotly图表
  updateAllPlotlyCharts()
}

function removeCurveFromPlotly(fileId: string) {
  plotlyCurves.value.delete(fileId)
  updateAllPlotlyCharts()
}

function updateAllPlotlyCharts() {
  const allCurves = Array.from(plotlyCurves.value.values()).flatMap(item => item.curves)
  
  // 更新原始曲线图
  if (plotlyRefs.value.original) {
    plotlyRefs.value.original.data = allCurves
    plotlyRefs.value.original.layout.title = `原始曲线 (${allCurves.length} 条)`
    plotlyRefs.value.original.react()
  }
  
  // 更新峰值图
  if (plotlyRefs.value.peaks) {
    plotlyRefs.value.peaks.data = allCurves
    plotlyRefs.value.peaks.layout.title = `峰值检测 (${allCurves.length} 条)`
    plotlyRefs.value.peaks.react()
  }
  
  // 更新拟合图
  if (plotlyRefs.value.fitted) {
    plotlyRefs.value.fitted.data = allCurves
    plotlyRefs.value.fitted.layout.title = `峰拟合 (${allCurves.length} 条)`
    plotlyRefs.value.fitted.react()
  }
}

function getCurveColor(fileId: string, curveIndex: number): string {
  const colors = [
    '#1f77b4', '#ff7f0e', '#2ca02c', '#d62728', '#9467bd',
    '#8c564b', '#e377c2', '#7f7f7f', '#bcbd22', '#17becf'
  ]
  
  // 基于文件ID和曲线索引生成颜色
  const hash = fileId.split('').reduce((a, b) => {
    a = ((a << 5) - a) + b.charCodeAt(0)
    return a & a
  }, 0)
  
  return colors[Math.abs(hash + curveIndex) % colors.length]
}

// 批量处理优化 - 逐步加载与分析
async function handleProcessAll() {
  const fileList = fileListRef.value?.getFileList() || []
  const pendingFiles = fileList.filter((f: any) => f.status === 'pending')
  
  if (pendingFiles.length === 0) {
    ElMessage.warning('没有待处理的文件')
    return
  }
  
  addLog('info', '批量处理开始', `开始批量处理 ${pendingFiles.length} 个文件`)
  
  // 第一阶段：批量加载所有文件
  addLog('info', '批量加载', '开始批量加载文件...')
  for (const file of pendingFiles) {
    await handleSingleFileLoad(file.id)
    await new Promise(resolve => setTimeout(resolve, 100)) // 短暂延迟
  }
  
  // 第二阶段：批量分析所有已加载的文件
  const loadedFiles = fileList.filter((f: any) => f.status === 'loaded')
  if (loadedFiles.length > 0) {
    addLog('info', '批量分析', `开始批量分析 ${loadedFiles.length} 个文件...`)
    for (const file of loadedFiles) {
      await handleSingleFileAnalyze(file.id)
      await new Promise(resolve => setTimeout(resolve, 100)) // 短暂延迟
    }
  }
  
  addLog('info', '批量处理完成', `已完成所有文件的处理`)
  ElMessage.success('批量处理完成')
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
