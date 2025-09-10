use serde::{Deserialize, Serialize};
use crate::core::data::Curve;

/// 基线校准算法的配置参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineConfig {
    /// 基线校准方法
    pub method: BaselineMethod,
    /// 是否保留原始数据
    pub preserve_original: bool,
    /// 是否输出基线数据
    pub output_baseline: bool,
    /// 自定义参数
    pub custom_params: std::collections::HashMap<String, serde_json::Value>,
}

impl Default for BaselineConfig {
    fn default() -> Self {
        Self {
            method: BaselineMethod::Linear,
            preserve_original: true,
            output_baseline: false,
            custom_params: std::collections::HashMap::new(),
        }
    }
}

/// 基线校准方法枚举
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BaselineMethod {
    /// 线性基线校准
    Linear,
    /// 多项式基线校准
    Polynomial { degree: u32 },
    /// 移动平均基线校准
    MovingAverage { window_size: usize },
    /// 非对称最小二乘法
    AsymmetricLeastSquares { 
        lambda: f64, 
        p: f64, 
        max_iterations: usize 
    },
    /// 手动基线校准
    Manual { baseline_points: Vec<(f64, f64)> },
}

/// 基线校准结果
#[derive(Debug, Clone)]
pub struct BaselineResult {
    /// 校准后的曲线数据
    pub corrected_curve: Curve,
    /// 基线数据（如果请求）
    pub baseline_curve: Option<Curve>,
    /// 校准统计信息
    pub statistics: BaselineStatistics,
}

/// 基线校准统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineStatistics {
    /// 原始基线强度
    pub original_baseline: f64,
    /// 校准后基线强度
    pub corrected_baseline: f64,
    /// 基线偏移量
    pub baseline_offset: f64,
    /// 校准质量评分 (0-1)
    pub quality_score: f64,
    /// 使用的校准方法
    pub method_used: String,
    /// 处理时间（毫秒）
    pub processing_time_ms: u64,
}

/// 基线校准算法trait
pub trait BaselineAlgorithm {
    /// 算法名称
    fn name(&self) -> &str;
    
    /// 算法描述
    fn description(&self) -> &str;
    
    /// 校准基线
    fn correct_baseline(
        &self,
        curve: &Curve,
        config: &BaselineConfig,
    ) -> Result<BaselineResult, BaselineError>;
    
    /// 验证配置参数
    fn validate_config(&self, config: &BaselineConfig) -> Result<(), BaselineError>;
}

/// 基线校准错误类型
#[derive(Debug, thiserror::Error)]
pub enum BaselineError {
    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),
    
    #[error("Insufficient data points: required {required}, got {actual}")]
    InsufficientData { required: usize, actual: usize },
    
    #[error("Invalid polynomial degree: {degree}")]
    InvalidPolynomialDegree { degree: u32 },
    
    #[error("Invalid window size: {window_size}")]
    InvalidWindowSize { window_size: usize },
    
    #[error("Convergence failed after {iterations} iterations")]
    ConvergenceFailed { iterations: usize },
    
    #[error("Mathematical error: {0}")]
    MathError(String),
    
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// 基线校准工具函数
pub struct BaselineUtils;

impl BaselineUtils {
    /// 计算曲线的局部最小值点
    pub fn find_local_minima(curve: &Curve, window_size: usize) -> Vec<usize> {
        let mut minima = Vec::new();
        let half_window = window_size / 2;
        
        for i in half_window..curve.point_count - half_window {
            let mut is_minimum = true;
            let current_y = curve.y_values[i];
            
            // 检查窗口内的所有点
            for j in (i - half_window)..=(i + half_window) {
                if j != i && curve.y_values[j] <= current_y {
                    is_minimum = false;
                    break;
                }
            }
            
            if is_minimum {
                minima.push(i);
            }
        }
        
        minima
    }
    
    /// 线性插值
    pub fn linear_interpolation(
        x_values: &[f64],
        y_values: &[f64],
        target_x: f64,
    ) -> Option<f64> {
        if x_values.is_empty() || target_x < x_values[0] || target_x > x_values[x_values.len() - 1] {
            return None;
        }
        
        for i in 0..x_values.len() - 1 {
            if target_x >= x_values[i] && target_x <= x_values[i + 1] {
                let x1 = x_values[i];
                let x2 = x_values[i + 1];
                let y1 = y_values[i];
                let y2 = y_values[i + 1];
                
                let interpolated = y1 + (y2 - y1) * (target_x - x1) / (x2 - x1);
                return Some(interpolated);
            }
        }
        
        None
    }
    
    /// 计算均方根误差
    pub fn calculate_rmse(original: &[f64], corrected: &[f64]) -> f64 {
        if original.len() != corrected.len() {
            return f64::INFINITY;
        }
        
        let sum_squared_errors: f64 = original.iter()
            .zip(corrected.iter())
            .map(|(orig, corr)| (orig - corr).powi(2))
            .sum();
        
        (sum_squared_errors / original.len() as f64).sqrt()
    }
    
    /// 计算信噪比改善
    pub fn calculate_snr_improvement(
        original_snr: f64,
        corrected_snr: f64,
    ) -> f64 {
        if original_snr > 0.0 {
            (corrected_snr - original_snr) / original_snr
        } else {
            0.0
        }
    }
}
