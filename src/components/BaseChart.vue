<template>
  <div class="base-chart">
    <div class="chart-header">
      <el-row justify="space-between" align="middle">
        <el-col :span="12">
          <h4>{{ chartTitle }}</h4>
        </el-col>
        <el-col :span="12" style="text-align: right">
          <el-button-group size="small">
            <el-button 
              :type="displayMode === 'line' ? 'primary' : ''"
              @click="setDisplayMode('line')"
            >
              线图
            </el-button>
            <el-button 
              :type="displayMode === 'scatter' ? 'primary' : ''"
              @click="setDisplayMode('scatter')"
            >
              散点图
            </el-button>
            <el-button 
              :type="displayMode === 'both' ? 'primary' : ''"
              @click="setDisplayMode('both')"
            >
              混合
            </el-button>
          </el-button-group>
        </el-col>
      </el-row>
      
      <!-- 图表控制选项 -->
      <div class="chart-controls">
        <el-row :gutter="12" align="middle">
          <el-col :span="6">
            <el-checkbox v-model="showGrid" @change="updateChart">
              显示网格
            </el-checkbox>
          </el-col>
          <el-col :span="6">
            <el-checkbox v-model="showLegend" @change="updateChart">
              显示图例
            </el-checkbox>
          </el-col>
          <el-col :span="6">
            <el-checkbox v-model="smoothCurve" @change="updateChart">
              平滑曲线
            </el-checkbox>
          </el-col>
          <el-col :span="6">
            <el-button size="small" @click="exportChart">
              导出图表
            </el-button>
          </el-col>
        </el-row>
      </div>
    </div>
    
    <div class="chart-container">
      <div :id="chartId" ref="chartContainer" class="plotly-chart"></div>
    </div>
    
    <!-- 数据统计面板 -->
    <div v-if="showStats" class="stats-panel">
      <el-card class="stats-card">
        <template #header>
          <div class="stats-header">
            <span>数据统计</span>
            <el-button type="text" size="small" @click="showStats = false">收起</el-button>
          </div>
        </template>
        
        <div class="stats-content">
          <el-row :gutter="12">
            <el-col :span="8">
              <div class="stat-item">
                <div class="stat-label">数据点数</div>
                <div class="stat-value">{{ dataStats.pointCount }}</div>
              </div>
            </el-col>
            <el-col :span="8">
              <div class="stat-item">
                <div class="stat-label">最大值</div>
                <div class="stat-value">{{ dataStats.maxValue.toFixed(2) }}</div>
              </div>
            </el-col>
            <el-col :span="8">
              <div class="stat-item">
                <div class="stat-label">最小值</div>
                <div class="stat-value">{{ dataStats.minValue.toFixed(2) }}</div>
              </div>
            </el-col>
          </el-row>
          <el-row :gutter="12" style="margin-top: 12px;">
            <el-col :span="8">
              <div class="stat-item">
                <div class="stat-label">平均值</div>
                <div class="stat-value">{{ dataStats.meanValue.toFixed(2) }}</div>
              </div>
            </el-col>
            <el-col :span="8">
              <div class="stat-item">
                <div class="stat-label">标准差</div>
                <div class="stat-value">{{ dataStats.stdValue.toFixed(2) }}</div>
              </div>
            </el-col>
            <el-col :span="8">
              <div class="stat-item">
                <div class="stat-label">信噪比</div>
                <div class="stat-value">{{ dataStats.snr.toFixed(1) }}</div>
              </div>
            </el-col>
          </el-row>
        </div>
      </el-card>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import Plotly from 'plotly.js-dist'

// 定义 props
const props = defineProps<{
  curve: {
    x_values: number[]
    y_values: number[]
    x_label?: string
    y_label?: string
    curve_type?: string
  }
  peaks?: Array<{
    center: number
    amplitude: number
    [key: string]: any
  }>
  title?: string
  showPeaks?: boolean
}>()

// 定义事件
const emit = defineEmits<{
  'chart-updated': []
  'peak-selected': [peak: any, index: number]
}>()

// 响应式数据
const chartContainer = ref<HTMLElement>()
const chartId = ref(`base-chart-${Math.random().toString(36).substr(2, 9)}`)
const displayMode = ref<'line' | 'scatter' | 'both'>('line')
const showGrid = ref(true)
const showLegend = ref(true)
const smoothCurve = ref(false)
const showStats = ref(false)

// 计算属性
const chartTitle = computed(() => {
  return props.title || `${props.curve.curve_type || 'DT'} 曲线`
})

const dataStats = computed(() => {
  if (!props.curve.y_values || props.curve.y_values.length === 0) {
    return {
      pointCount: 0,
      maxValue: 0,
      minValue: 0,
      meanValue: 0,
      stdValue: 0,
      snr: 0
    }
  }
  
  const values = props.curve.y_values
  const pointCount = values.length
  const maxValue = Math.max(...values)
  const minValue = Math.min(...values)
  const meanValue = values.reduce((sum, val) => sum + val, 0) / pointCount
  const variance = values.reduce((sum, val) => sum + Math.pow(val - meanValue, 2), 0) / pointCount
  const stdValue = Math.sqrt(variance)
  const snr = meanValue / stdValue
  
  return {
    pointCount,
    maxValue,
    minValue,
    meanValue,
    stdValue,
    snr
  }
})

// 监听器
watch(() => props.curve, () => {
  nextTick(() => {
    updateChart()
  })
}, { deep: true })

watch(() => props.peaks, () => {
  nextTick(() => {
    updateChart()
  })
}, { deep: true })

// 生命周期
onMounted(() => {
  initializeChart()
})

// 方法
function initializeChart() {
  if (!chartContainer.value) return
  
  const data = generateChartData()
  const layout = generateLayout()
  const config = generateConfig()
  
  Plotly.newPlot(chartContainer.value, data, layout, config)
  
  // 添加点击事件监听
  chartContainer.value.on('plotly_click', handleChartClick)
  
  // 添加双击事件监听（显示统计信息）
  chartContainer.value.on('plotly_doubleclick', () => {
    showStats.value = !showStats.value
  })
}

function generateChartData() {
  const data: any[] = []
  
  if (!props.curve.x_values || !props.curve.y_values) return data
  
  // 原始曲线数据
  const curveData: any = {
    x: props.curve.x_values,
    y: props.curve.y_values,
    type: 'scatter',
    name: '原始曲线',
    line: { 
      color: '#2c3e50', 
      width: 2 
    },
    hovertemplate: `
      <b>原始曲线</b><br>
      位置: %{x:.3f}<br>
      强度: %{y:.2f}<br>
      <extra></extra>
    `
  }
  
  // 根据显示模式设置
  switch (displayMode.value) {
    case 'line':
      curveData.mode = 'lines'
      break
    case 'scatter':
      curveData.mode = 'markers'
      curveData.marker = { size: 4, color: '#2c3e50' }
      break
    case 'both':
      curveData.mode = 'lines+markers'
      curveData.marker = { size: 3, color: '#2c3e50' }
      break
  }
  
  // 平滑曲线处理
  if (smoothCurve.value && displayMode.value !== 'scatter') {
    curveData.line.smoothing = 0.3
  }
  
  data.push(curveData)
  
  // 添加峰标记
  if (props.showPeaks && props.peaks && props.peaks.length > 0) {
    const peakX = props.peaks.map(peak => peak.center)
    const peakY = props.peaks.map(peak => peak.amplitude)
    
    data.push({
      x: peakX,
      y: peakY,
      type: 'scatter',
      mode: 'markers',
      name: '检测到的峰',
      marker: { 
        color: '#e74c3c', 
        size: 10,
        symbol: 'diamond',
        line: { color: '#fff', width: 2 }
      },
      hovertemplate: `
        <b>峰 %{pointNumber + 1}</b><br>
        位置: %{x:.3f}<br>
        强度: %{y:.2f}<br>
        <extra></extra>
      `
    })
  }
  
  return data
}

function generateLayout() {
  return {
    title: {
      text: chartTitle.value,
      font: { size: 16 }
    },
    xaxis: { 
      title: props.curve.x_label || 'X轴',
      showgrid: showGrid.value,
      gridcolor: '#f0f0f0'
    },
    yaxis: { 
      title: props.curve.y_label || 'Y轴',
      showgrid: showGrid.value,
      gridcolor: '#f0f0f0'
    },
    showlegend: showLegend.value,
    legend: {
      orientation: 'v',
      x: 1.02,
      y: 1
    },
    margin: { t: 80, r: 120, b: 80, l: 80 },
    plot_bgcolor: '#fff',
    paper_bgcolor: '#fff',
    hovermode: 'closest'
  }
}

function generateConfig() {
  return {
    responsive: true,
    displayModeBar: true,
    modeBarButtonsToRemove: ['pan2d', 'lasso2d', 'select2d'],
    displaylogo: false,
    toImageButtonOptions: {
      format: 'png',
      filename: 'base_chart',
      height: 600,
      width: 800,
      scale: 2
    }
  }
}

function updateChart() {
  if (!chartContainer.value) return
  
  const data = generateChartData()
  const layout = generateLayout()
  
  Plotly.react(chartContainer.value, data, layout)
  emit('chart-updated')
}

function setDisplayMode(mode: 'line' | 'scatter' | 'both') {
  displayMode.value = mode
  updateChart()
}

function handleChartClick(event: any) {
  if (event.points && event.points.length > 0) {
    const point = event.points[0]
    
    // 如果是峰数据点
    if (point.data.name && point.data.name.includes('峰')) {
      const peakIndex = point.pointNumber
      if (props.peaks && peakIndex < props.peaks.length) {
        emit('peak-selected', props.peaks[peakIndex], peakIndex)
      }
    }
  }
}

function exportChart() {
  if (!chartContainer.value) return
  
  Plotly.downloadImage(chartContainer.value, {
    format: 'png',
    filename: 'base_chart',
    height: 600,
    width: 800,
    scale: 2
  })
}
</script>

<style scoped>
.base-chart {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #fff;
  border-radius: 8px;
  overflow: hidden;
}

.chart-header {
  padding: 16px 20px;
  border-bottom: 1px solid #e4e7ed;
  background: #fafafa;
}

.chart-header h4 {
  margin: 0;
  color: #303133;
  font-size: 16px;
  font-weight: 500;
}

.chart-controls {
  margin-top: 12px;
}

.chart-container {
  flex: 1;
  padding: 20px;
  position: relative;
}

.plotly-chart {
  width: 100%;
  height: 100%;
  min-height: 400px;
}

.stats-panel {
  position: absolute;
  top: 20px;
  right: 20px;
  width: 250px;
  z-index: 1000;
}

.stats-card {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border-radius: 8px;
}

.stats-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: bold;
  color: #303133;
}

.stats-content {
  padding: 8px 0;
}

.stat-item {
  text-align: center;
  padding: 8px;
  background: #f8f9fa;
  border-radius: 6px;
  border: 1px solid #e9ecef;
}

.stat-label {
  font-size: 12px;
  color: #666;
  margin-bottom: 4px;
  font-weight: 500;
}

.stat-value {
  font-size: 16px;
  font-weight: bold;
  color: #333;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .chart-header {
    padding: 12px 16px;
  }
  
  .chart-container {
    padding: 16px;
  }
  
  .stats-panel {
    position: relative;
    top: auto;
    right: auto;
    width: 100%;
    margin-top: 16px;
  }
  
  .chart-controls .el-col {
    margin-bottom: 8px;
  }
}
</style>
