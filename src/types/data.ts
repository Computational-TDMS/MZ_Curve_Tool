// 数据容器类型
export interface DataContainer {
  metadata: Record<string, any>
  spectra: any[]
  curves: Curve[]
  peaks: Peak[]
}

// 可序列化的数据容器类型（用于前后端通信）
export interface SerializableDataContainer {
  metadata: Record<string, any>
  spectra: any[]
  curves: Curve[]
  peaks: Peak[]
}

// 光谱数据类型
export interface Spectrum {
  id: string
  ms_level: number
  precursor_mz?: number
  retention_time?: number
  mz_values: number[]
  intensity_values: number[]
  point_count: number
}

// 曲线类型
export interface Curve {
  id: string
  curve_type: string
  x_values: number[]
  y_values: number[]
  x_min: number
  x_max: number
  y_min: number
  y_max: number
  point_count: number
}

// 峰类型
export interface Peak {
  id: string
  center: number
  amplitude: number
  fwhm: number
  area: number
  rsquared: number
  fit_parameters: Record<string, any>
  metadata: Record<string, any>
}

// 数据范围类型
export interface DataRanges {
  rt_min: number
  rt_max: number
  mz_min: number
  mz_max: number
  ms_levels: number[]
}

// 文件信息类型
export interface FileInfo {
  path: string
  name: string
  size: number
  format: string
  is_valid: boolean
  spectra_count?: number
  data_ranges?: DataRanges
}

// 日志消息类型
export interface LogMessage {
  id: string
  level: string
  title: string
  content: string
  timestamp: string
}

export interface CurveDisplayData {
  id: string
  curve_type: string
  x_label: string
  y_label: string
  x_unit: string
  y_unit: string
  point_count: number
  x_values: number[]
  y_values: number[]
  mz_min?: number
  mz_max?: number
}

// 进度更新类型
export interface ProgressUpdate {
  current: number
  total: number
  message: string
  percentage: number
}

// 处理状态类型
export type ProcessingStatus = 
  | 'idle'
  | 'loading'
  | 'extracting'
  | 'analyzing'
  | 'exporting'
  | 'error'
  | 'success'
