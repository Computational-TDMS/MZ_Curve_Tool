<template>
  <div class="peak-fitting-chart">
    <div class="chart-header">
      <el-row justify="space-between" align="middle">
        <el-col :span="12">
          <h4>{{ chartTitle }}</h4>
        </el-col>
        <el-col :span="12" style="text-align: right">
          <el-button-group size="small">
            <el-button 
              :type="visualizationMode === 'overlay' ? 'primary' : ''"
              @click="setVisualizationMode('overlay')"
            >
              叠加模式
            </el-button>
            <el-button 
              :type="visualizationMode === 'separate' ? 'primary' : ''"
              @click="setVisualizationMode('separate')"
            >
              分离模式
            </el-button>
            <el-button 
              :type="visualizationMode === 'residual' ? 'primary' : ''"
              @click="setVisualizationMode('residual')"
            >
              残差模式
            </el-button>
          </el-button-group>
        </el-col>
      </el-row>
      
      <!-- 图表控制选项 -->
      <div class="chart-controls">
        <el-row :gutter="12" align="middle">
          <el-col :span="6">
            <el-checkbox v-model="showOriginalCurve" @change="updateChart">
              显示原始曲线
            </el-checkbox>
          </el-col>
          <el-col :span="6">
            <el-checkbox v-model="showPeakCenters" @change="updateChart">
              显示峰中心
            </el-checkbox>
          </el-col>
          <el-col :span="6">
            <el-checkbox v-model="showFittedCurves" @change="updateChart">
              显示拟合曲线
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
    
    <!-- 峰信息面板 -->
    <div v-if="selectedPeak" class="peak-info-panel">
      <el-card class="info-card">
        <template #header>
          <div class="info-header">
            <span>峰 {{ selectedPeakIndex + 1 }} 详细信息</span>
            <el-button type="text" size="small" @click="selectedPeak = null">关闭</el-button>
          </div>
        </template>
        
        <div class="peak-details">
          <el-row :gutter="12">
            <el-col :span="12">
              <div class="detail-item">
                <span class="label">中心位置:</span>
                <span class="value">{{ selectedPeak.center.toFixed(3) }}</span>
              </div>
              <div class="detail-item">
                <span class="label">峰高:</span>
                <span class="value">{{ selectedPeak.amplitude.toFixed(2) }}</span>
              </div>
              <div class="detail-item">
                <span class="label">峰面积:</span>
                <span class="value">{{ selectedPeak.area.toFixed(2) }}</span>
              </div>
              <div class="detail-item">
                <span class="label">FWHM:</span>
                <span class="value">{{ selectedPeak.fwhm.toFixed(3) }}</span>
              </div>
            </el-col>
            <el-col :span="12">
              <div class="detail-item">
                <span class="label">R²:</span>
                <span class="value" :class="getQualityClass(selectedPeak.rsquared)">
                  {{ selectedPeak.rsquared?.toFixed(4) || 'N/A' }}
                </span>
              </div>
              <div class="detail-item">
                <span class="label">迭代次数:</span>
                <span class="value">{{ selectedPeak.iterations || 'N/A' }}</span>
              </div>
              <div class="detail-item">
                <span class="label">收敛状态:</span>
                <span class="value" :class="selectedPeak.converged ? 'converged' : 'not-converged'">
                  {{ selectedPeak.converged ? '已收敛' : '未收敛' }}
                </span>
              </div>
              <div class="detail-item">
                <span class="label">峰类型:</span>
                <span class="value">{{ selectedPeak.peakType || '高斯' }}</span>
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
  peaks: Array<{
    center: number
    amplitude: number
    area: number
    fwhm: number
    rsquared?: number
    iterations?: number
    converged?: boolean
    peakType?: string
    fitParameters?: number[]
    [key: string]: any
  }>
  title?: string
}>()

// 定义事件
const emit = defineEmits<{
  'peak-selected': [peak: any, index: number]
  'chart-updated': []
}>()

// 响应式数据
const chartContainer = ref<HTMLElement>()
const chartId = ref(`peak-fitting-chart-${Math.random().toString(36).substr(2, 9)}`)
const visualizationMode = ref<'overlay' | 'separate' | 'residual'>('overlay')
const showOriginalCurve = ref(true)
const showPeakCenters = ref(true)
const showFittedCurves = ref(true)
const showPeakAreas = ref(true)
const selectedPeak = ref<any>(null)
const selectedPeakIndex = ref(0)

// 计算属性
const chartTitle = computed(() => {
  return props.title || `${props.curve.curve_type || 'DT'} 曲线 - 峰拟合结果`
})

// 峰颜色生成器
const peakColors = computed(() => {
  const colors = [
    'rgba(31, 119, 180, 0.7)',   // 蓝色
    'rgba(255, 127, 14, 0.7)',   // 橙色
    'rgba(44, 160, 44, 0.7)',    // 绿色
    'rgba(214, 39, 40, 0.7)',    // 红色
    'rgba(148, 103, 189, 0.7)',  // 紫色
    'rgba(140, 86, 75, 0.7)',    // 棕色
    'rgba(227, 119, 194, 0.7)',  // 粉色
    'rgba(127, 127, 127, 0.7)',  // 灰色
    'rgba(188, 189, 34, 0.7)',   // 橄榄色
    'rgba(23, 190, 207, 0.7)'    // 青色
  ]
  return colors
})

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
}

function generateChartData() {
  const data: any[] = []
  
  // 原始曲线
  if (showOriginalCurve.value) {
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
  }
  
  // 根据可视化模式生成不同的数据
  switch (visualizationMode.value) {
    case 'overlay':
      generateOverlayData(data)
      break
    case 'separate':
      generateSeparateData(data)
      break
    case 'residual':
      generateResidualData(data)
      break
  }
  
  return data
}

function generateOverlayData(data: any[]) {
  // 峰面积填充
  if (showPeakAreas.value) {
    props.peaks.forEach((peak, index) => {
      const peakData = generatePeakData(peak, index)
      if (peakData) {
        // 添加面积填充
        data.push({
          x: peakData.x,
          y: peakData.y,
          type: 'scatter',
          mode: 'lines',
          fill: 'tozeroy',
          name: `峰 ${index + 1} 面积`,
          line: { 
            color: peakColors.value[index % peakColors.value.length],
            width: 0
          },
          fillcolor: peakColors.value[index % peakColors.value.length],
          showlegend: false,
          hovertemplate: `
            <b>峰 ${index + 1}</b><br>
            位置: %{x:.3f}<br>
            强度: %{y:.2f}<br>
            R²: ${peak.rsquared?.toFixed(4) || 'N/A'}<br>
            <extra></extra>
          `
        })
      }
    })
  }
  
  // 拟合曲线
  if (showFittedCurves.value) {
    props.peaks.forEach((peak, index) => {
      const peakData = generatePeakData(peak, index)
      if (peakData) {
        data.push({
          x: peakData.x,
          y: peakData.y,
          type: 'scatter',
          mode: 'lines',
          name: `峰 ${index + 1} 拟合`,
          line: { 
            color: peakColors.value[index % peakColors.value.length].replace('0.7', '1.0'),
            width: 2,
            dash: 'solid'
          },
          hovertemplate: `
            <b>峰 ${index + 1} 拟合</b><br>
            位置: %{x:.3f}<br>
            强度: %{y:.2f}<br>
            R²: ${peak.rsquared?.toFixed(4) || 'N/A'}<br>
            迭代: ${peak.iterations || 'N/A'}<br>
            <extra></extra>
          `
        })
      }
    })
  }
  
  // 峰中心标记
  if (showPeakCenters.value) {
    const peakCenters = props.peaks.map((peak, index) => ({
      x: peak.center,
      y: peak.amplitude,
      peak,
      index
    }))
    
    data.push({
      x: peakCenters.map(p => p.x),
      y: peakCenters.map(p => p.y),
      type: 'scatter',
      mode: 'markers',
      name: '峰中心',
      marker: {
        color: peakCenters.map((_, index) => peakColors.value[index % peakColors.value.length].replace('0.7', '1.0')),
        size: 12,
        symbol: 'diamond',
        line: { color: '#fff', width: 2 }
      },
      hovertemplate: `
        <b>峰 %{pointNumber + 1}</b><br>
        中心: %{x:.3f}<br>
        强度: %{y:.2f}<br>
        R²: ${peakCenters[0]?.peak.rsquared?.toFixed(4) || 'N/A'}<br>
        <extra></extra>
      `
    })
  }
}

function generateSeparateData(data: any[]) {
  // 在分离模式下，每个峰显示在单独的子图中
  // 这里简化处理，显示所有峰但用不同颜色区分
  generateOverlayData(data)
}

function generateResidualData(data: any[]) {
  // 残差模式：显示拟合残差
  if (props.curve.x_values && props.curve.y_values) {
    const residuals = calculateResiduals()
    data.push({
      x: props.curve.x_values,
      y: residuals,
      type: 'scatter',
      mode: 'lines+markers',
      name: '拟合残差',
      line: { color: '#e74c3c', width: 1 },
      marker: { size: 3, color: '#e74c3c' },
      hovertemplate: `
        <b>拟合残差</b><br>
        位置: %{x:.3f}<br>
        残差: %{y:.4f}<br>
        <extra></extra>
      `
    })
  }
}

function generatePeakData(peak: any, index: number) {
  if (!props.curve.x_values || props.curve.x_values.length === 0) return null
  
  const xRange = props.curve.x_values
  const sigma = peak.fwhm / 2.355
  const fittedY = xRange.map(x => {
    // 高斯拟合
    return peak.amplitude * Math.exp(-0.5 * Math.pow((x - peak.center) / sigma, 2))
  })
  
  return {
    x: xRange,
    y: fittedY
  }
}

function calculateResiduals(): number[] {
  if (!props.curve.x_values || !props.curve.y_values) return []
  
  const residuals: number[] = []
  const xValues = props.curve.x_values
  const yValues = props.curve.y_values
  
  // 计算所有峰的拟合总和
  const fittedSum = xValues.map(x => {
    return props.peaks.reduce((sum, peak) => {
      const sigma = peak.fwhm / 2.355
      return sum + peak.amplitude * Math.exp(-0.5 * Math.pow((x - peak.center) / sigma, 2))
    }, 0)
  })
  
  // 计算残差
  for (let i = 0; i < xValues.length; i++) {
    residuals.push(yValues[i] - fittedSum[i])
  }
  
  return residuals
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
      gridcolor: '#f0f0f0'
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
      filename: 'peak_fitting_chart',
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

function setVisualizationMode(mode: 'overlay' | 'separate' | 'residual') {
  visualizationMode.value = mode
  updateChart()
}

function handleChartClick(event: any) {
  if (event.points && event.points.length > 0) {
    const point = event.points[0]
    const pointNumber = point.pointNumber
    
    // 查找对应的峰
    if (point.data.name && point.data.name.includes('峰')) {
      const peakIndex = parseInt(point.data.name.match(/\d+/)?.[0]) - 1
      if (peakIndex >= 0 && peakIndex < props.peaks.length) {
        selectedPeak.value = props.peaks[peakIndex]
        selectedPeakIndex.value = peakIndex
        emit('peak-selected', props.peaks[peakIndex], peakIndex)
      }
    }
  }
}

function getQualityClass(rSquared?: number): string {
  if (!rSquared) return ''
  if (rSquared >= 0.95) return 'quality-excellent'
  if (rSquared >= 0.90) return 'quality-good'
  if (rSquared >= 0.80) return 'quality-fair'
  return 'quality-poor'
}
</script>

<style scoped>
.peak-fitting-chart {
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

.peak-info-panel {
  position: absolute;
  top: 20px;
  right: 20px;
  width: 300px;
  z-index: 1000;
}

.info-card {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border-radius: 8px;
}

.info-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  font-weight: bold;
  color: #303133;
}

.peak-details {
  padding: 8px 0;
}

.detail-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 4px 0;
  border-bottom: 1px solid #f0f0f0;
}

.detail-item:last-child {
  border-bottom: none;
}

.label {
  font-size: 12px;
  color: #666;
  font-weight: 500;
}

.value {
  font-size: 13px;
  font-weight: bold;
  color: #333;
}

/* 质量等级颜色 */
.quality-excellent {
  color: #67c23a !important;
}

.quality-good {
  color: #409eff !important;
}

.quality-fair {
  color: #e6a23c !important;
}

.quality-poor {
  color: #f56c6c !important;
}

.converged {
  color: #67c23a !important;
}

.not-converged {
  color: #f56c6c !important;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .chart-header {
    padding: 12px 16px;
  }
  
  .chart-container {
    padding: 16px;
  }
  
  .peak-info-panel {
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
