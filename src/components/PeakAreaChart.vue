<template>
  <div class="peak-area-chart">
    <div class="chart-header">
      <el-row justify="space-between" align="middle">
        <el-col :span="12">
          <h4>{{ chartTitle }}</h4>
        </el-col>
        <el-col :span="12" style="text-align: right">
          <el-button-group size="small">
            <el-button 
              :type="fillMode === 'transparent' ? 'primary' : ''"
              @click="setFillMode('transparent')"
            >
              透明填充
            </el-button>
            <el-button 
              :type="fillMode === 'gradient' ? 'primary' : ''"
              @click="setFillMode('gradient')"
            >
              渐变填充
            </el-button>
            <el-button 
              :type="fillMode === 'solid' ? 'primary' : ''"
              @click="setFillMode('solid')"
            >
              实心填充
            </el-button>
          </el-button-group>
        </el-col>
      </el-row>
      
      <!-- 填充控制选项 -->
      <div class="fill-controls">
        <el-row :gutter="12" align="middle">
          <el-col :span="6">
            <el-slider 
              v-model="fillOpacity" 
              :min="0.1" 
              :max="1.0" 
              :step="0.1"
              :format-tooltip="(val: number) => (val * 100).toFixed(0) + '%'"
              @change="updateChart"
            />
            <div class="control-label">填充透明度</div>
          </el-col>
          <el-col :span="6">
            <el-slider 
              v-model="borderWidth" 
              :min="0" 
              :max="5" 
              :step="0.5"
              @change="updateChart"
            />
            <div class="control-label">边框宽度</div>
          </el-col>
          <el-col :span="6">
            <el-checkbox v-model="showPeakLabels" @change="updateChart">
              显示峰标签
            </el-checkbox>
          </el-col>
          <el-col :span="6">
            <el-checkbox v-model="showPeakAreas" @change="updateChart">
              显示峰面积
            </el-checkbox>
          </el-col>
        </el-row>
      </div>
    </div>
    
    <div class="chart-container">
      <div :id="chartId" ref="chartContainer" class="plotly-chart"></div>
    </div>
    
    <!-- 峰面积统计面板 -->
    <div v-if="showAreaStats" class="area-stats-panel">
      <el-card class="stats-card">
        <template #header>
          <div class="stats-header">
            <span>峰面积统计</span>
            <el-button type="text" size="small" @click="showAreaStats = false">收起</el-button>
          </div>
        </template>
        
        <div class="area-stats-content">
          <div class="total-area">
            <span class="label">总面积:</span>
            <span class="value">{{ totalArea.toFixed(2) }}</span>
          </div>
          
          <el-divider />
          
          <div class="peak-areas">
            <div 
              v-for="(peak, index) in peaks" 
              :key="index"
              class="peak-area-item"
            >
              <div class="peak-info">
                <span class="peak-name">峰 {{ index + 1 }}</span>
                <span class="peak-area">{{ peak.area.toFixed(2) }}</span>
                <span class="peak-percentage">{{ getPeakPercentage(peak.area).toFixed(1) }}%</span>
              </div>
              <div class="area-bar">
                <div 
                  class="area-fill" 
                  :style="{ 
                    width: getPeakPercentage(peak.area) + '%',
                    backgroundColor: getPeakColor(index)
                  }"
                ></div>
              </div>
            </div>
          </div>
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
  peaks: Array<{
    center: number
    amplitude: number
    area: number
    fwhm: number
    rsquared?: number
    peakType?: string
    [key: string]: any
  }>
  title?: string
}>()

// 定义事件
const emit = defineEmits<{
  'chart-updated': []
  'peak-selected': [peak: any, index: number]
}>()

// 响应式数据
const chartContainer = ref<HTMLElement>()
const chartId = ref(`peak-area-chart-${Math.random().toString(36).substr(2, 9)}`)
const fillMode = ref<'transparent' | 'gradient' | 'solid'>('transparent')
const fillOpacity = ref(0.6)
const borderWidth = ref(1.5)
const showPeakLabels = ref(true)
const showPeakAreas = ref(true)
const showAreaStats = ref(false)

// 计算属性
const chartTitle = computed(() => {
  return props.title || `${props.curve.curve_type || 'DT'} 曲线 - 峰面积填充`
})

const totalArea = computed(() => {
  return props.peaks.reduce((sum, peak) => sum + peak.area, 0)
})

// Seaborn风格的颜色调色板
const seabornColors = [
  '#1f77b4', '#ff7f0e', '#2ca02c', '#d62728', '#9467bd',
  '#8c564b', '#e377c2', '#7f7f7f', '#bcbd22', '#17becf',
  '#aec7e8', '#ffbb78', '#98df8a', '#ff9896', '#c5b0d5',
  '#c49c94', '#f7b6d3', '#c7c7c7', '#dbdb8d', '#9edae5'
]

// 监听器
watch(() => props.peaks, () => {
  nextTick(() => {
    updateChart()
  })
}, { deep: true })

watch(() => props.curve, () => {
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
  
  // 添加双击事件监听（显示面积统计）
  chartContainer.value.on('plotly_doubleclick', () => {
    showAreaStats.value = !showAreaStats.value
  })
}

function generateChartData() {
  const data: any[] = []
  
  if (!props.curve.x_values || !props.curve.y_values) return data
  
  // 原始曲线
  data.push({
    x: props.curve.x_values,
    y: props.curve.y_values,
    type: 'scatter',
    mode: 'lines',
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
  })
  
  // 峰面积填充
  if (showPeakAreas.value) {
    props.peaks.forEach((peak, index) => {
      const peakData = generatePeakAreaData(peak, index)
      if (peakData) {
        data.push(peakData)
      }
    })
  }
  
  // 峰中心标记
  data.push({
    x: props.peaks.map(peak => peak.center),
    y: props.peaks.map(peak => peak.amplitude),
    type: 'scatter',
    mode: 'markers',
    name: '峰中心',
    marker: {
      color: props.peaks.map((_, index) => getPeakColor(index)),
      size: 12,
      symbol: 'diamond',
      line: { color: '#fff', width: 2 }
    },
    hovertemplate: `
      <b>峰 %{pointNumber + 1}</b><br>
      中心: %{x:.3f}<br>
      强度: %{y:.2f}<br>
      面积: ${props.peaks[0]?.area.toFixed(2) || 'N/A'}<br>
      <extra></extra>
    `
  })
  
  // 峰标签
  if (showPeakLabels.value) {
    props.peaks.forEach((peak, index) => {
      data.push({
        x: [peak.center],
        y: [peak.amplitude * 1.1], // 稍微向上偏移
        type: 'scatter',
        mode: 'text',
        text: [`峰${index + 1}`],
        textposition: 'top center',
        textfont: {
          color: getPeakColor(index),
          size: 12,
          family: 'Arial, sans-serif'
        },
        showlegend: false,
        hovertemplate: `
          <b>峰 ${index + 1}</b><br>
          中心: ${peak.center.toFixed(3)}<br>
          强度: ${peak.amplitude.toFixed(2)}<br>
          面积: ${peak.area.toFixed(2)}<br>
          <extra></extra>
        `
      })
    })
  }
  
  return data
}

function generatePeakAreaData(peak: any, index: number) {
  if (!props.curve.x_values || props.curve.x_values.length === 0) return null
  
  const xRange = props.curve.x_values
  const sigma = peak.fwhm / 2.355
  
  // 生成峰拟合数据
  const fittedY = xRange.map(x => {
    return peak.amplitude * Math.exp(-0.5 * Math.pow((x - peak.center) / sigma, 2))
  })
  
  // 创建基线（y=0）
  const baseline = new Array(xRange.length).fill(0)
  
  const peakColor = getPeakColor(index)
  const fillColor = getFillColor(peakColor)
  
  return {
    x: xRange,
    y: fittedY,
    type: 'scatter',
    mode: 'lines',
    fill: 'tonexty',
    fillcolor: fillColor,
    line: {
      color: peakColor,
      width: borderWidth.value
    },
    name: `峰 ${index + 1} 面积`,
    showlegend: false,
    hovertemplate: `
      <b>峰 ${index + 1}</b><br>
      位置: %{x:.3f}<br>
      强度: %{y:.2f}<br>
      面积: ${peak.area.toFixed(2)}<br>
      R²: ${peak.rsquared?.toFixed(4) || 'N/A'}<br>
      <extra></extra>
    `
  }
}

function getPeakColor(index: number): string {
  return seabornColors[index % seabornColors.length]
}

function getFillColor(baseColor: string): string {
  switch (fillMode.value) {
    case 'transparent':
      // 转换为RGBA格式，设置透明度
      const rgb = hexToRgb(baseColor)
      return `rgba(${rgb.r}, ${rgb.g}, ${rgb.b}, ${fillOpacity.value})`
    
    case 'gradient':
      // 创建渐变效果（简化版本）
      const rgb2 = hexToRgb(baseColor)
      return `rgba(${rgb2.r}, ${rgb2.g}, ${rgb2.b}, ${fillOpacity.value * 0.5})`
    
    case 'solid':
      return baseColor
    
    default:
      return baseColor
  }
}

function hexToRgb(hex: string): { r: number, g: number, b: number } {
  const result = /^#?([a-f\d]{2})([a-f\d]{2})([a-f\d]{2})$/i.exec(hex)
  return result ? {
    r: parseInt(result[1], 16),
    g: parseInt(result[2], 16),
    b: parseInt(result[3], 16)
  } : { r: 0, g: 0, b: 0 }
}

function getPeakPercentage(peakArea: number): number {
  if (totalArea.value === 0) return 0
  return (peakArea / totalArea.value) * 100
}

function generateLayout() {
  return {
    title: {
      text: chartTitle.value,
      font: { size: 16 }
    },
    xaxis: { 
      title: props.curve.x_label || 'X轴',
      showgrid: true,
      gridcolor: '#f0f0f0'
    },
    yaxis: { 
      title: props.curve.y_label || 'Y轴',
      showgrid: true,
      gridcolor: '#f0f0f0',
      zeroline: true,
      zerolinecolor: '#ccc'
    },
    showlegend: true,
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
      filename: 'peak_area_chart',
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

function setFillMode(mode: 'transparent' | 'gradient' | 'solid') {
  fillMode.value = mode
  updateChart()
}

function handleChartClick(event: any) {
  if (event.points && event.points.length > 0) {
    const point = event.points[0]
    
    // 如果是峰数据点
    if (point.data.name && point.data.name.includes('峰')) {
      const peakIndex = parseInt(point.data.name.match(/\d+/)?.[0]) - 1
      if (peakIndex >= 0 && peakIndex < props.peaks.length) {
        emit('peak-selected', props.peaks[peakIndex], peakIndex)
      }
    }
  }
}
</script>

<style scoped>
.peak-area-chart {
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

.fill-controls {
  margin-top: 12px;
}

.control-label {
  font-size: 12px;
  color: #666;
  text-align: center;
  margin-top: 4px;
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

.area-stats-panel {
  position: absolute;
  top: 20px;
  right: 20px;
  width: 280px;
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

.area-stats-content {
  padding: 8px 0;
}

.total-area {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: #f8f9fa;
  border-radius: 6px;
  margin-bottom: 12px;
}

.label {
  font-size: 14px;
  color: #666;
  font-weight: 500;
}

.value {
  font-size: 16px;
  font-weight: bold;
  color: #333;
}

.peak-areas {
  space-y: 8px;
}

.peak-area-item {
  margin-bottom: 12px;
}

.peak-info {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 4px;
}

.peak-name {
  font-size: 12px;
  color: #666;
  font-weight: 500;
}

.peak-area {
  font-size: 12px;
  color: #333;
  font-weight: bold;
}

.peak-percentage {
  font-size: 12px;
  color: #409eff;
  font-weight: bold;
}

.area-bar {
  height: 8px;
  background: #f0f0f0;
  border-radius: 4px;
  overflow: hidden;
}

.area-fill {
  height: 100%;
  border-radius: 4px;
  transition: width 0.3s ease;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .chart-header {
    padding: 12px 16px;
  }
  
  .chart-container {
    padding: 16px;
  }
  
  .area-stats-panel {
    position: relative;
    top: auto;
    right: auto;
    width: 100%;
    margin-top: 16px;
  }
  
  .fill-controls .el-col {
    margin-bottom: 12px;
  }
}
</style>
