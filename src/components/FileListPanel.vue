<template>
  <div class="file-list-panel">
    <el-card class="panel-card">
      <template #header>
        <div class="card-header">
          <span>文件列表</span>
          <div class="header-actions">
            <el-button 
              type="primary" 
              size="small" 
              @click="addFiles"
              :icon="Plus"
            >
              添加文件
            </el-button>
            <el-button 
              size="small" 
              @click="clearAll"
              :icon="Delete"
            >
              清空
            </el-button>
          </div>
        </div>
      </template>

      <!-- 文件列表 -->
      <div class="file-list">
        <el-scrollbar height="300px">
          <div 
            v-for="(file, index) in fileList" 
            :key="file.id"
            class="file-item"
            :class="{ 
              'selected': selectedFileId === file.id,
              'loading': file.status === 'loading',
              'loaded': file.status === 'loaded',
              'analyzing': file.status === 'analyzing',
              'analyzed': file.status === 'analyzed',
              'completed': file.status === 'completed',
              'failed': file.status === 'failed'
            }"
          >
            <div class="file-header">
              <div class="file-checkbox">
                <el-checkbox 
                  v-model="file.checked"
                  @change="handleFileChecked(file.id, file.checked)"
                  :disabled="file.status !== 'completed'"
                />
              </div>
              
              <div class="file-info" @click="selectFile(file.id)">
                <div class="file-name">
                  <el-icon class="file-icon">
                    <Document />
                  </el-icon>
                  <span class="name">{{ getFileName(file.path) }}</span>
                </div>
                
                <div class="file-status">
                  <el-tag 
                    :type="getStatusType(file.status)" 
                    size="small"
                    effect="plain"
                  >
                    {{ getStatusText(file.status) }}
                  </el-tag>
                  <span v-if="file.currentStep" class="current-step">
                    {{ file.currentStep }}
                  </span>
                </div>
              </div>
            </div>

            <div class="file-actions">
              <!-- 待处理状态 -->
              <el-button 
                v-if="file.status === 'pending'"
                type="primary" 
                size="small" 
                @click.stop="loadFile(file.id)"
              >
                加载
              </el-button>
              
              <!-- 加载完成状态 -->
              <el-button 
                v-if="file.status === 'loaded'"
                type="success" 
                size="small" 
                @click.stop="analyzeFile(file.id)"
              >
                分析
              </el-button>
              
              <!-- 分析完成状态 -->
              <el-button 
                v-if="file.status === 'analyzed'"
                type="success" 
                size="small" 
                @click.stop="viewResults(file.id)"
              >
                查看
              </el-button>
              
              <!-- 已完成状态 -->
              <el-button 
                v-if="file.status === 'completed'"
                type="success" 
                size="small" 
                @click.stop="viewResults(file.id)"
              >
                查看
              </el-button>
              
              <!-- 失败状态 -->
              <el-button 
                v-if="file.status === 'failed'"
                type="warning" 
                size="small" 
                @click.stop="retryFile(file.id)"
              >
                重试
              </el-button>
              
              <!-- 删除按钮 -->
              <el-button 
                type="danger" 
                size="small" 
                @click.stop="removeFile(file.id)"
                :icon="Delete"
              />
            </div>

            <!-- 处理进度 -->
            <div v-if="file.status === 'loading' || file.status === 'analyzing'" class="progress-bar">
              <el-progress 
                :percentage="file.progress || 0" 
                :stroke-width="4"
                :show-text="false"
              />
            </div>

            <!-- 错误信息 -->
            <div v-if="file.status === 'failed' && file.error" class="error-message">
              <el-icon><Warning /></el-icon>
              <span>{{ file.error }}</span>
            </div>

            <!-- 处理结果摘要 -->
            <div v-if="(file.status === 'analyzed' || file.status === 'completed') && file.result" class="result-summary">
              <el-tag size="small" type="info">
                曲线: {{ file.result.curvesCount }}
              </el-tag>
              <el-tag size="small" type="success" style="margin-left: 4px;">
                峰值: {{ file.result.peaksCount }}
              </el-tag>
            </div>
          </div>
        </el-scrollbar>
      </div>

      <!-- 批量操作 -->
      <div v-if="fileList.length > 0" class="batch-actions">
        <el-divider />
        <div class="batch-buttons">
          <el-button 
            type="success" 
            @click="processAllPending"
            :loading="isProcessingAll"
            :disabled="!hasPendingFiles"
          >
            处理所有待处理文件
          </el-button>
          
          <el-button 
            type="primary" 
            @click="compareAllCompleted"
            :disabled="!hasCompletedFiles"
          >
            对比所有已处理文件
          </el-button>
        </div>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { Plus, Delete, Document, Warning } from '@element-plus/icons-vue'

// 文件状态类型
type FileStatus = 'pending' | 'loading' | 'loaded' | 'analyzing' | 'analyzed' | 'completed' | 'failed'

// 文件项接口
interface FileItem {
  id: string
  path: string
  status: FileStatus
  progress?: number
  error?: string
  result?: {
    curvesCount: number
    peaksCount: number
    processingTime: number
    curves?: any[] // 存储曲线数据
  }
  selected?: boolean
  checked?: boolean // 用于控制曲线显示
  currentStep?: string // 当前处理步骤
}

// 定义事件
const emit = defineEmits<{
  'file-selected': [fileId: string]
  'file-processed': [fileId: string]
  'process-all': []
  'compare-files': [fileIds: string[]]
  'file-checked': [fileId: string, checked: boolean]
  'single-file-load': [fileId: string]
  'single-file-analyze': [fileId: string]
}>()

// 响应式数据
const fileList = ref<FileItem[]>([])
const selectedFileId = ref<string | null>(null)
const isProcessingAll = ref(false)

// 计算属性
const hasPendingFiles = computed(() => 
  fileList.value.some(file => file.status === 'pending')
)

const hasCompletedFiles = computed(() => 
  fileList.value.some(file => file.status === 'analyzed' || file.status === 'completed')
)

// 方法
function getFileName(path: string): string {
  return path.split('\\').pop() || path.split('/').pop() || path
}

function getStatusType(status: FileStatus): string {
  switch (status) {
    case 'pending': return 'info'
    case 'loading': return 'warning'
    case 'loaded': return 'primary'
    case 'analyzing': return 'warning'
    case 'analyzed': return 'primary'
    case 'completed': return 'success'
    case 'failed': return 'danger'
    default: return 'info'
  }
}

function getStatusText(status: FileStatus): string {
  switch (status) {
    case 'pending': return '待处理'
    case 'loading': return '加载中'
    case 'loaded': return '加载完成'
    case 'analyzing': return '分析中'
    case 'analyzed': return '分析完成'
    case 'completed': return '已完成'
    case 'failed': return '失败'
    default: return '未知'
  }
}

async function addFiles() {
  try {
    const { open } = await import('@tauri-apps/plugin-dialog')
    const selected = await open({
      title: '选择质谱数据文件',
      filters: [{
        name: '质谱数据文件',
        extensions: ['mzml', 'mzxml', 'mzdata', 'raw', 'd']
      }, {
        name: '所有文件',
        extensions: ['*']
      }],
      multiple: true,
      defaultPath: ''
    })
    
    if (selected && Array.isArray(selected) && selected.length > 0) {
      const newFiles: FileItem[] = selected.map(path => ({
        id: generateId(),
        path,
        status: 'pending' as FileStatus
      }))
      
      fileList.value.push(...newFiles)
      ElMessage.success(`已添加 ${selected.length} 个文件`)
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    ElMessage.error('添加文件失败: ' + errorMessage)
  }
}

function selectFile(fileId: string) {
  selectedFileId.value = fileId
  emit('file-selected', fileId)
}

function loadFile(fileId: string) {
  const file = fileList.value.find(f => f.id === fileId)
  if (file) {
    file.status = 'loading'
    file.progress = 0
    file.currentStep = '正在加载文件...'
    emit('single-file-load', fileId)
  }
}

function analyzeFile(fileId: string) {
  const file = fileList.value.find(f => f.id === fileId)
  if (file) {
    file.status = 'analyzing'
    file.progress = 0
    file.currentStep = '正在分析数据...'
    emit('single-file-analyze', fileId)
  }
}

function handleFileChecked(fileId: string, checked: boolean) {
  emit('file-checked', fileId, checked)
}

function viewResults(fileId: string) {
  selectFile(fileId)
  // 可以添加查看结果的逻辑
}

function retryFile(fileId: string) {
  const file = fileList.value.find(f => f.id === fileId)
  if (file) {
    file.status = 'pending'
    file.error = undefined
    file.progress = 0
  }
}

function removeFile(fileId: string) {
  const index = fileList.value.findIndex(f => f.id === fileId)
  if (index > -1) {
    fileList.value.splice(index, 1)
    
    if (selectedFileId.value === fileId) {
      selectedFileId.value = null
    }
    
    ElMessage.info('文件已移除')
  }
}

function clearAll() {
  fileList.value = []
  selectedFileId.value = null
  ElMessage.info('已清空所有文件')
}

function processAllPending() {
  const pendingFiles = fileList.value.filter(f => f.status === 'pending')
  if (pendingFiles.length === 0) {
    ElMessage.warning('没有待处理的文件')
    return
  }
  
  isProcessingAll.value = true
  emit('process-all')
}

function compareAllCompleted() {
  const completedFiles = fileList.value.filter(f => f.status === 'completed')
  if (completedFiles.length === 0) {
    ElMessage.warning('没有已完成的文件')
    return
  }
  
  emit('compare-files', completedFiles.map(f => f.id))
}

function generateId(): string {
  return Date.now().toString() + Math.random().toString(36).substr(2, 9)
}

// 暴露方法给父组件
defineExpose({
  updateFileStatus: (fileId: string, status: FileStatus, progress?: number, error?: string, currentStep?: string) => {
    const file = fileList.value.find(f => f.id === fileId)
    if (file) {
      file.status = status
      if (progress !== undefined) file.progress = progress
      if (error !== undefined) file.error = error
      if (currentStep !== undefined) file.currentStep = currentStep
    }
  },
  
  updateFileResult: (fileId: string, result: FileItem['result']) => {
    const file = fileList.value.find(f => f.id === fileId)
    if (file) {
      file.result = result
      file.status = 'completed'
      file.progress = 100
      file.currentStep = undefined
    }
  },
  
  getSelectedFile: () => {
    return fileList.value.find(f => f.id === selectedFileId.value)
  },
  
  getFileList: () => fileList.value,
  
  getCheckedFiles: () => {
    return fileList.value.filter(f => f.checked && f.status === 'completed')
  }
})
</script>

<style scoped>
.file-list-panel {
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

.header-actions {
  display: flex;
  gap: 8px;
}

.file-list {
  margin-bottom: 16px;
}

.file-item {
  padding: 12px;
  margin-bottom: 8px;
  border: 1px solid #e4e7ed;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.3s;
  background: #fff;
}

.file-item:hover {
  border-color: #409eff;
  box-shadow: 0 2px 8px rgba(64, 158, 255, 0.1);
}

.file-item.selected {
  border-color: #409eff;
  background: #f0f9ff;
}

.file-item.loading {
  border-color: #e6a23c;
  background: #fdf6ec;
}

.file-item.loaded {
  border-color: #409eff;
  background: #ecf5ff;
}

.file-item.analyzing {
  border-color: #e6a23c;
  background: #fdf6ec;
}

.file-item.analyzed {
  border-color: #409eff;
  background: #ecf5ff;
}

.file-item.completed {
  border-color: #67c23a;
  background: #f0f9ff;
}

.file-item.failed {
  border-color: #f56c6c;
  background: #fef0f0;
}

.file-header {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.file-checkbox {
  flex-shrink: 0;
}

.file-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex: 1;
  cursor: pointer;
}

.current-step {
  font-size: 12px;
  color: #909399;
  margin-left: 8px;
}

.file-name {
  display: flex;
  align-items: center;
  gap: 8px;
  flex: 1;
}

.file-icon {
  color: #606266;
}

.name {
  font-weight: 500;
  color: #303133;
  word-break: break-all;
}

.file-actions {
  display: flex;
  gap: 4px;
}

.progress-bar {
  margin-top: 8px;
}

.error-message {
  display: flex;
  align-items: center;
  gap: 4px;
  margin-top: 8px;
  font-size: 12px;
  color: #f56c6c;
}

.result-summary {
  margin-top: 8px;
  display: flex;
  gap: 4px;
}

.batch-actions {
  margin-top: 16px;
}

.batch-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}
</style>
