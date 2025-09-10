use mz_curve::data::{DataContainer, Curve};
use mz_curve::processors::baseline_correction::{
    BaselineProcessor, BaselineConfig, BaselineMethod,
    quick_baseline_correction
};
use serde_json::json;

/// 基线校准使用示例
pub async fn baseline_correction_example() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 基线校准使用示例 ===");
    
    // 创建示例数据
    let sample_curve = create_sample_curve_with_baseline_drift();
    let mut container = DataContainer::new();
    container.curves.push(sample_curve);
    
    println!("原始曲线数据点数: {}", container.curves[0].point_count);
    println!("原始基线强度: {:.2}", container.curves[0].baseline_intensity);
    
    // 示例1: 线性基线校准
    println!("\n--- 线性基线校准 ---");
    let linear_config = json!({
        "method": "linear",
        "preserve_original": true,
        "output_baseline": true
    });
    
    let processor = BaselineProcessor::new();
    let result = processor.process(container.clone(), linear_config).await?;
    
    println!("校准后曲线数: {}", result.data.curves.len());
    if let Some(corrected_curve) = result.data.curves.first() {
        println!("校准后基线强度: {:.2}", corrected_curve.baseline_intensity);
        println!("信噪比改善: {:.2}", corrected_curve.signal_to_noise_ratio);
    }
    
    // 示例2: 多项式基线校准
    println!("\n--- 多项式基线校准 (3次) ---");
    let polynomial_config = json!({
        "method": "polynomial",
        "degree": 3,
        "preserve_original": true,
        "output_baseline": true
    });
    
    let result = processor.process(container.clone(), polynomial_config).await?;
    if let Some(corrected_curve) = result.data.curves.first() {
        println!("校准后基线强度: {:.2}", corrected_curve.baseline_intensity);
        println!("信噪比改善: {:.2}", corrected_curve.signal_to_noise_ratio);
    }
    
    // 示例3: 移动平均基线校准
    println!("\n--- 移动平均基线校准 ---");
    let moving_avg_config = json!({
        "method": "moving_average",
        "window_size": 15,
        "preserve_original": true,
        "output_baseline": true,
        "custom_params": {
            "algorithm": "weighted"
        }
    });
    
    let result = processor.process(container.clone(), moving_avg_config).await?;
    if let Some(corrected_curve) = result.data.curves.first() {
        println!("校准后基线强度: {:.2}", corrected_curve.baseline_intensity);
        println!("信噪比改善: {:.2}", corrected_curve.signal_to_noise_ratio);
    }
    
    // 示例4: 非对称最小二乘法基线校准
    println!("\n--- 非对称最小二乘法基线校准 ---");
    let als_config = json!({
        "method": "asymmetric_least_squares",
        "lambda": 0.0,  // 自动选择
        "p": 0.0,       // 自动选择
        "max_iterations": 100,
        "preserve_original": true,
        "output_baseline": true
    });
    
    let result = processor.process(container.clone(), als_config).await?;
    if let Some(corrected_curve) = result.data.curves.first() {
        println!("校准后基线强度: {:.2}", corrected_curve.baseline_intensity);
        println!("信噪比改善: {:.2}", corrected_curve.signal_to_noise_ratio);
    }
    
    // 示例5: 快速基线校准
    println!("\n--- 快速基线校准 ---");
    let quick_result = quick_baseline_correction(container, "linear").await?;
    if let Some(corrected_curve) = quick_result.data.curves.first() {
        println!("快速校准后基线强度: {:.2}", corrected_curve.baseline_intensity);
    }
    
    println!("\n=== 基线校准示例完成 ===");
    Ok(())
}

/// 创建带有基线漂移的示例曲线
fn create_sample_curve_with_baseline_drift() -> Curve {
    let mut x_values = Vec::new();
    let mut y_values = Vec::new();
    
    // 生成时间轴 (0-100秒，0.1秒间隔)
    for i in 0..1000 {
        let t = i as f64 * 0.1;
        x_values.push(t);
        
        // 创建带有基线漂移的信号
        let baseline_drift = 100.0 + 50.0 * (t / 100.0).sin(); // 基线漂移
        let signal = if t > 20.0 && t < 30.0 {
            1000.0 * (-((t - 25.0) / 2.0).powi(2)).exp() // 高斯峰
        } else if t > 50.0 && t < 70.0 {
            800.0 * (-((t - 60.0) / 3.0).powi(2)).exp() // 另一个峰
        } else {
            0.0
        };
        
        let noise = 10.0 * (2.0 * std::f64::consts::PI * t * 0.1).sin(); // 噪声
        y_values.push(baseline_drift + signal + noise);
    }
    
    Curve::new(
        "sample_curve_with_drift".to_string(),
        "TIC".to_string(),
        x_values,
        y_values,
        "Time".to_string(),
        "Intensity".to_string(),
        "s".to_string(),
        "counts".to_string(),
    )
}

/// 比较不同基线校准方法的效果
pub async fn compare_baseline_methods() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 基线校准方法比较 ===");
    
    let sample_curve = create_sample_curve_with_baseline_drift();
    let mut container = DataContainer::new();
    container.curves.push(sample_curve);
    
    let methods = vec![
        ("linear", "线性基线校准"),
        ("polynomial", "多项式基线校准"),
        ("moving_average", "移动平均基线校准"),
        ("asymmetric_least_squares", "非对称最小二乘法"),
    ];
    
    let processor = BaselineProcessor::new();
    
    for (method, description) in methods {
        println!("\n--- {} ---", description);
        
        let config = json!({
            "method": method,
            "preserve_original": true,
            "output_baseline": false
        });
        
        let result = processor.process(container.clone(), config).await?;
        
        if let Some(corrected_curve) = result.data.curves.first() {
            println!("  校准后基线强度: {:.2}", corrected_curve.baseline_intensity);
            println!("  信噪比: {:.2}", corrected_curve.signal_to_noise_ratio);
            println!("  平均强度: {:.2}", corrected_curve.mean_intensity);
            println!("  强度标准差: {:.2}", corrected_curve.intensity_std);
        }
        
        // 显示处理统计信息
        if let Some(stats) = result.metadata.get("processing_stats") {
            if let Some(stats_array) = stats.as_array() {
                if let Some(first_stat) = stats_array.first() {
                    if let Some(quality_score) = first_stat.get("quality_score") {
                        println!("  质量评分: {:.3}", quality_score.as_f64().unwrap_or(0.0));
                    }
                    if let Some(processing_time) = first_stat.get("processing_time_ms") {
                        println!("  处理时间: {} ms", processing_time.as_u64().unwrap_or(0));
                    }
                }
            }
        }
    }
    
    println!("\n=== 方法比较完成 ===");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_baseline_correction() {
        let result = baseline_correction_example().await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_compare_methods() {
        let result = compare_baseline_methods().await;
        assert!(result.is_ok());
    }
}
