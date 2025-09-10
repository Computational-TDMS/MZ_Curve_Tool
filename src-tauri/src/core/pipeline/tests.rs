//! 流水线模块测试

#[cfg(test)]
mod tests {
    use crate::core::pipeline::PipelineManager;
    use crate::core::data::container::SerializableDataContainer;
    use crate::core::data::{Curve, Peak, PeakType};
    use serde_json::json;

    #[test]
    fn test_serializable_data_container_creation() {
        let container = SerializableDataContainer::new();
        assert_eq!(container.curve_count(), 0);
        assert_eq!(container.peak_count(), 0);
    }

    #[test]
    fn test_serializable_data_container_conversion() {
        let mut container = SerializableDataContainer::new();
        
        // 添加测试曲线
        let curve = Curve::new(
            "test_curve".to_string(),
            "DT".to_string(),
            vec![1.0, 2.0, 3.0],
            vec![10.0, 20.0, 15.0],
            "Time".to_string(),
            "Intensity".to_string(),
            "ms".to_string(),
            "counts".to_string(),
        );
        container.add_curve(curve);
        
        // 添加测试峰
        let peak = Peak::new(
            "test_peak".to_string(),
            "test_curve".to_string(),
            2.0,
            20.0,
            PeakType::Gaussian,
        );
        container.add_peak(peak);
        
        assert_eq!(container.curve_count(), 1);
        assert_eq!(container.peak_count(), 1);
        
        // 测试转换为DataContainer
        let data_container = container.to_data_container();
        assert_eq!(data_container.curves.len(), 1);
        assert_eq!(data_container.peaks.len(), 1);
        assert_eq!(data_container.spectra.len(), 0); // 应该为空
    }

    #[test]
    fn test_pipeline_manager_creation() {
        let _pipeline = PipelineManager::new();
        // 这里可以添加更多测试，但需要异步运行时
    }

    #[test]
    fn test_pipeline_step_configuration() {
        let config = json!({
            "sensitivity": 0.7,
            "threshold_multiplier": 3.0,
            "min_peak_width": 0.1,
            "max_peak_width": 10.0
        });
        
        let _pipeline = PipelineManager::new()
            .add_peak_detection("cwt", config.clone())
            .add_peak_fitting("gaussian", config.clone());
        
        // 验证pipeline创建成功
        // 注意：这里无法直接测试execute方法，因为它需要异步运行时
    }

    #[test]
    fn test_metadata_operations() {
        let mut container = SerializableDataContainer::new();
        
        // 测试添加元数据
        container.add_metadata("test_key".to_string(), json!("test_value"));
        
        // 测试获取元数据
        let value = container.get_metadata("test_key");
        assert!(value.is_some());
        assert_eq!(value.unwrap(), &json!("test_value"));
        
        // 测试获取不存在的元数据
        let missing_value = container.get_metadata("missing_key");
        assert!(missing_value.is_none());
    }

    #[test]
    fn test_container_clear() {
        let mut container = SerializableDataContainer::new();
        
        // 添加一些数据
        let curve = Curve::new(
            "test_curve".to_string(),
            "DT".to_string(),
            vec![1.0, 2.0, 3.0],
            vec![10.0, 20.0, 15.0],
            "Time".to_string(),
            "Intensity".to_string(),
            "ms".to_string(),
            "counts".to_string(),
        );
        container.add_curve(curve);
        
        let peak = Peak::new(
            "test_peak".to_string(),
            "test_curve".to_string(),
            2.0,
            20.0,
            PeakType::Gaussian,
        );
        container.add_peak(peak);
        
        container.add_metadata("test_key".to_string(), json!("test_value"));
        
        // 验证数据已添加
        assert_eq!(container.curve_count(), 1);
        assert_eq!(container.peak_count(), 1);
        assert!(container.get_metadata("test_key").is_some());
        
        // 清空数据
        container.clear();
        
        // 验证数据已清空
        assert_eq!(container.curve_count(), 0);
        assert_eq!(container.peak_count(), 0);
        assert!(container.get_metadata("test_key").is_none());
    }
}
