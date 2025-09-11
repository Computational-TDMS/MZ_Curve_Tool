//! 峰处理工作流类型定义
//! 提供前端调用峰处理工作流的类型支持

// 处理模式
export type ProcessingMode = 
  | { type: 'automatic' }
  | { type: 'manual'; strategy: ProcessingStrategyRequest }
  | { type: 'hybrid'; manualOverrides: Record<string, string> }
  | { type: 'predefined'; strategyName: string };

// 处理策略请求
export interface ProcessingStrategyRequest {
  name: string;
  description: string;
  peakDetection: string;
  overlapProcessing: string;
  fittingMethod: string;
  optimizationAlgorithm: string;
  advancedAlgorithm?: string;
  postProcessing?: string;
  configuration: Record<string, any>;
}

// 峰处理请求
export interface PeakProcessingRequest {
  peaks: Peak[];
  curve: Curve;
  mode: ProcessingMode;
  config?: Record<string, any>;
  manualOverrides?: Record<string, string>;
}

// 峰处理响应
export interface PeakProcessingResponse {
  peaks: Peak[];
  statistics: ProcessingStatistics;
  logs: string[];
  success: boolean;
  error?: string;
}

// 处理统计信息
export interface ProcessingStatistics {
  inputPeakCount: number;
  outputPeakCount: number;
  processingTimeMs: number;
  strategyName: string;
  qualityScore: number;
  stageTimes: Record<string, number>;
}

// 组件信息
export interface ComponentInfo {
  componentType: string;
  name: string;
  version: string;
  description: string;
  capabilities: string[];
}

// 策略信息
export interface StrategyInfo {
  name: string;
  description: string;
  peakDetection: string;
  overlapProcessing: string;
  fittingMethod: string;
  optimizationAlgorithm: string;
  advancedAlgorithm?: string;
  postProcessing?: string;
}

// 峰数据结构（与后端兼容）
export interface Peak {
  id: string;
  curveId: string;
  center: number;
  amplitude: number;
  area: number;
  fwhm: number;
  hwhm: number;
  sigma: number;
  gamma: number;
  tau: number;
  leftHwhm: number;
  rightHwhm: number;
  asymmetryFactor: number;
  leftBoundary: number;
  rightBoundary: number;
  peakSpan: number;
  rsquared: number;
  residualSumSquares: number;
  standardError: number;
  parameterCount: number;
  peakType: string;
  mixingParameter: number;
  signalToBaselineRatio: number;
  areaPercentage: number;
  intensityPercentage: number;
  leftDerivative: number;
  rightDerivative: number;
  derivativeRatio: number;
  mz?: number;
  retentionTime?: number;
  driftTime?: number;
  msLevel?: number;
  detectionAlgorithm: string;
  detectionThreshold: number;
  confidence: number;
  fitParameters: number[];
  fitParameterErrors: number[];
  fitCovarianceMatrix?: number[][];
  metadata: Record<string, any>;
}

// 曲线数据结构（与后端兼容）
export interface Curve {
  id: string;
  curveType: string;
  xValues: number[];
  yValues: number[];
  xLabel: string;
  yLabel: string;
  xUnit: string;
  yUnit: string;
  xMin: number;
  xMax: number;
  yMin: number;
  yMax: number;
  pointCount: number;
  totalIonCurrent: number;
  meanIntensity: number;
  intensityStd: number;
  baselineIntensity: number;
  signalToNoiseRatio: number;
  mzRange?: [number, number];
  rtRange?: [number, number];
  dtRange?: [number, number];
  msLevel?: number;
  smoothingFactor?: number;
  baselineCorrection?: string;
  noiseLevel: number;
  detectionThreshold: number;
  qualityScore: number;
  completeness: number;
  hasMissingPoints: boolean;
  metadata: Record<string, any>;
  peaks: Peak[];
}

// 工作流配置选项
export interface WorkflowConfigOptions {
  // 自动模式选项
  autoMode?: {
    enableOverlapRule: boolean;
    enableComplexityRule: boolean;
    enableSNRRule: boolean;
    enableQualityRule: boolean;
  };
  
  // 手动模式选项
  manualMode?: {
    strategy: ProcessingStrategyRequest;
  };
  
  // 混合模式选项
  hybridMode?: {
    autoStrategy: string;
    manualOverrides: Record<string, string>;
  };
  
  // 预定义策略选项
  predefinedMode?: {
    strategyName: string;
  };
  
  // 全局配置
  globalConfig?: Record<string, any>;
}

// 工作流状态
export interface WorkflowStatus {
  isInitialized: boolean;
  isProcessing: boolean;
  currentMode: ProcessingMode['type'] | null;
  availableStrategies: string[];
  availableComponents: ComponentInfo[];
  lastProcessingResult?: PeakProcessingResponse;
  error?: string;
}

// 工作流事件
export interface WorkflowEvent {
  type: 'initialized' | 'processing_started' | 'processing_completed' | 'processing_failed' | 'error';
  data?: any;
  timestamp: number;
}

// 工作流管理器接口
export interface PeakProcessingWorkflow {
  // 初始化
  initialize(): Promise<void>;
  
  // 处理峰数据
  processPeaks(request: PeakProcessingRequest): Promise<PeakProcessingResponse>;
  
  // 获取可用组件
  getAvailableComponents(): Promise<ComponentInfo[]>;
  
  // 获取可用策略
  getAvailableStrategies(): Promise<StrategyInfo[]>;
  
  // 获取组件信息
  getComponentInfo(componentType: string, componentName: string): Promise<ComponentInfo | null>;
  
  // 验证配置
  validateConfig(configName: string, config: Record<string, any>): Promise<boolean>;
  
  // 获取配置架构
  getConfigSchema(configName: string): Promise<Record<string, any> | null>;
  
  // 获取工作流状态
  getStatus(): WorkflowStatus;
  
  // 事件监听
  on(event: WorkflowEvent['type'], callback: (event: WorkflowEvent) => void): void;
  off(event: WorkflowEvent['type'], callback: (event: WorkflowEvent) => void): void;
}
