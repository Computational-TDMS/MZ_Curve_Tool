<template>
  <div class="batch-processing-panel">
    <el-card class="panel-card">
      <template #header>
        <div class="card-header">
          <span>批量处理状态</span>
          <el-button 
            v-if="isProcessing" 
            type="danger" 
            size="small" 
            @click="stopBatchProcessing"
          >
            停止处理
          </el-button>
        </div>
      </template>

      <!-- 处理进度 -->
      <div v-if="isProcessing || results.length > 0" class="processing-status">
        <el-progress 
          :percentage="progressPercentage" 
          :status="progressStatus"
          :stroke-width="8"
        />
        <div class="progress-info">
          <span>{{ progressMessage }}</span>
          <span class="progress-numbers">{{ current }}/{{ total }}</span>
        </div>
        
        <!-- 拟合精度统计 -->
        <div v-if="fittingStats" class="fitting-stats">
          <el-row :gutter="8">
            <el-col :span="8">
              <div class="stat-item">
                <span class="stat-label">平均R²</span>
                <span class="stat-value">{{ fittingStats.avgRSquared.toFixed(3) }}</span>
              </div>
            </el-col>
            <el-col :span="8">
              <div class="stat-item">
                <span class="stat-label">平均迭代</span>
                <span class="stat-value">{{ fittingStats.avgIterations.toFixed(0) }}</span>
              </div>
            </el-col>
            <el-col :span="8">
              <div class="stat-item">
                <span class="stat-label">收敛率</span>
                <span class="stat-value">{{ (fittingStats.convergenceRate * 100).toFixed(1) }}%</span>
              </div>
            </el-col>
          </el-row>
        </div>
      </div>

      <!-- 处理结果列表 -->
      <div v-if="results.length > 0" class="results-section">
        <el-divider content-position="left">处理结果</el-divider>
        
        <div class="results-summary">
          <el-tag type="success" size="small">
            成功: {{ successCount }}
          </el-tag>
          <el-tag type="danger" size="small" style="margin-left: 8px;">
            失败: {{ failCount }}
          </el-tag>
        </div>

        <el-scrollbar height="200px" class="results-list">
          <div 
            v-for="(result, index) in results" 
            :key="index"
            class="result-item"
            :class="{ 'success': result.success, 'error': !result.success }"
          >
            <div class="result-header">
              <el-icon>
                <Check v-if="result.success" />
                <Close v-else />
              </el-icon>
              <span class="file-name">
                {{ getFileName(result.filePath) }}
              </span>
            </div>
            <div v-if="!result.success && result.error" class="error-message">
              {{ result.error }}
            </div>
          </div>
        </el-scrollbar>
      </div>

      <!-- 操作按钮 -->
      <div v-if="results.length > 0" class="action-buttons">
        <el-button 
          type="primary" 
          size="small" 
          @click="exportResults"
          :icon="Download"
        >
          导出结果
        </el-button>
        <el-button 
          size="small" 
          @click="clearResults"
          :icon="Delete"
        >
          清空结果
        </el-button>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { ElMessage } from 'element-plus'
import { Check, Close, Download, Delete } from '@element-plus/icons-vue'

// 定义 props
const props = defineProps<{
  isProcessing: boolean
  current: number
  total: number
  progressMessage: string
  results: Array<{
    filePath: string
    success: boolean
    error?: string
    fittingStats?: {
      avgRSquared: number
      avgIterations: number
      convergenceRate: number
    }
  }>
  fittingStats?: {
    avgRSquared: number
    avgIterations: number
    convergenceRate: number
  }
}>()

// 定义事件
const emit = defineEmits<{
  'stop-processing': []
  'export-results': []
  'clear-results': []
}>()

// 计算属性
const progressPercentage = computed(() => {
  if (props.total === 0) return 0
  return Math.round((props.current / props.total) * 100)
})

const progressStatus = computed(() => {
  if (props.isProcessing) return 'active'
  if (props.results.length === 0) return 'normal'
  
  const hasErrors = props.results.some(r => !r.success)
  return hasErrors ? 'exception' : 'success'
})

const successCount = computed(() => 
  props.results.filter(r => r.success).length
)

const failCount = computed(() => 
  props.results.filter(r => !r.success).length
)

// 方法
function getFileName(filePath: string): string {
  return filePath.split('\\').pop() || filePath.split('/').pop() || filePath
}

function stopBatchProcessing() {
  emit('stop-processing')
  ElMessage.info('正在停止批量处理...')
}

function exportResults() {
  emit('export-results')
}

function clearResults() {
  emit('clear-results')
  ElMessage.info('已清空处理结果')
}
</script>

<style scoped>
.batch-processing-panel {
  height: 100%;
  padding: 16px;
}

.panel-card {
  height: 100%;
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: bold;
  color: #303133;
}

.processing-status {
  margin-bottom: 16px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 8px;
  font-size: 14px;
  color: #606266;
}

.progress-numbers {
  font-weight: bold;
  color: #409eff;
}

.results-section {
  margin-top: 16px;
}

.results-summary {
  margin-bottom: 12px;
}

.results-list {
  border: 1px solid #e4e7ed;
  border-radius: 4px;
  padding: 8px;
}

.result-item {
  padding: 8px;
  margin-bottom: 4px;
  border-radius: 4px;
  border-left: 4px solid transparent;
}

.result-item.success {
  background-color: #f0f9ff;
  border-left-color: #67c23a;
}

.result-item.error {
  background-color: #fef0f0;
  border-left-color: #f56c6c;
}

.result-header {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-name {
  font-weight: 500;
  color: #303133;
}

.error-message {
  margin-top: 4px;
  font-size: 12px;
  color: #f56c6c;
  padding-left: 24px;
}

.action-buttons {
  margin-top: 16px;
  display: flex;
  gap: 8px;
}

.fitting-stats {
  margin-top: 12px;
  padding: 12px;
  background-color: #f8f9fa;
  border-radius: 6px;
  border: 1px solid #e9ecef;
}

.stat-item {
  text-align: center;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.stat-label {
  font-size: 12px;
  color: #6c757d;
  font-weight: 500;
}

.stat-value {
  font-size: 16px;
  color: #495057;
  font-weight: bold;
}
</style>
