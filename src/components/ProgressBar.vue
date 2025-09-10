<template>
  <div class="progress-bar-container">
    <div class="progress-info">
      <span class="progress-message">{{ message }}</span>
      <span class="progress-percentage" v-if="total > 0">
        {{ Math.round((current / total) * 100) }}%
      </span>
    </div>
    
    <el-progress 
      :percentage="percentage"
      :status="progressStatus"
      :stroke-width="8"
      :show-text="false"
      class="progress-bar"
    />
    
    <div class="progress-details" v-if="total > 0">
      <span class="progress-count">{{ current }} / {{ total }}</span>
      <span class="progress-time" v-if="estimatedTime">
        预计剩余: {{ formatTime(estimatedTime) }}
      </span>
    </div>
  </div>
</template>

<script setup lang="ts">
import { computed, ref, watch } from 'vue'

// 定义props
const props = defineProps<{
  current: number
  total: number
  message: string
}>()

// 响应式数据
const startTime = ref<number>(0)
const estimatedTime = ref<number>(0)

// 计算属性
const percentage = computed(() => {
  if (props.total <= 0) return 0
  return Math.min(Math.round((props.current / props.total) * 100), 100)
})

const progressStatus = computed(() => {
  if (props.total <= 0) return 'success'
  if (props.current >= props.total) return 'success'
  if (props.current > 0) return 'active'
  return 'success'
})

// 监听器
watch(() => props.current, (newCurrent, oldCurrent) => {
  if (newCurrent > oldCurrent && oldCurrent === 0) {
    // 开始处理
    startTime.value = Date.now()
  }
  
  if (newCurrent > 0 && props.total > 0) {
    // 计算预计剩余时间
    const elapsed = Date.now() - startTime.value
    const rate = newCurrent / elapsed // 每秒处理的项目数
    const remaining = props.total - newCurrent
    estimatedTime.value = remaining / rate
  }
})

watch(() => props.total, (newTotal) => {
  if (newTotal > 0 && props.current === 0) {
    startTime.value = Date.now()
  }
})

// 方法
function formatTime(milliseconds: number): string {
  if (milliseconds < 1000) {
    return `${Math.round(milliseconds)}ms`
  } else if (milliseconds < 60000) {
    return `${Math.round(milliseconds / 1000)}s`
  } else {
    const minutes = Math.floor(milliseconds / 60000)
    const seconds = Math.round((milliseconds % 60000) / 1000)
    return `${minutes}m ${seconds}s`
  }
}
</script>

<style scoped>
.progress-bar-container {
  width: 100%;
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.progress-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.progress-message {
  font-size: 14px;
  font-weight: 500;
  color: #303133;
}

.progress-percentage {
  font-size: 14px;
  font-weight: 600;
  color: #409EFF;
}

.progress-bar {
  width: 100%;
}

.progress-details {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-size: 12px;
  color: #909399;
}

.progress-count {
  font-family: 'Courier New', monospace;
}

.progress-time {
  font-style: italic;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .progress-info {
    flex-direction: column;
    align-items: flex-start;
    gap: 4px;
  }
  
  .progress-details {
    flex-direction: column;
    align-items: flex-start;
    gap: 2px;
  }
}
</style>
