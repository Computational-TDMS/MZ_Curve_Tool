use std::collections::HashMap;
use serde::{Serialize, Deserialize};
use mzdata::prelude::SpectrumLike;

use super::curve::Curve;
use super::peak::Peak;

/// Universal data container - does not directly serialize mzdata types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataContainer {
    pub metadata: HashMap<String, serde_json::Value>,
    #[serde(skip)] // 跳过序列化，仅内部使用
    pub spectra: Vec<mzdata::spectrum::Spectrum>,
    pub curves: Vec<Curve>,
    // 注意：peaks 现在嵌套在 curves 中，实现树状结构
}

/// 用于序列化的数据容器，不包含复杂的 mzdata 类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableDataContainer {
    pub metadata: HashMap<String, serde_json::Value>,
    pub spectra: Vec<serde_json::Value>, // 简化的光谱数据
    pub curves: Vec<Curve>,
    // 注意：peaks 现在嵌套在 curves 中，实现树状结构
}

impl From<DataContainer> for SerializableDataContainer {
    fn from(container: DataContainer) -> Self {
        // 优化：预分配容量，避免动态扩容
        let spectra_count = container.spectra.len();
        let mut spectra = Vec::with_capacity(spectra_count);
        
        // 将复杂的光谱数据转换为简单的 JSON 值
        for spectrum in container.spectra {
            // 简化光谱数据，只包含基本信息，避免不必要的计算
            spectra.push(serde_json::json!({
                "id": spectrum.id(),
                "ms_level": spectrum.ms_level(),
                "spectrum_type": "MultiLayerSpectrum",
                "has_data": true
            }));
        }
        
        Self {
            metadata: container.metadata,
            spectra,
            curves: container.curves,
        }
    }
}

impl From<SerializableDataContainer> for DataContainer {
    fn from(container: SerializableDataContainer) -> Self {
        // 从SerializableDataContainer转换回DataContainer
        // 注意：spectra字段会被清空，因为SerializableDataContainer中的spectra是简化的JSON数据
        Self {
            metadata: container.metadata,
            spectra: Vec::new(), // 清空spectra，因为导出时不需要原始光谱数据
            curves: container.curves,
        }
    }
}

impl Default for SerializableDataContainer {
    fn default() -> Self {
        Self {
            metadata: HashMap::new(),
            spectra: Vec::new(),
            curves: Vec::new(),
        }
    }
}

impl SerializableDataContainer {
    /// Create a new empty serializable data container
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a curve to the container
    pub fn add_curve(&mut self, curve: Curve) {
        self.curves.push(curve);
    }
    
    /// Add a peak to a specific curve
    pub fn add_peak_to_curve(&mut self, curve_id: &str, peak: Peak) -> Result<(), String> {
        if let Some(curve) = self.curves.iter_mut().find(|c| c.id == curve_id) {
            curve.add_peak(peak);
            Ok(())
        } else {
            Err(format!("Curve with ID '{}' not found", curve_id))
        }
    }
    
    /// Get the number of curves
    pub fn curve_count(&self) -> usize {
        self.curves.len()
    }
    
    /// Get the total number of peaks across all curves
    pub fn total_peak_count(&self) -> usize {
        self.curves.iter().map(|curve| curve.peak_count()).sum()
    }
    
    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
    
    /// Convert to DataContainer (alias for From trait)
    pub fn to_data_container(self) -> DataContainer {
        self.into()
    }
    
    /// Clear all data
    pub fn clear(&mut self) {
        self.metadata.clear();
        self.spectra.clear();
        self.curves.clear();
    }
}

impl Default for DataContainer {
    fn default() -> Self {
        Self {
            metadata: HashMap::new(),
            spectra: Vec::new(),
            curves: Vec::new(),
        }
    }
}

impl DataContainer {
    /// Create a new empty data container
    pub fn new() -> Self {
        Self::default()
    }
    
    /// Add a curve to the container
    pub fn add_curve(&mut self, curve: Curve) {
        self.curves.push(curve);
    }
    
    /// Add a peak to a specific curve
    pub fn add_peak_to_curve(&mut self, curve_id: &str, peak: Peak) -> Result<(), String> {
        if let Some(curve) = self.curves.iter_mut().find(|c| c.id == curve_id) {
            curve.add_peak(peak);
            Ok(())
        } else {
            Err(format!("Curve with ID '{}' not found", curve_id))
        }
    }
    
    /// Get the number of curves
    pub fn curve_count(&self) -> usize {
        self.curves.len()
    }
    
    /// Get the total number of peaks across all curves
    pub fn total_peak_count(&self) -> usize {
        self.curves.iter().map(|curve| curve.peak_count()).sum()
    }
    
    /// Get the number of spectra
    pub fn spectrum_count(&self) -> usize {
        self.spectra.len()
    }
    
    /// Clear all data
    pub fn clear(&mut self) {
        self.metadata.clear();
        self.spectra.clear();
        self.curves.clear();
    }
    
    /// Get curves by type
    pub fn get_curves_by_type(&self, curve_type: &str) -> Vec<&Curve> {
        self.curves.iter()
            .filter(|curve| curve.curve_type == curve_type)
            .collect()
    }
    
    /// Get peaks by curve ID
    pub fn get_peaks_by_curve_id(&self, curve_id: &str) -> Vec<&Peak> {
        if let Some(curve) = self.curves.iter().find(|c| c.id == curve_id) {
            curve.get_peaks().iter().collect()
        } else {
            Vec::new()
        }
    }
    
    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
    
    // === Memory management optimization methods ===
    
    /// Remove processed spectra to free memory
    /// 清理已处理的光谱数据以释放内存
    pub fn remove_processed_spectra(&mut self) {
        self.spectra.clear();
    }
    
    /// Split the container into smaller chunks for processing
    /// 将容器分割成小块进行处理，避免一次性加载所有数据到内存
    pub fn split_chunks(&self, chunk_size: usize) -> Vec<DataContainer> {
        let mut chunks = Vec::new();
        let total_curves = self.curves.len();
        
        for i in (0..total_curves).step_by(chunk_size) {
            let end = (i + chunk_size).min(total_curves);
            let chunk_curves = self.curves[i..end].to_vec();
            
            let chunk = DataContainer {
                metadata: self.metadata.clone(),
                spectra: Vec::new(), // 新块不包含原始光谱数据
                curves: chunk_curves,
            };
            
            chunks.push(chunk);
        }
        
        chunks
    }
    
    /// Estimate memory usage of the container
    /// 估算容器的内存使用量
    pub fn estimate_memory_usage(&self) -> usize {
        let mut total_size = 0;
        
        // 估算 metadata 大小
        total_size += std::mem::size_of_val(&self.metadata);
        for (key, value) in &self.metadata {
            total_size += key.len() + std::mem::size_of_val(value);
        }
        
        // 估算 spectra 大小
        total_size += std::mem::size_of_val(&self.spectra);
        for spectrum in &self.spectra {
            total_size += std::mem::size_of_val(spectrum);
        }
        
        // 估算 curves 大小
        total_size += std::mem::size_of_val(&self.curves);
        for curve in &self.curves {
            total_size += std::mem::size_of_val(curve);
            // 估算曲线数据点大小
            total_size += curve.x_values.len() * std::mem::size_of::<f64>();
            total_size += curve.y_values.len() * std::mem::size_of::<f64>();
            
            // 估算 peaks 大小
            for peak in &curve.peaks {
                total_size += std::mem::size_of_val(peak);
                total_size += peak.fit_parameters.len() * std::mem::size_of::<f64>();
                total_size += peak.fit_parameter_errors.len() * std::mem::size_of::<f64>();
                if let Some(ref matrix) = peak.fit_covariance_matrix {
                    for row in matrix {
                        total_size += row.len() * std::mem::size_of::<f64>();
                    }
                }
            }
        }
        
        total_size
    }
    
    /// Get memory usage in human-readable format
    /// 获取人类可读的内存使用格式
    pub fn get_memory_usage_string(&self) -> String {
        let bytes = self.estimate_memory_usage();
        
        if bytes < 1024 {
            format!("{} B", bytes)
        } else if bytes < 1024 * 1024 {
            format!("{:.2} KB", bytes as f64 / 1024.0)
        } else if bytes < 1024 * 1024 * 1024 {
            format!("{:.2} MB", bytes as f64 / (1024.0 * 1024.0))
        } else {
            format!("{:.2} GB", bytes as f64 / (1024.0 * 1024.0 * 1024.0))
        }
    }
    
    /// Check if container is too large for processing
    /// 检查容器是否过大，需要分块处理
    pub fn is_too_large(&self, max_memory_mb: usize) -> bool {
        let memory_mb = self.estimate_memory_usage() / (1024 * 1024);
        memory_mb > max_memory_mb
    }
    
    /// Get processing recommendations based on container size
    /// 根据容器大小获取处理建议
    pub fn get_processing_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let memory_mb = self.estimate_memory_usage() / (1024 * 1024);
        
        if memory_mb > 100 {
            recommendations.push("建议使用分块处理模式".to_string());
        }
        
        if self.spectra.len() > 1000 {
            recommendations.push("建议清理已处理的光谱数据以释放内存".to_string());
        }
        
        if self.total_peak_count() > 10000 {
            recommendations.push("峰数量较多，建议使用批量处理".to_string());
        }
        
        if recommendations.is_empty() {
            recommendations.push("当前数据大小适中，可以正常处理".to_string());
        }
        
        recommendations
    }
}
