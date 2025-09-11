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
            <el-button 
              :type="plotMode === 'area' ? 'primary' : ''"
              size="small"
              @click="setPlotMode('area')"
            >
              面积填充
            </el-button>
          </el-button-group>
        </el-col>
      </el-row>
    </div>
    
    <div class="plot-container">
      <!-- 原始曲线图表 -->
      <BaseChart 
        v-if="plotMode === 'original' && container?.curves?.[0]"
        :curve="container.curves[0]"
        :title="`${container.curves[0].curve_type || 'DT'} 曲线`"
        @chart-updated="handleChartUpdated"
        @peak-selected="handlePeakSelected"
      />
      
      <!-- 峰检测图表 -->
      <BaseChart 
        v-else-if="plotMode === 'peaks' && container?.curves?.[0]"
        :curve="container.curves[0]"
        :peaks="container.peaks || []"
        :show-peaks="true"
        :title="`${container.curves[0].curve_type || 'DT'} 曲线 - 峰检测结果`"
        @chart-updated="handleChartUpdated"
        @peak-selected="handlePeakSelected"
      />
      
      <!-- 峰拟合图表 -->
      <PeakFittingChart 
        v-else-if="plotMode === 'fitted' && container?.curves?.[0]"
        :curve="container.curves[0]"
        :peaks="container.peaks || []"
        :title="`${container.curves[0].curve_type || 'DT'} 曲线 - 峰拟合结果`"
        @chart-updated="handleChartUpdated"
        @peak-selected="handlePeakSelected"
      />
      
      <!-- 峰面积填充图表 -->
      <PeakAreaChart 
        v-else-if="plotMode === 'area' && container?.curves?.[0]"
        :curve="container.curves[0]"
        :peaks="container.peaks || []"
        :title="`${container.curves[0].curve_type || 'DT'} 曲线 - 峰面积填充`"
        @chart-updated="handleChartUpdated"
        @peak-selected="handlePeakSelected"
      />
      
      <!-- 拟合质量统计面板 -->
      <div v-if="plotMode === 'fitted' && fittingQualityStats" class="fitting-quality-panel">
        <el-card class="quality-card">
          <template #header>
            <span>拟合质量统计</span>
          </template>
          <el-row :gutter="12">
            <el-col :span="6">
              <div class="quality-item">
                <div class="quality-label">平均R²</div>
                <div class="quality-value" :class="getQualityClass(fittingQualityStats.avgRSquared)">
                  {{ fittingQualityStats.avgRSquared.toFixed(3) }}
                </div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="quality-item">
                <div class="quality-label">平均迭代</div>
                <div class="quality-value">
                  {{ fittingQualityStats.avgIterations.toFixed(0) }}
                </div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="quality-item">
                <div class="quality-label">收敛率</div>
                <div class="quality-value" :class="getConvergenceClass(fittingQualityStats.convergenceRate)">
                  {{ (fittingQualityStats.convergenceRate * 100).toFixed(1) }}%
                </div>
              </div>
            </el-col>
            <el-col :span="6">
              <div class="quality-item">
                <div class="quality-label">峰数量</div>
                <div class="quality-value">
                  {{ fittingQualityStats.peakCount }}
                </div>
              </div>
            </el-col>
          </el-row>
        </el-card>
      </div>
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
import { ref, computed, watch } from 'vue'
import BaseChart from './BaseChart.vue'
import PeakFittingChart from './PeakFittingChart.vue'
import PeakAreaChart from './PeakAreaChart.vue'

// 定义props和emits
const props = defineProps<{
  container: any
  plotMode: string
  multiCurveData?: Array<{fileId: string, fileName: string, curves: any[]}>
  isComparing?: boolean
}>()

const emit = defineEmits(['request-data', 'plot-mode-changed', 'chart-updated', 'peak-selected'])

// 响应式数据
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


// 计算拟合质量统计
const fittingQualityStats = computed(() => {
  if (!props.container || !props.container.peaks || props.container.peaks.length === 0) {
    return null
  }
  
  const peaks = props.container.peaks
  const validPeaks = peaks.filter((peak: any) => peak.rsquared !== undefined)
  
  if (validPeaks.length === 0) {
    return null
  }
  
  const avgRSquared = validPeaks.reduce((sum: number, peak: any) => sum + peak.rsquared, 0) / validPeaks.length
  const avgIterations = validPeaks.reduce((sum: number, peak: any) => sum + (peak.iterations || 0), 0) / validPeaks.length
  const convergedCount = validPeaks.filter((peak: any) => peak.converged !== false).length
  const convergenceRate = convergedCount / validPeaks.length
  
  return {
    avgRSquared,
    avgIterations,
    convergenceRate,
    peakCount: peaks.length
  }
})

// 监听器
watch(() => props.plotMode, (newMode) => {
  plotMode.value = newMode
}, { immediate: true })

// 方法
function setPlotMode(mode: string) {
  plotMode.value = mode
  emit('plot-mode-changed', mode)
}

function handleChartUpdated() {
  emit('chart-updated')
}

function handlePeakSelected(peak: any, index: number) {
  emit('peak-selected', peak, index)
}

// 获取质量等级样式类
function getQualityClass(rSquared: number): string {
  if (rSquared >= 0.95) return 'quality-excellent'
  if (rSquared >= 0.90) return 'quality-good'
  if (rSquared >= 0.80) return 'quality-fair'
  return 'quality-poor'
}

// 获取收敛率样式类
function getConvergenceClass(rate: number): string {
  if (rate >= 0.95) return 'convergence-excellent'
  if (rate >= 0.80) return 'convergence-good'
  if (rate >= 0.60) return 'convergence-fair'
  return 'convergence-poor'
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

/* 拟合质量面板样式 */
.fitting-quality-panel {
  position: absolute;
  top: 20px;
  right: 20px;
  width: 300px;
  z-index: 1000;
}

.quality-card {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.15);
  border-radius: 8px;
}

.quality-item {
  text-align: center;
  padding: 8px;
}

.quality-label {
  font-size: 12px;
  color: #666;
  margin-bottom: 4px;
  font-weight: 500;
}

.quality-value {
  font-size: 16px;
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

/* 收敛率颜色 */
.convergence-excellent {
  color: #67c23a !important;
}

.convergence-good {
  color: #409eff !important;
}

.convergence-fair {
  color: #e6a23c !important;
}

.convergence-poor {
  color: #f56c6c !important;
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
  
  .fitting-quality-panel {
    position: relative;
    top: auto;
    right: auto;
    width: 100%;
    margin-top: 16px;
  }
}
</style>
