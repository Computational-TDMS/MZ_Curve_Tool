<template>
  <div class="plot-panel">
    <div class="plot-header">
      <el-row justify="space-between" align="middle">
        <el-col :span="12">
          <h3>{{ plotTitle }}</h3>
        </el-col>
        <el-col :span="12" style="text-align: right">
          <el-button-group>
            <el-button 
              :type="plotMode === 'original' ? 'primary' : ''"
              size="small"
              @click="setPlotMode('original')"
            >
              原始曲线
            </el-button>
            <el-button 
              :type="plotMode === 'peaks' ? 'primary' : ''"
              size="small"
              @click="setPlotMode('peaks')"
            >
              峰检测
            </el-button>
            <el-button 
              :type="plotMode === 'fitted' ? 'primary' : ''"
              size="small"
              @click="setPlotMode('fitted')"
            >
              峰拟合
            </el-button>
          </el-button-group>
        </el-col>
      </el-row>
    </div>
    
    <div class="plot-container">
      <div id="plotly-chart" ref="plotContainer"></div>
    </div>
    
    <!-- 空状态 -->
    <div v-if="!container || !container.curves || container.curves.length === 0" class="empty-state">
      <el-empty description="暂无数据">
        <el-button type="primary" @click="$emit('request-data')">开始分析</el-button>
      </el-empty>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch, onMounted, nextTick } from 'vue'
import Plotly from 'plotly.js-dist'

// 定义props和emits
const props = defineProps<{
  container: any
  plotMode: string
}>()

const emit = defineEmits(['request-data', 'plot-mode-changed'])

// 响应式数据
const plotContainer = ref<HTMLElement>()
const plotMode = ref(props.plotMode)

// 计算属性
const plotTitle = computed(() => {
  if (!props.container || !props.container.curves || props.container.curves.length === 0) {
    return 'MZ Curve 数据可视化'
  }
  
  const curve = props.container.curves[0]
  const mode = plotMode.value
  
  switch (mode) {
    case 'peaks':
      return `${curve.curve_type} 曲线 - 峰检测结果`
    case 'fitted':
      return `${curve.curve_type} 曲线 - 峰拟合结果`
    default:
      return `${curve.curve_type} 曲线`
  }
})

// 计算坐标轴标签
const axisLabels = computed(() => {
  if (!props.container || !props.container.curves || props.container.curves.length === 0) {
    return {
      x: 'Drift Time (ms)',
      y: 'Intensity'
    }
  }
  
  const curve = props.container.curves[0]
  const curveType = curve.curve_type?.toLowerCase() || 'dt'
  
  // 根据曲线类型设置坐标轴标签
  switch (curveType) {
    case 'dt':
      return {
        x: 'Drift Time (ms)',
        y: 'Intensity'
      }
    case 'tic':
      return {
        x: 'Retention Time (min)',
        y: 'Total Ion Current'
      }
    case 'xic':
      return {
        x: 'Retention Time (min)',
        y: 'Extracted Ion Current'
      }
    default:
      return {
        x: curve.x_label || 'X轴',
        y: curve.y_label || 'Y轴'
      }
  }
})

// 监听器
watch(() => props.container, () => {
  nextTick(() => {
    updatePlot()
  })
}, { deep: true })

watch(() => props.plotMode, (newMode) => {
  plotMode.value = newMode
  nextTick(() => {
    updatePlot()
  })
}, { immediate: true })

// 生命周期
onMounted(() => {
  initializePlot()
})

// 方法
function initializePlot() {
  if (!plotContainer.value) return
  
  const data = [{
    x: [],
    y: [],
    type: 'scatter',
    mode: 'lines',
    name: '原始曲线',
    line: { color: '#409EFF' }
  }]

  const layout = {
    title: 'MZ Curve 数据可视化',
    xaxis: { 
      title: axisLabels.value.x,
      showgrid: true,
      gridcolor: '#f0f0f0'
    },
    yaxis: { 
      title: axisLabels.value.y,
      showgrid: true,
      gridcolor: '#f0f0f0'
    },
    showlegend: true,
    margin: { t: 80, r: 50, b: 80, l: 80 },
    plot_bgcolor: '#fff',
    paper_bgcolor: '#fff'
  }

  const config = {
    responsive: true,
    displayModeBar: true,
    modeBarButtonsToRemove: ['pan2d', 'lasso2d', 'select2d'],
    displaylogo: false
  }

  Plotly.newPlot(plotContainer.value, data, layout, config)
}

function updatePlot() {
  if (!plotContainer.value || !props.container || !props.container.curves || props.container.curves.length === 0) {
    return
  }

  const curve = props.container.curves[0]
  const data = []

  // 原始曲线
  data.push({
    x: curve.x_values || [],
    y: curve.y_values || [],
    type: 'scatter',
    mode: 'lines',
    name: '原始曲线',
    line: { color: '#409EFF', width: 2 }
  })

  // 根据模式添加额外数据
  if (plotMode.value === 'peaks' && props.container.peaks && props.container.peaks.length > 0) {
    // 添加峰位置标记
    const peakX = props.container.peaks.map((peak: any) => peak.center)
    const peakY = props.container.peaks.map((peak: any) => peak.amplitude)
    
    data.push({
      x: peakX,
      y: peakY,
      type: 'scatter',
      mode: 'markers',
      name: '检测到的峰',
      marker: { 
        color: '#F56C6C', 
        size: 10,
        symbol: 'diamond',
        line: { color: '#fff', width: 2 }
      }
    })
  }

  if (plotMode.value === 'fitted' && props.container.peaks && props.container.peaks.length > 0) {
    // 添加拟合的峰
    props.container.peaks.forEach((peak: any, index: number) => {
      if (peak.fit_parameters && Object.keys(peak.fit_parameters).length > 0) {
        // 生成拟合曲线数据点
        const xRange = curve.x_values || []
        const fittedY = xRange.map((x: number) => {
          // 简化的高斯拟合显示
          const sigma = peak.fwhm / 2.355
          return peak.amplitude * Math.exp(-0.5 * Math.pow((x - peak.center) / sigma, 2))
        })

        data.push({
          x: xRange,
          y: fittedY,
          type: 'scatter',
          mode: 'lines',
          name: `峰 ${index + 1} 拟合`,
          line: { 
            color: `hsl(${index * 60}, 70%, 50%)`,
            dash: 'dash',
            width: 2
          }
        })
      }
    })
  }

  const layout = {
    title: plotTitle.value,
    xaxis: { 
      title: axisLabels.value.x,
      showgrid: true,
      gridcolor: '#f0f0f0'
    },
    yaxis: { 
      title: axisLabels.value.y,
      showgrid: true,
      gridcolor: '#f0f0f0'
    },
    showlegend: true,
    margin: { t: 80, r: 50, b: 80, l: 80 },
    plot_bgcolor: '#fff',
    paper_bgcolor: '#fff'
  }

  Plotly.newPlot(plotContainer.value, data, layout, { responsive: true })
}

function setPlotMode(mode: string) {
  plotMode.value = mode
  emit('plot-mode-changed', mode)
}
</script>

<style scoped>
.plot-panel {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: #fff;
}

.plot-header {
  padding: 16px 20px;
  border-bottom: 1px solid #e4e7ed;
  background: #fafafa;
}

.plot-header h3 {
  margin: 0;
  color: #303133;
  font-size: 16px;
  font-weight: 500;
}

.plot-container {
  flex: 1;
  padding: 20px;
  position: relative;
}

#plotly-chart {
  width: 100%;
  height: 100%;
  min-height: 400px;
}

.empty-state {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  background: #fafafa;
}

/* 响应式设计 */
@media (max-width: 768px) {
  .plot-header {
    padding: 12px 16px;
  }
  
  .plot-container {
    padding: 16px;
  }
  
  .plot-header h3 {
    font-size: 14px;
  }
}
</style>
