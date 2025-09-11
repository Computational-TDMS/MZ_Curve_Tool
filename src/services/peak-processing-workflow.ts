//! 峰处理工作流管理器
//! 提供前端峰处理工作流的统一管理接口

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import type {
  PeakProcessingWorkflow,
  WorkflowStatus,
  WorkflowEvent,
  PeakProcessingRequest,
  PeakProcessingResponse,
  ComponentInfo,
  StrategyInfo,
  ProcessingMode,
  ProcessingStrategyRequest,
} from '../types/peak-processing';

export class PeakProcessingWorkflowManager implements PeakProcessingWorkflow {
  private status: WorkflowStatus = {
    isInitialized: false,
    isProcessing: false,
    currentMode: null,
    availableStrategies: [],
    availableComponents: [],
  };
  
  private eventListeners: Map<string, Set<(event: WorkflowEvent) => void>> = new Map();
  
  constructor() {
    this.setupEventListeners();
  }
  
  /**
   * 初始化峰处理控制器
   */
  async initialize(): Promise<void> {
    try {
      const result = await invoke<string>('init_peak_processing_controller');
      console.log('峰处理控制器初始化:', result);
      
      this.status.isInitialized = true;
      this.emitEvent({
        type: 'initialized',
        timestamp: Date.now(),
      });
      
      // 加载可用组件和策略
      await this.loadAvailableComponents();
      await this.loadAvailableStrategies();
      
    } catch (error) {
      console.error('峰处理控制器初始化失败:', error);
      this.status.error = error as string;
      this.emitEvent({
        type: 'error',
        data: { error },
        timestamp: Date.now(),
      });
      throw error;
    }
  }
  
  /**
   * 处理峰数据
   */
  async processPeaks(request: PeakProcessingRequest): Promise<PeakProcessingResponse> {
    if (!this.status.isInitialized) {
      throw new Error('峰处理控制器未初始化，请先调用 initialize()');
    }
    
    this.status.isProcessing = true;
    this.status.currentMode = request.mode.type;
    
    this.emitEvent({
      type: 'processing_started',
      data: { request },
      timestamp: Date.now(),
    });
    
    try {
      const response = await invoke<PeakProcessingResponse>('process_peaks', { request });
      
      this.status.isProcessing = false;
      this.status.lastProcessingResult = response;
      
      if (response.success) {
        this.emitEvent({
          type: 'processing_completed',
          data: { response },
          timestamp: Date.now(),
        });
      } else {
        this.emitEvent({
          type: 'processing_failed',
          data: { response },
          timestamp: Date.now(),
        });
      }
      
      return response;
      
    } catch (error) {
      this.status.isProcessing = false;
      this.status.error = error as string;
      
      this.emitEvent({
        type: 'processing_failed',
        data: { error },
        timestamp: Date.now(),
      });
      
      throw error;
    }
  }
  
  /**
   * 获取可用组件列表
   */
  async getAvailableComponents(): Promise<ComponentInfo[]> {
    try {
      const components = await invoke<ComponentInfo[]>('get_available_components');
      this.status.availableComponents = components;
      return components;
    } catch (error) {
      console.error('获取可用组件失败:', error);
      throw error;
    }
  }
  
  /**
   * 获取可用策略列表
   */
  async getAvailableStrategies(): Promise<StrategyInfo[]> {
    try {
      const strategies = await invoke<StrategyInfo[]>('get_available_strategies');
      this.status.availableStrategies = strategies.map(s => s.name);
      return strategies;
    } catch (error) {
      console.error('获取可用策略失败:', error);
      throw error;
    }
  }
  
  /**
   * 获取组件详细信息
   */
  async getComponentInfo(componentType: string, componentName: string): Promise<ComponentInfo | null> {
    try {
      return await invoke<ComponentInfo | null>('get_component_info', {
        componentType,
        componentName,
      });
    } catch (error) {
      console.error('获取组件信息失败:', error);
      throw error;
    }
  }
  
  /**
   * 验证配置
   */
  async validateConfig(configName: string, config: Record<string, any>): Promise<boolean> {
    try {
      return await invoke<boolean>('validate_config', {
        configName,
        config,
      });
    } catch (error) {
      console.error('配置验证失败:', error);
      return false;
    }
  }
  
  /**
   * 获取配置架构
   */
  async getConfigSchema(configName: string): Promise<Record<string, any> | null> {
    try {
      return await invoke<Record<string, any> | null>('get_config_schema', {
        configName,
      });
    } catch (error) {
      console.error('获取配置架构失败:', error);
      throw error;
    }
  }
  
  /**
   * 获取工作流状态
   */
  getStatus(): WorkflowStatus {
    return { ...this.status };
  }
  
  /**
   * 事件监听
   */
  on(event: WorkflowEvent['type'], callback: (event: WorkflowEvent) => void): void {
    if (!this.eventListeners.has(event)) {
      this.eventListeners.set(event, new Set());
    }
    this.eventListeners.get(event)!.add(callback);
  }
  
  /**
   * 移除事件监听
   */
  off(event: WorkflowEvent['type'], callback: (event: WorkflowEvent) => void): void {
    const listeners = this.eventListeners.get(event);
    if (listeners) {
      listeners.delete(callback);
    }
  }
  
  /**
   * 创建自动模式请求
   */
  createAutomaticRequest(peaks: any[], curve: any, config?: Record<string, any>): PeakProcessingRequest {
    return {
      peaks,
      curve,
      mode: { type: 'automatic' },
      config,
    };
  }
  
  /**
   * 创建手动模式请求
   */
  createManualRequest(
    peaks: any[],
    curve: any,
    strategy: ProcessingStrategyRequest,
    config?: Record<string, any>
  ): PeakProcessingRequest {
    return {
      peaks,
      curve,
      mode: { type: 'manual', strategy },
      config,
    };
  }
  
  /**
   * 创建混合模式请求
   */
  createHybridRequest(
    peaks: any[],
    curve: any,
    manualOverrides: Record<string, string>,
    config?: Record<string, any>
  ): PeakProcessingRequest {
    return {
      peaks,
      curve,
      mode: { type: 'hybrid', manualOverrides },
      config,
    };
  }
  
  /**
   * 创建预定义策略请求
   */
  createPredefinedRequest(
    peaks: any[],
    curve: any,
    strategyName: string,
    config?: Record<string, any>
  ): PeakProcessingRequest {
    return {
      peaks,
      curve,
      mode: { type: 'predefined', strategyName },
      config,
    };
  }
  
  /**
   * 加载可用组件
   */
  private async loadAvailableComponents(): Promise<void> {
    try {
      await this.getAvailableComponents();
    } catch (error) {
      console.warn('加载可用组件失败:', error);
    }
  }
  
  /**
   * 加载可用策略
   */
  private async loadAvailableStrategies(): Promise<void> {
    try {
      await this.getAvailableStrategies();
    } catch (error) {
      console.warn('加载可用策略失败:', error);
    }
  }
  
  /**
   * 设置事件监听器
   */
  private setupEventListeners(): void {
    // 监听 Tauri 事件
    listen('status-updated', (event) => {
      console.log('状态更新事件:', event.payload);
    });
    
    listen('log-message', (event) => {
      console.log('日志消息事件:', event.payload);
    });
    
    listen('progress-updated', (event) => {
      console.log('进度更新事件:', event.payload);
    });
  }
  
  /**
   * 发送事件
   */
  private emitEvent(event: WorkflowEvent): void {
    const listeners = this.eventListeners.get(event.type);
    if (listeners) {
      listeners.forEach(callback => {
        try {
          callback(event);
        } catch (error) {
          console.error('事件回调执行失败:', error);
        }
      });
    }
  }
}

// 创建全局实例
export const peakProcessingWorkflow = new PeakProcessingWorkflowManager();

// 导出便捷方法
export const {
  initialize,
  processPeaks,
  getAvailableComponents,
  getAvailableStrategies,
  getComponentInfo,
  validateConfig,
  getConfigSchema,
  getStatus,
  on,
  off,
  createAutomaticRequest,
  createManualRequest,
  createHybridRequest,
  createPredefinedRequest,
} = peakProcessingWorkflow;
