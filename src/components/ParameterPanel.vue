<template>
  <div class="parameter-panel">
    <el-card class="panel-card">
      <template #header>
        <div class="card-header">
          <span>参数配置</span>
        </div>
      </template>

      <!-- 文件选择 -->
      <el-form :model="form" label-width="80px" size="small">
        <el-form-item label="数据文件">
          <el-input 
            v-model="form.filePath" 
            placeholder="选择或输入文件路径"
            readonly
          >
            <template #append>
              <el-button @click="selectFile" :icon="FolderOpened">选择</el-button>
            </template>
          </el-input>
        </el-form-item>


        <!-- 数据范围设置 -->
        <el-divider content-position="left">数据范围</el-divider>
        
        <el-form-item label="m/z 范围">
          <el-row :gutter="8">
            <el-col :span="12">
              <el-input 
                v-model="form.mzMin" 
                placeholder="最小值"
                style="width: 100%"
                type="number"
                step="0.1"
              />
            </el-col>
            <el-col :span="12">
              <el-input 
                v-model="form.mzMax" 
                placeholder="最大值"
                style="width: 100%"
                type="number"
                step="0.1"
              />
            </el-col>
          </el-row>
        </el-form-item>

        <el-form-item label="RT 范围">
          <el-row :gutter="8">
            <el-col :span="12">
              <el-input 
                v-model="form.rtMin" 
                placeholder="最小值"
                style="width: 100%"
                type="number"
                step="0.1"
              />
            </el-col>
            <el-col :span="12">
              <el-input 
                v-model="form.rtMax" 
                placeholder="最大值"
                style="width: 100%"
                type="number"
                step="0.1"
              />
            </el-col>
          </el-row>
        </el-form-item>

        <el-form-item label="MS 级别">
          <el-select v-model="form.msLevel" style="width: 100%">
            <el-option label="MS1" :value="1" />
            <el-option label="MS2" :value="2" />
            <el-option label="MS3" :value="3" />
          </el-select>
        </el-form-item>

        <el-form-item label="曲线类型">
          <el-select v-model="form.curveType" style="width: 100%">
            <el-option label="DT (Drift Time)" value="dt" />
            <el-option label="TIC (Total Ion Current)" value="tic" />
            <el-option label="XIC (Extracted Ion Current)" value="xic" />
          </el-select>
        </el-form-item>

        <!-- 峰检测参数 -->
        <el-divider content-position="left">峰检测</el-divider>
        
        <el-form-item label="检测方法">
          <el-select v-model="form.detectionMethod" style="width: 100%">
            <el-option label="CWT (连续小波变换)" value="cwt" />
            <el-option label="简单阈值检测" value="simple" />
            <el-option label="PeakFinder算法" value="peak_finder" />
          </el-select>
        </el-form-item>

        <el-form-item label="检测灵敏度">
          <el-slider 
            v-model="form.sensitivity" 
            :min="0.1" 
            :max="1.0" 
            :step="0.1"
            :format-tooltip="(val: number) => val.toFixed(1)"
            show-input
            :show-input-controls="false"
          />
        </el-form-item>

        <!-- 峰拟合参数 -->
        <el-divider content-position="left">峰拟合</el-divider>
        
        <el-form-item label="拟合方法">
          <el-select v-model="form.fittingMethod" style="width: 100%">
            <el-option label="高斯拟合" value="gaussian" />
            <el-option label="洛伦兹拟合" value="lorentzian" />
            <el-option label="Pseudo-Voigt拟合" value="pseudo_voigt" />
            <el-option label="EMG (指数修正高斯)" value="emg" />
          </el-select>
        </el-form-item>

        <el-form-item label="重叠峰处理">
          <el-select v-model="form.overlappingMethod" style="width: 100%">
            <el-option label="自动" value="auto" />
            <el-option label="FBF预处理" value="fbf" />
            <el-option label="CWT锐化" value="sharpen_cwt" />
            <el-option label="EMG NLLS" value="emg_nlls" />
          </el-select>
        </el-form-item>

        <!-- 迭代优化参数 -->
        <el-divider content-position="left">迭代优化</el-divider>
        
        <el-form-item label="优化算法">
          <el-select v-model="form.optimizationAlgorithm" style="width: 100%">
            <el-option label="Levenberg-Marquardt" value="levenberg_marquardt" />
            <el-option label="梯度下降" value="gradient_descent" />
            <el-option label="网格搜索" value="grid_search" />
            <el-option label="模拟退火" value="simulated_annealing" />
          </el-select>
        </el-form-item>

        <el-form-item label="最大迭代次数">
          <el-input-number 
            v-model="form.maxIterations" 
            :min="10" 
            :max="1000" 
            :step="10"
            style="width: 100%"
          />
        </el-form-item>

        <el-form-item label="收敛阈值">
          <el-input-number 
            v-model="form.convergenceThreshold" 
            :min="1e-8" 
            :max="1e-3" 
            :step="1e-6"
            :precision="8"
            style="width: 100%"
          />
        </el-form-item>

        <el-form-item label="学习率">
          <el-input-number 
            v-model="form.learningRate" 
            :min="0.001" 
            :max="0.1" 
            :step="0.001"
            :precision="3"
            style="width: 100%"
          />
        </el-form-item>

        <el-form-item label="拟合质量阈值">
          <el-slider 
            v-model="form.fitQualityThreshold" 
            :min="0.5" 
            :max="1.0" 
            :step="0.01"
            :format-tooltip="(val: number) => val.toFixed(2)"
            show-input
            :show-input-controls="false"
          />
        </el-form-item>

        <el-form-item label="高级算法">
          <el-select v-model="form.advancedAlgorithm" style="width: 100%" clearable>
            <el-option label="自适应步长" value="adaptive_step" />
            <el-option label="多起点优化" value="multi_start" />
            <el-option label="约束优化" value="constrained" />
            <el-option label="鲁棒拟合" value="robust" />
          </el-select>
        </el-form-item>

        <!-- 操作按钮 -->
        <el-divider content-position="left">操作</el-divider>
        
        <el-form-item>
          <el-row :gutter="8">
            <el-col :span="12">
              <el-button 
                type="primary" 
                @click="loadFile" 
                :loading="loading"
                style="width: 100%"
              >
                加载文件
              </el-button>
            </el-col>
            <el-col :span="12">
              <el-button 
                @click="extractCurve" 
                :loading="loading"
                style="width: 100%"
              >
                提取曲线
              </el-button>
            </el-col>
          </el-row>
        </el-form-item>

        <el-form-item>
          <el-row :gutter="8">
            <el-col :span="12">
              <el-button 
                type="success" 
                @click="detectPeaks" 
                :loading="loading"
                style="width: 100%"
              >
                峰检测
              </el-button>
            </el-col>
            <el-col :span="12">
              <el-button 
                type="warning" 
                @click="fitPeaks" 
                :loading="loading"
                style="width: 100%"
              >
                峰拟合
              </el-button>
            </el-col>
          </el-row>
        </el-form-item>

        <el-form-item>
          <el-row :gutter="8">
            <el-col :span="12">
              <el-button 
                type="success" 
                @click="runFullPipeline" 
                :loading="loading"
                style="width: 100%"
              >
                自动处理
              </el-button>
            </el-col>
            <el-col :span="12">
              <el-button 
                @click="exportResults" 
                :loading="loading"
                style="width: 100%"
              >
                导出结果
              </el-button>
            </el-col>
          </el-row>
        </el-form-item>

        <el-form-item>
          <el-button 
            type="primary" 
            @click="startBatchProcessing" 
            :loading="loading"
            style="width: 100%"
            :icon="Upload"
          >
            批量处理模式
          </el-button>
        </el-form-item>


        <el-form-item>
          <el-button 
            type="primary" 
            @click="exportSpectroData" 
            :loading="loading"
            style="width: 100%"
            :icon="Download"
          >
            导出DT-mz数据
          </el-button>
        </el-form-item>
      </el-form>
    </el-card>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { FolderOpened, Download, Upload } from '@element-plus/icons-vue'
import type { DataRanges } from '../types/data'

// 定义 props
const props = defineProps<{
  dataRanges?: DataRanges
  selectedFile?: {id: string, path: string} | null
}>()

// 定义事件
const emit = defineEmits([
  'load-file',
  'extract-curve', 
  'detect-peaks',
  'fit-peaks',
  'run-pipeline',
  'export-results',
  'export-spectro-data',
  'batch-processing'
])

// 响应式数据
const loading = ref(false)

const form = reactive({
  filePath: '',
  mzMin: '100.0',
  mzMax: '200.0',
  rtMin: '0.0',
  rtMax: '60.0',
  msLevel: 1,
  curveType: 'dt',
  detectionMethod: 'cwt',
  sensitivity: 0.7,
  fittingMethod: 'gaussian',
  overlappingMethod: 'auto',
  // 迭代优化参数
  optimizationAlgorithm: 'levenberg_marquardt',
  maxIterations: 100,
  convergenceThreshold: 1e-6,
  learningRate: 0.01,
  fitQualityThreshold: 0.8,
  advancedAlgorithm: ''
})

// 监听数据范围变化，自动更新表单值
watch(() => props.dataRanges, (newRanges) => {
  console.log('数据范围变化:', newRanges)
  if (newRanges) {
    form.rtMin = newRanges.rt_min.toFixed(2)
    form.rtMax = newRanges.rt_max.toFixed(2)
    form.mzMin = newRanges.mz_min.toFixed(2)
    form.mzMax = newRanges.mz_max.toFixed(2)
    console.log('表单已更新:', form.rtMin, form.rtMax, form.mzMin, form.mzMax)
    ElMessage.success('已自动更新数据范围')
  }
}, { deep: true, immediate: true })

// 方法
async function selectFile() {
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
      multiple: false,
      defaultPath: ''
    })
    
    if (selected && typeof selected === 'string') {
      form.filePath = selected
      ElMessage.success(`已选择文件: ${selected.split('\\').pop() || selected.split('/').pop()}`)
    }
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : String(error)
    ElMessage.error('文件选择失败: ' + errorMessage)
    console.error('文件选择错误:', error)
  }
}

function loadFile() {
  if (!form.filePath) {
    ElMessage.warning('请先选择文件路径')
    return
  }
  emit('load-file', form.filePath)
}

function extractCurve() {
  const params = {
    file_path: form.filePath,
    mz_range: `${form.mzMin}-${form.mzMax}`,
    rt_range: `${form.rtMin}-${form.rtMax}`,
    ms_level: form.msLevel,
    curve_type: form.curveType
  }
  emit('extract-curve', params)
}

function detectPeaks() {
  const params = {
    method: form.detectionMethod,
    sensitivity: form.sensitivity,
    threshold_multiplier: 3.0,
    min_peak_width: 0.1,
    max_peak_width: 10.0
  }
  emit('detect-peaks', params)
}

function fitPeaks() {
  const params = {
    method: form.fittingMethod,
    overlapping_method: form.overlappingMethod,
    min_peak_width: 0.1,
    max_peak_width: 10.0,
    fit_quality_threshold: form.fitQualityThreshold,
    // 迭代优化参数
    optimization_algorithm: form.optimizationAlgorithm,
    max_iterations: form.maxIterations,
    convergence_threshold: form.convergenceThreshold,
    learning_rate: form.learningRate,
    advanced_algorithm: form.advancedAlgorithm || null
  }
  emit('fit-peaks', params)
}

function runFullPipeline() {
  const params = {
    extraction: {
      file_path: form.filePath,
      mz_range: `${form.mzMin}-${form.mzMax}`,
      rt_range: `${form.rtMin}-${form.rtMax}`,
      ms_level: form.msLevel,
      curve_type: form.curveType
    },
    detection: {
      method: form.detectionMethod,
      sensitivity: form.sensitivity,
      threshold_multiplier: 3.0,
      min_peak_width: 0.1,
      max_peak_width: 10.0
    },
    fitting: {
      method: form.fittingMethod,
      overlapping_method: form.overlappingMethod,
      fit_quality_threshold: form.fitQualityThreshold,
      optimization_algorithm: form.optimizationAlgorithm,
      max_iterations: form.maxIterations,
      convergence_threshold: form.convergenceThreshold,
      learning_rate: form.learningRate,
      advanced_algorithm: form.advancedAlgorithm || null
    }
  }
  emit('run-pipeline', params)
}

function exportResults() {
  emit('export-results')
}

function exportSpectroData() {
  if (!form.filePath) {
    ElMessage.warning('请先选择文件')
    return
  }
  
  const params = {
    file_path: form.filePath,
    include_header: true,
    decimal_precision: 6,
    include_metadata: true,
    filter_by_ms_level: form.msLevel,
    mz_range_min: parseFloat(form.mzMin),
    mz_range_max: parseFloat(form.mzMax),
    rt_range_min: parseFloat(form.rtMin),
    rt_range_max: parseFloat(form.rtMax),
    intensity_threshold: 0.0
  }
  
  emit('export-spectro-data', params)
}

function startBatchProcessing() {
  const params = {
    extraction: {
      file_path: form.filePath,
      mz_range: `${form.mzMin}-${form.mzMax}`,
      rt_range: `${form.rtMin}-${form.rtMax}`,
      ms_level: form.msLevel,
      curve_type: form.curveType
    },
    detection: {
      method: form.detectionMethod,
      sensitivity: form.sensitivity,
      threshold_multiplier: 3.0,
      min_peak_width: 0.1,
      max_peak_width: 10.0
    },
    fitting: {
      method: form.fittingMethod,
      overlapping_method: form.overlappingMethod,
      fit_quality_threshold: form.fitQualityThreshold,
      optimization_algorithm: form.optimizationAlgorithm,
      max_iterations: form.maxIterations,
      convergence_threshold: form.convergenceThreshold,
      learning_rate: form.learningRate,
      advanced_algorithm: form.advancedAlgorithm || null
    }
  }
  emit('batch-processing', params)
}

</script>

<style scoped>
.parameter-panel {
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

.el-form-item {
  margin-bottom: 16px;
}

.el-divider {
  margin: 16px 0;
}

.batch-info {
  margin-top: 8px;
  display: flex;
  align-items: center;
}
</style>
