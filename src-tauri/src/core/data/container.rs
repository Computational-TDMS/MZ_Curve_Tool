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
    pub peaks: Vec<Peak>,
}

/// 用于序列化的数据容器，不包含复杂的 mzdata 类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SerializableDataContainer {
    pub metadata: HashMap<String, serde_json::Value>,
    pub spectra: Vec<serde_json::Value>, // 简化的光谱数据
    pub curves: Vec<Curve>,
    pub peaks: Vec<Peak>,
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
            peaks: container.peaks,
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
            peaks: container.peaks,
        }
    }
}

impl Default for SerializableDataContainer {
    fn default() -> Self {
        Self {
            metadata: HashMap::new(),
            spectra: Vec::new(),
            curves: Vec::new(),
            peaks: Vec::new(),
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
    
    /// Add a peak to the container
    pub fn add_peak(&mut self, peak: Peak) {
        self.peaks.push(peak);
    }
    
    /// Get the number of curves
    pub fn curve_count(&self) -> usize {
        self.curves.len()
    }
    
    /// Get the number of peaks
    pub fn peak_count(&self) -> usize {
        self.peaks.len()
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
        self.peaks.clear();
    }
}

impl Default for DataContainer {
    fn default() -> Self {
        Self {
            metadata: HashMap::new(),
            spectra: Vec::new(),
            curves: Vec::new(),
            peaks: Vec::new(),
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
    
    /// Add a peak to the container
    pub fn add_peak(&mut self, peak: Peak) {
        self.peaks.push(peak);
    }
    
    /// Get the number of curves
    pub fn curve_count(&self) -> usize {
        self.curves.len()
    }
    
    /// Get the number of peaks
    pub fn peak_count(&self) -> usize {
        self.peaks.len()
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
        self.peaks.clear();
    }
    
    /// Get curves by type
    pub fn get_curves_by_type(&self, curve_type: &str) -> Vec<&Curve> {
        self.curves.iter()
            .filter(|curve| curve.curve_type == curve_type)
            .collect()
    }
    
    /// Get peaks by curve ID
    pub fn get_peaks_by_curve_id(&self, curve_id: &str) -> Vec<&Peak> {
        self.peaks.iter()
            .filter(|peak| peak.curve_id == curve_id)
            .collect()
    }
    
    /// Add metadata
    pub fn add_metadata(&mut self, key: String, value: serde_json::Value) {
        self.metadata.insert(key, value);
    }
    
    /// Get metadata value
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }
}
