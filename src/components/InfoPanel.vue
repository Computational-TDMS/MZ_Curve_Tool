<template>
  <div class="info-panel">
    <!-- 文件信息卡片 -->
    <el-card class="info-card" v-if="fileInfo">
      <template #header>
        <div class="card-header">
          <el-icon><Document /></el-icon>
          <span>文件信息</span>
        </div>
      </template>
      
      <div class="file-info">
        <div class="info-item">
          <span class="label">文件名:</span>
          <span class="value">{{ fileInfo.name }}</span>
        </div>
        <div class="info-item">
          <span class="label">大小:</span>
          <span class="value">{{ formatFileSize(fileInfo.size) }}</span>
        </div>
        <div class="info-item">
          <span class="label">格式:</span>
          <span class="value">{{ fileInfo.format.toUpperCase() }}</span>
        </div>
        <div class="info-item">
          <span class="label">谱图数:</span>
          <span class="value">{{ fileInfo.spectra_count || '未知' }}</span>
        </div>
        <div class="info-item">
          <span class="label">状态:</span>
          <el-tag :type="fileInfo.is_valid ? 'success' : 'danger'">
            {{ fileInfo.is_valid ? '有效' : '无效' }}
          </el-tag>
        </div>
      </div>
    </el-card>

    <!-- 处理状态卡片 -->
    <el-card class="info-card">
      <template #header>
        <div class="card-header">
          <el-icon><Monitor /></el-icon>
          <span>处理状态</span>
        </div>
      </template>
      
      <div class="status-info">
        <div class="status-item">
          <span class="status-indicator" :class="statusClass"></span>
          <span class="status-text">{{ statusText }}</span>
        </div>
        <div class="status-details" v-if="statusDetails">
          <p>{{ statusDetails }}</p>
        </div>
      </div>
    </el-card>

    <!-- 曲线数据卡片 -->
    <el-card class="info-card" v-if="curveData && curveData.length > 0">
      <template #header>
        <div class="card-header">
          <el-icon><TrendCharts /></el-icon>
          <span>曲线数据</span>
          <el-button 
            type="primary" 
            size="small" 
            @click="handleExportCurves"
            style="margin-left: auto;"
          >
            导出TSV
          </el-button>
        </div>
      </template>
      
      <div class="curve-data">
        <div v-for="curve in curveData" :key="curve.id" class="curve-item">
          <div class="curve-header">
            <span class="curve-type">{{ curve.curve_type }}</span>
            <span class="curve-points">{{ curve.point_count }} 点</span>
          </div>
          <div class="curve-info">
            <div class="info-row">
              <span class="label">X轴:</span>
              <span class="value">{{ curve.x_label }} ({{ curve.x_unit }})</span>
            </div>
            <div class="info-row">
              <span class="label">Y轴:</span>
              <span class="value">{{ curve.y_label }} ({{ curve.y_unit }})</span>
            </div>
            <div v-if="curve.mz_min && curve.mz_max" class="info-row">
              <span class="label">m/z范围:</span>
              <span class="value">{{ curve.mz_min.toFixed(2) }} - {{ curve.mz_max.toFixed(2) }}</span>
            </div>
          </div>
          
          <!-- 数据表格 -->
          <div class="curve-table">
            <el-table 
              :data="getCurveTableData(curve)" 
              size="small" 
              max-height="200"
              style="width: 100%"
            >
              <el-table-column prop="x" :label="curve.x_label" width="80">
                <template #default="{ row }">
                  {{ row.x.toFixed(3) }}
                </template>
              </el-table-column>
              <el-table-column prop="y" :label="curve.y_label" width="80">
                <template #default="{ row }">
                  {{ row.y.toFixed(3) }}
                </template>
              </el-table-column>
            </el-table>
          </div>
        </div>
      </div>
    </el-card>

    <!-- 多曲线对比卡片 -->
    <el-card class="info-card" v-if="isComparing && multiCurveData && multiCurveData.length > 0">
      <template #header>
        <div class="card-header">
          <el-icon><TrendCharts /></el-icon>
          <span>多曲线对比</span>
          <el-button 
            type="text" 
            size="small" 
            @click="exitComparison"
            style="margin-left: auto;"
          >
            退出对比
          </el-button>
        </div>
      </template>
      
      <div class="comparison-info">
        <div class="comparison-summary">
          <el-tag type="info" size="small">
            对比文件: {{ multiCurveData.length }} 个
          </el-tag>
        </div>
        
        <div class="comparison-list">
          <div 
            v-for="fileData in multiCurveData" 
            :key="fileData.fileId"
            class="comparison-item"
          >
            <div class="file-name">
              <el-icon><Document /></el-icon>
              <span>{{ fileData.fileName }}</span>
            </div>
            <div class="curve-count">
              <el-tag size="small" type="success">
                曲线: {{ fileData.curves.length }}
              </el-tag>
            </div>
          </div>
        </div>
        
        <div class="comparison-actions">
          <el-button 
            type="primary" 
            size="small" 
            @click="exportComparison"
            :icon="Download"
          >
            导出对比结果
          </el-button>
        </div>
      </div>
    </el-card>

    <!-- 处理日志卡片 -->
    <el-card class="info-card log-card">
      <template #header>
        <div class="card-header">
          <el-icon><ChatDotRound /></el-icon>
          <span>处理日志</span>
          <el-button 
            size="small" 
            type="danger" 
            plain 
            @click="clearLogs"
            style="margin-left: auto"
          >
            清空
          </el-button>
        </div>
      </template>
      
      <div class="log-container" ref="logContainer">
        <div 
          v-for="log in logs" 
          :key="log.id"
          class="log-entry"
          :class="`log-${log.level}`"
        >
          <div class="log-header">
            <span class="log-time">{{ log.timestamp }}</span>
            <el-tag 
              :type="getLogTagType(log.level)" 
              size="small"
              effect="plain"
            >
              {{ log.title }}
            </el-tag>
          </div>
          <div class="log-content">{{ log.content }}</div>
        </div>
        
        <div v-if="logs.length === 0" class="empty-logs">
          <el-empty description="暂无日志" :image-size="60" />
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, nextTick } from 'vue'
import { Document, Monitor, ChatDotRound, TrendCharts, Download } from '@element-plus/icons-vue'
import type { CurveDisplayData } from '../types/data'

// 定义props
const props = defineProps<{
  fileInfo: any
  status: string
  logs: Array<{
    id: string
    level: string
    title: string
    content: string
    timestamp: string
  }>
  curveData?: CurveDisplayData[]
  multiCurveData?: Array<{fileId: string, fileName: string, curves: any[]}>
  isComparing?: boolean
}>()

// 定义emits
const emit = defineEmits<{
  'export-curves': []
  'exit-comparison': []
  'export-comparison': []
}>()

// 响应式数据
const logContainer = ref<HTMLElement>()

// 计算属性
const statusClass = computed(() => {
  switch (props.status) {
    case 'idle':
      return 'status-idle'
    case 'loading':
      return 'status-loading'
    case 'processing':
      return 'status-processing'
    case 'success':
      return 'status-success'
    case 'error':
      return 'status-error'
    default:
      return 'status-idle'
  }
})

const statusText = computed(() => {
  switch (props.status) {
    case 'idle':
      return '空闲'
    case 'loading':
      return '加载中'
    case 'processing':
      return '处理中'
    case 'success':
      return '成功'
    case 'error':
      return '错误'
    default:
      return '未知'
  }
})

const statusDetails = computed(() => {
  if (props.fileInfo && props.fileInfo.is_valid) {
    return `文件已加载，包含 ${props.fileInfo.spectra_count || 0} 个谱图`
  }
  return null
})

// 监听器
watch(() => props.logs, () => {
  nextTick(() => {
    scrollToBottom()
  })
}, { deep: true })

// 方法
function formatFileSize(bytes: number): string {
  if (bytes === 0) return '0 B'
  
  const k = 1024
  const sizes = ['B', 'KB', 'MB', 'GB']
  const i = Math.floor(Math.log(bytes) / Math.log(k))
  
  return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + sizes[i]
}

function getLogTagType(level: string): string {
  switch (level) {
    case 'info':
      return 'info'
    case 'success':
      return 'success'
    case 'warning':
      return 'warning'
    case 'error':
      return 'danger'
    default:
      return ''
  }
}

function scrollToBottom() {
  if (logContainer.value) {
    logContainer.value.scrollTop = logContainer.value.scrollHeight
  }
}

function clearLogs() {
  // 这里可以emit一个事件来清空日志
  // emit('clear-logs')
}

function handleExportCurves() {
  emit('export-curves')
}

function exitComparison() {
  emit('exit-comparison')
}

function exportComparison() {
  emit('export-comparison')
}

function getCurveTableData(curve: CurveDisplayData) {
  // 只显示前20个数据点
  const maxPoints = 20
  return curve.x_values.slice(0, maxPoints).map((x, index) => ({
    x,
    y: curve.y_values[index]
  }))
}
</script>

<style scoped>
.info-panel {
  height: 100%;
  padding: 16px;
  overflow-y: auto;
}

.info-card {
  margin-bottom: 16px;
}

.info-card:last-child {
  margin-bottom: 0;
}

.card-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  color: #303133;
}

/* 文件信息样式 */
.file-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.info-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 0;
}

.info-item .label {
  font-weight: 500;
  color: #606266;
  font-size: 14px;
}

.info-item .value {
  color: #303133;
  font-size: 14px;
  text-align: right;
  max-width: 60%;
  word-break: break-all;
}

/* 状态信息样式 */
.status-info {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.status-item {
  display: flex;
  align-items: center;
  gap: 8px;
}

.status-indicator {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  display: inline-block;
}

.status-idle {
  background-color: #909399;
}

.status-loading {
  background-color: #409EFF;
  animation: pulse 1.5s ease-in-out infinite;
}

.status-processing {
  background-color: #E6A23C;
  animation: pulse 1.5s ease-in-out infinite;
}

.status-success {
  background-color: #67C23A;
}

.status-error {
  background-color: #F56C6C;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.5;
  }
}

.status-text {
  font-weight: 500;
  color: #303133;
}

.status-details {
  font-size: 12px;
  color: #909399;
  margin-top: 4px;
}

/* 曲线数据样式 */
.curve-data {
  max-height: 400px;
  overflow-y: auto;
}

.curve-item {
  margin-bottom: 16px;
  padding: 12px;
  border: 1px solid #e4e7ed;
  border-radius: 6px;
  background-color: #fafafa;
}

.curve-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.curve-type {
  font-weight: 600;
  color: #409eff;
}

.curve-points {
  font-size: 12px;
  color: #909399;
}

/* 多曲线对比样式 */
.comparison-info {
  padding: 8px 0;
}

.comparison-summary {
  margin-bottom: 12px;
}

.comparison-list {
  margin-bottom: 16px;
}

.comparison-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  margin-bottom: 8px;
  background: #f8f9fa;
  border-radius: 4px;
  border: 1px solid #e9ecef;
}

.file-name {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
  color: #303133;
}

.comparison-actions {
  display: flex;
  justify-content: center;
}

.curve-info {
  margin-bottom: 8px;
}

.info-row {
  display: flex;
  margin-bottom: 4px;
  font-size: 12px;
}

.info-row .label {
  width: 60px;
  color: #606266;
  font-weight: 500;
}

.info-row .value {
  color: #303133;
}

.curve-table {
  margin-top: 8px;
}

/* 日志样式 */
.log-card {
  flex: 1;
  display: flex;
  flex-direction: column;
}

.log-card :deep(.el-card__body) {
  flex: 1;
  display: flex;
  flex-direction: column;
  padding: 16px;
}

.log-container {
  flex: 1;
  overflow-y: auto;
  max-height: 300px;
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  padding: 8px;
  background: #fafafa;
}

.log-entry {
  margin-bottom: 8px;
  padding: 8px;
  border-radius: 4px;
  background: #fff;
  border-left: 3px solid #e4e7ed;
}

.log-entry:last-child {
  margin-bottom: 0;
}

.log-info {
  border-left-color: #409EFF;
}

.log-success {
  border-left-color: #67C23A;
}

.log-warning {
  border-left-color: #E6A23C;
}

.log-error {
  border-left-color: #F56C6C;
}

.log-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.log-time {
  font-size: 11px;
  color: #909399;
  font-family: 'Courier New', monospace;
}

.log-content {
  font-size: 13px;
  color: #303133;
  line-height: 1.4;
  word-break: break-word;
}

.empty-logs {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100px;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .info-panel {
    padding: 12px;
  }
  
  .info-card {
    margin-bottom: 12px;
  }
  
  .log-container {
    max-height: 200px;
  }
}
</style>
