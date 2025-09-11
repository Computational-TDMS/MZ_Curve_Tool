use mzdata::prelude::*;
use mzdata::MZReader;
use mzdata::spectrum::Spectrum;
use crate::core::data::{DataContainer, ProcessingError};
use std::collections::HashMap;

/// 进度回调函数类型
pub type ProgressCallback = Box<dyn Fn(usize, usize, &str) + Send + Sync>;

/// 数据加载器 - 支持进度报告
pub struct DataLoader;

impl DataLoader {
    /// 加载文件并支持进度报告
    pub fn load_from_file_with_progress(
        path: &str, 
        progress_callback: Option<ProgressCallback>
    ) -> Result<DataContainer, ProcessingError> {
        log::info!("🚀 开始加载文件: {}", path);
        
        // 使用MZReader自动推断文件格式
        let reader = MZReader::open_path(path).map_err(|e| ProcessingError::MzDataError(e.to_string()))?;
        
        if let Some(ref callback) = progress_callback {
            callback(0, 0, "开始读取光谱数据...");
        }
        
        let mut container = DataContainer {
            metadata: HashMap::new(),
            spectra: Vec::new(),
            curves: Vec::new(),
        };
        
        let mut processed_count = 0;
        const PROGRESS_UPDATE_INTERVAL: usize = 100; // 每100个光谱更新一次进度
        
        // 直接收集 mzdata::Spectrum，无需转换
        for spectrum in reader {
            container.spectra.push(spectrum);
            processed_count += 1;
            
            // 定期更新进度
            if processed_count % PROGRESS_UPDATE_INTERVAL == 0 {
                if let Some(ref callback) = progress_callback {
                    callback(processed_count, 0, &format!("已读取 {} 个光谱", processed_count));
                }
            }
        }
        
        // 最终进度更新
        if let Some(ref callback) = progress_callback {
            callback(processed_count, processed_count, &format!("完成读取 {} 个光谱", processed_count));
        }
        
        log::info!("✅ 文件加载完成: {} 个光谱", processed_count);
        
        // 添加基本元数据
        container.metadata.insert("file_path".to_string(), serde_json::Value::String(path.to_string()));
        container.metadata.insert("spectrum_count".to_string(), serde_json::Value::Number(serde_json::Number::from(processed_count)));
        
        // 自动计算 RT 和 m/z 范围
        if !container.spectra.is_empty() {
            if let Some(ref callback) = progress_callback {
                callback(processed_count, processed_count, "计算数据范围...");
            }
            
            let (rt_min, rt_max) = Self::calculate_rt_range(&container.spectra);
            let (mz_min, mz_max) = Self::calculate_mz_range(&container.spectra);
            
            container.metadata.insert("rt_min".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(rt_min).unwrap()));
            container.metadata.insert("rt_max".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(rt_max).unwrap()));
            container.metadata.insert("mz_min".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(mz_min).unwrap()));
            container.metadata.insert("mz_max".to_string(), serde_json::Value::Number(serde_json::Number::from_f64(mz_max).unwrap()));
            
            log::info!("📊 数据范围 - RT: {:.2} - {:.2}, m/z: {:.2} - {:.2}", rt_min, rt_max, mz_min, mz_max);
        }
        
        Ok(container)
    }
    
    /// 原始方法，保持向后兼容
    pub fn load_from_file(path: &str) -> Result<DataContainer, ProcessingError> {
        Self::load_from_file_with_progress(path, None)
    }
    
    /// 过滤光谱数据 - 保留此函数，因为被其他模块使用
    pub fn filter_spectra(
        spectra: &[Spectrum],
        ms_level: Option<u8>,
        rt_min: Option<f64>,
        rt_max: Option<f64>,
        mz_min: Option<f64>,
        mz_max: Option<f64>,
    ) -> Vec<&Spectrum> {
        spectra.iter()
            .filter(|s| {
                // MS级别过滤
                if let Some(level) = ms_level {
                    if s.ms_level() != level {
                        return false;
                    }
                }
                
                // 保留时间过滤
                if let Some(min) = rt_min {
                    if s.start_time() < min {
                        return false;
                    }
                }
                if let Some(max) = rt_max {
                    if s.start_time() > max {
                        return false;
                    }
                }
                
                // m/z范围过滤（检查是否有数据在指定范围内）
                if let (Some(min), Some(max)) = (mz_min, mz_max) {
                    let peaks = s.peaks();
                    let has_data_in_range = peaks.iter().any(|peak| {
                        let mz = peak.mz();
                        mz >= min && mz <= max
                    });
                    if !has_data_in_range {
                        return false;
                    }
                }
                
                true
            })
            .collect()
    }
    
    /// 计算保留时间范围
    fn calculate_rt_range(spectra: &[Spectrum]) -> (f64, f64) {
        if spectra.is_empty() {
            return (0.0, 0.0);
        }
        
        let mut min_rt = f64::INFINITY;
        let mut max_rt: f64 = 0.0;
        
        for spectrum in spectra {
            let rt = spectrum.start_time();
            min_rt = min_rt.min(rt);
            max_rt = max_rt.max(rt);
        }
        
        (min_rt, max_rt)
    }
    
    /// 计算m/z范围
    fn calculate_mz_range(spectra: &[Spectrum]) -> (f64, f64) {
        if spectra.is_empty() {
            return (0.0, 0.0);
        }
        
        let mut min_mz = f64::INFINITY;
        let mut max_mz: f64 = 0.0;
        
        for spectrum in spectra {
            let peaks = spectrum.peaks();
            for peak in peaks.iter() {
                let mz = peak.mz();
                min_mz = min_mz.min(mz);
                max_mz = max_mz.max(mz);
            }
        }
        
        (min_mz, max_mz)
    }
}
