<template>
  <div class="fitting-quality-panel">
    <el-card class="panel-card">
      <template #header>
        <div class="card-header">
          <span>峰拟合质量评估</span>
          <el-button 
            v-if="showDetails" 
            type="text" 
            size="small" 
            @click="showDetails = false"
          >
            收起
          </el-button>
          <el-button 
            v-else 
            type="text" 
            size="small" 
            @click="showDetails = true"
          >
            详情
          </el-button>
        </div>
      </template>

      <!-- 质量概览 -->
      <div class="quality-overview">
        <el-row :gutter="12">
          <el-col :span="6">
            <div class="quality-metric">
              <div class="metric-icon" :class="getOverallQualityClass()">
                <el-icon><TrendCharts /></el-icon>
              </div>
              <div class="metric-content">
                <div class="metric-label">整体质量</div>
                <div class="metric-value" :class="getOverallQualityClass()">
                  {{ getOverallQualityText() }}
                </div>
              </div>
            </div>
          </el-col>
          <el-col :span="6">
            <div class="quality-metric">
              <div class="metric-icon">
                <el-icon><DataAnalysis /></el-icon>
              </div>
              <div class="metric-content">
                <div class="metric-label">平均R²</div>
                <div class="metric-value" :class="getQualityClass(stats.avgRSquared)">
                  {{ stats.avgRSquared.toFixed(3) }}
                </div>
              </div>
            </div>
          </el-col>
          <el-col :span="6">
            <div class="quality-metric">
              <div class="metric-icon">
                <el-icon><Refresh /></el-icon>
              </div>
              <div class="metric-content">
                <div class="metric-label">平均迭代</div>
                <div class="metric-value">
                  {{ stats.avgIterations.toFixed(0) }}
                </div>
              </div>
            </div>
          </el-col>
          <el-col :span="6">
            <div class="quality-metric">
              <div class="metric-icon">
                <el-icon><Check /></el-icon>
              </div>
              <div class="metric-content">
                <div class="metric-label">收敛率</div>
                <div class="metric-value" :class="getConvergenceClass(stats.convergenceRate)">
                  {{ (stats.convergenceRate * 100).toFixed(1) }}%
                </div>
              </div>
            </div>
          </el-col>
        </el-row>
      </div>

      <!-- 详细统计 -->
      <div v-if="showDetails" class="quality-details">
        <el-divider content-position="left">详细统计</el-divider>
        
        <el-row :gutter="16">
          <el-col :span="12">
            <div class="detail-section">
              <h4>拟合质量分布</h4>
              <div class="quality-distribution">
                <div class="distribution-item">
                  <span class="dist-label">优秀 (R² ≥ 0.95)</span>
                  <el-progress 
                    :percentage="(qualityDistribution.excellent / stats.peakCount * 100)" 
                    :stroke-width="6"
                    :show-text="false"
                    color="#67c23a"
                  />
                  <span class="dist-count">{{ qualityDistribution.excellent }}</span>
                </div>
                <div class="distribution-item">
                  <span class="dist-label">良好 (R² ≥ 0.90)</span>
                  <el-progress 
                    :percentage="(qualityDistribution.good / stats.peakCount * 100)" 
                    :stroke-width="6"
                    :show-text="false"
                    color="#409eff"
                  />
                  <span class="dist-count">{{ qualityDistribution.good }}</span>
                </div>
                <div class="distribution-item">
                  <span class="dist-label">一般 (R² ≥ 0.80)</span>
                  <el-progress 
                    :percentage="(qualityDistribution.fair / stats.peakCount * 100)" 
                    :stroke-width="6"
                    :show-text="false"
                    color="#e6a23c"
                  />
                  <span class="dist-count">{{ qualityDistribution.fair }}</span>
                </div>
                <div class="distribution-item">
                  <span class="dist-label">较差 (R² < 0.80)</span>
                  <el-progress 
                    :percentage="(qualityDistribution.poor / stats.peakCount * 100)" 
                    :stroke-width="6"
                    :show-text="false"
                    color="#f56c6c"
                  />
                  <span class="dist-count">{{ qualityDistribution.poor }}</span>
                </div>
              </div>
            </div>
          </el-col>
          
          <el-col :span="12">
            <div class="detail-section">
              <h4>优化建议</h4>
              <div class="suggestions">
                <div 
                  v-for="suggestion in optimizationSuggestions" 
                  :key="suggestion.type"
                  class="suggestion-item"
                  :class="suggestion.priority"
                >
                  <el-icon>
                    <Warning v-if="suggestion.priority === 'high'" />
                    <InfoFilled v-else-if="suggestion.priority === 'medium'" />
                    <SuccessFilled v-else />
                  </el-icon>
                  <span>{{ suggestion.message }}</span>
                </div>
              </div>
            </div>
          </el-col>
        </el-row>
      </div>

      <!-- 操作按钮 -->
      <div class="action-buttons">
        <el-button 
          type="primary" 
          size="small" 
          @click="exportQualityReport"
          :icon="Download"
        >
          导出质量报告
        </el-button>
        <el-button 
          size="small" 
          @click="optimizeParameters"
          :icon="Setting"
        >
          优化参数
        </el-button>
      </div>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { ElMessage } from 'element-plus'
import { 
  TrendCharts, 
  DataAnalysis, 
  Refresh, 
  Check, 
  Warning, 
  InfoFilled, 
  SuccessFilled,
  Download,
  Setting
} from '@element-plus/icons-vue'

// 定义 props
const props = defineProps<{
  peaks: Array<{
    rsquared?: number
    iterations?: number
    converged?: boolean
    [key: string]: any
  }>
}>()

// 定义事件
const emit = defineEmits<{
  'export-report': []
  'optimize-parameters': []
}>()

// 响应式数据
const showDetails = ref(false)

// 计算统计信息
const stats = computed(() => {
  if (!props.peaks || props.peaks.length === 0) {
    return {
      avgRSquared: 0,
      avgIterations: 0,
      convergenceRate: 0,
      peakCount: 0
    }
  }
  
  const validPeaks = props.peaks.filter(peak => peak.rsquared !== undefined)
  
  if (validPeaks.length === 0) {
    return {
      avgRSquared: 0,
      avgIterations: 0,
      convergenceRate: 0,
      peakCount: props.peaks.length
    }
  }
  
  const avgRSquared = validPeaks.reduce((sum, peak) => sum + peak.rsquared, 0) / validPeaks.length
  const avgIterations = validPeaks.reduce((sum, peak) => sum + (peak.iterations || 0), 0) / validPeaks.length
  const convergedCount = validPeaks.filter(peak => peak.converged !== false).length
  const convergenceRate = convergedCount / validPeaks.length
  
  return {
    avgRSquared,
    avgIterations,
    convergenceRate,
    peakCount: props.peaks.length
  }
})

// 计算质量分布
const qualityDistribution = computed(() => {
  if (!props.peaks) return { excellent: 0, good: 0, fair: 0, poor: 0 }
  
  const validPeaks = props.peaks.filter(peak => peak.rsquared !== undefined)
  
  return {
    excellent: validPeaks.filter(peak => peak.rsquared >= 0.95).length,
    good: validPeaks.filter(peak => peak.rsquared >= 0.90 && peak.rsquared < 0.95).length,
    fair: validPeaks.filter(peak => peak.rsquared >= 0.80 && peak.rsquared < 0.90).length,
    poor: validPeaks.filter(peak => peak.rsquared < 0.80).length
  }
})

// 计算优化建议
const optimizationSuggestions = computed(() => {
  const suggestions = []
  
  if (stats.value.avgRSquared < 0.80) {
    suggestions.push({
      type: 'quality',
      priority: 'high',
      message: '平均拟合质量较低，建议调整优化算法或增加迭代次数'
    })
  }
  
  if (stats.value.convergenceRate < 0.80) {
    suggestions.push({
      type: 'convergence',
      priority: 'high',
      message: '收敛率较低，建议降低收敛阈值或调整学习率'
    })
  }
  
  if (stats.value.avgIterations > 80) {
    suggestions.push({
      type: 'iterations',
      priority: 'medium',
      message: '平均迭代次数较多，建议优化初始参数估计'
    })
  }
  
  if (qualityDistribution.value.poor > stats.value.peakCount * 0.3) {
    suggestions.push({
      type: 'distribution',
      priority: 'medium',
      message: '较多峰拟合质量较差，建议检查数据质量或调整检测参数'
    })
  }
  
  if (suggestions.length === 0) {
    suggestions.push({
      type: 'good',
      priority: 'low',
      message: '拟合质量良好，当前参数设置合适'
    })
  }
  
  return suggestions
})

// 方法
function getQualityClass(rSquared: number): string {
  if (rSquared >= 0.95) return 'quality-excellent'
  if (rSquared >= 0.90) return 'quality-good'
  if (rSquared >= 0.80) return 'quality-fair'
  return 'quality-poor'
}

function getConvergenceClass(rate: number): string {
  if (rate >= 0.95) return 'convergence-excellent'
  if (rate >= 0.80) return 'convergence-good'
  if (rate >= 0.60) return 'convergence-fair'
  return 'convergence-poor'
}

function getOverallQualityClass(): string {
  const score = (stats.value.avgRSquared + stats.value.convergenceRate) / 2
  if (score >= 0.95) return 'quality-excellent'
  if (score >= 0.90) return 'quality-good'
  if (score >= 0.80) return 'quality-fair'
  return 'quality-poor'
}

function getOverallQualityText(): string {
  const score = (stats.value.avgRSquared + stats.value.convergenceRate) / 2
  if (score >= 0.95) return '优秀'
  if (score >= 0.90) return '良好'
  if (score >= 0.80) return '一般'
  return '较差'
}

function exportQualityReport() {
  emit('export-report')
  ElMessage.success('质量报告导出功能开发中...')
}

function optimizeParameters() {
  emit('optimize-parameters')
  ElMessage.info('参数优化建议已生成')
}
</script>

<style scoped>
.fitting-quality-panel {
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

.quality-overview {
  margin-bottom: 16px;
}

.quality-metric {
  display: flex;
  align-items: center;
  padding: 12px;
  background: #f8f9fa;
  border-radius: 8px;
  border: 1px solid #e9ecef;
}

.metric-icon {
  width: 40px;
  height: 40px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  margin-right: 12px;
  font-size: 18px;
  color: #fff;
}

.metric-icon.quality-excellent {
  background: #67c23a;
}

.metric-icon.quality-good {
  background: #409eff;
}

.metric-icon.quality-fair {
  background: #e6a23c;
}

.metric-icon.quality-poor {
  background: #f56c6c;
}

.metric-content {
  flex: 1;
}

.metric-label {
  font-size: 12px;
  color: #666;
  margin-bottom: 4px;
}

.metric-value {
  font-size: 16px;
  font-weight: bold;
  color: #333;
}

.quality-details {
  margin-top: 16px;
}

.detail-section h4 {
  margin: 0 0 12px 0;
  color: #303133;
  font-size: 14px;
}

.quality-distribution {
  space-y: 8px;
}

.distribution-item {
  display: flex;
  align-items: center;
  margin-bottom: 8px;
  gap: 8px;
}

.dist-label {
  width: 120px;
  font-size: 12px;
  color: #666;
}

.dist-count {
  width: 30px;
  text-align: right;
  font-size: 12px;
  font-weight: bold;
  color: #333;
}

.suggestions {
  space-y: 8px;
}

.suggestion-item {
  display: flex;
  align-items: center;
  padding: 8px 12px;
  border-radius: 6px;
  margin-bottom: 8px;
  font-size: 13px;
  gap: 8px;
}

.suggestion-item.high {
  background: #fef0f0;
  border: 1px solid #fbc4c4;
  color: #f56c6c;
}

.suggestion-item.medium {
  background: #fdf6ec;
  border: 1px solid #f5dab1;
  color: #e6a23c;
}

.suggestion-item.low {
  background: #f0f9ff;
  border: 1px solid #b3d8ff;
  color: #409eff;
}

.action-buttons {
  margin-top: 16px;
  display: flex;
  gap: 8px;
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
</style>
