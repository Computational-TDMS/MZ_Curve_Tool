//! 峰分析使用示例
//! 
//! 展示如何使用新的pipeline架构进行峰分析

use crate::core::data::{DataContainer, Curve, CurveType};
use crate::core::pipeline::{PipelineManager, SerializableDataContainer};
use serde_json::json;
use std::collections::HashMap;

/// 示例1: 基本峰分析流水线
pub async fn example_basic_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== 示例1: 基本峰分析流水线 ===");
    
    // 创建测试数据
    let test_data = create_test_data();
    let serializable_data = SerializableDataContainer::from_data_container(&test_data);
    
    // 创建流水线管理器
    let pipeline = PipelineManager::new()
        .add_peak_detection("cwt", json!({
            "sensitivity": 0.7,
            "threshold_multiplier": 3.0,
            "min_peak_width": 0.1,
            "max_peak_width": 10.0
        }))
        .add_peak_fitting("gaussian", json!({
            "min_peak_width": 0.1,
            "max_peak_width": 10.0,
            "fit_quality_threshold": 0.8
        }));
    
    // 执行流水线
    let result = pipeline.execute(serializable_data).await?;
    
    println!("检测到 {} 个峰", result.container.peak_count());
    println!("处理了 {} 条曲线", result.container.curve_count());
    println!("执行时间: {}ms", result.execution_time);
    println!("完成的步骤: {:?}", result.steps_completed);
    
    Ok(())
}

/// 示例2: 完整峰分析流水线
pub async fn example_complete_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 示例2: 完整峰分析流水线 ===");
    
    let test_data = create_test_data();
    let serializable_data = SerializableDataContainer::from_data_container(&test_data);
    
    // 创建完整的流水线
    let pipeline = PipelineManager::new()
        .add_peak_detection("cwt", json!({
            "sensitivity": 0.7,
            "threshold_multiplier": 3.0
        }))
        .add_peak_fitting("gaussian", json!({
            "fit_quality_threshold": 0.8
        }))
        .add_peak_enhancement("default", json!({
            "quality_threshold": 0.5,
            "boundary_method": "adaptive",
            "separation_analysis": true
        }))
        .add_curve_reconstruction("default", json!({
            "resolution": 100,
            "include_baseline": true,
            "include_individual_peaks": true
        }));
    
    // 执行完整流水线
    let result = pipeline.execute(serializable_data).await?;
    
    println!("完整流水线执行完成");
    println!("检测到 {} 个峰", result.container.peak_count());
    println!("执行时间: {}ms", result.execution_time);
    println!("完成的步骤: {:?}", result.steps_completed);
    
    // 输出峰信息
    for (i, peak) in result.container.peaks.iter().take(3).enumerate() {
        println!("峰 {}: ID={}, 中心={:.2}, 振幅={:.2}", 
                i + 1, peak.id, peak.center, peak.amplitude);
    }
    
    Ok(())
}

/// 示例3: 不同检测方法比较
pub async fn example_detection_methods_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 示例3: 不同检测方法比较 ===");
    
    let test_data = create_test_data();
    
    // 测试不同的检测方法
    let methods = vec![
        ("cwt", "连续小波变换"),
        ("simple", "简单峰值检测"),
        ("peak_finder", "峰值查找器")
    ];
    
    for (method, description) in methods {
        println!("\n--- 使用 {} ({}) ---", description, method);
        
        let serializable_data = SerializableDataContainer::from_data_container(&test_data);
        let pipeline = PipelineManager::new()
            .add_peak_detection(method, json!({
                "sensitivity": 0.7,
                "threshold_multiplier": 3.0
            }));
        
        let result = pipeline.execute(serializable_data).await?;
        println!("检测到 {} 个峰", result.container.peak_count());
        println!("执行时间: {}ms", result.execution_time);
    }
    
    Ok(())
}

/// 示例4: 不同拟合方法比较
pub async fn example_fitting_methods_comparison() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 示例4: 不同拟合方法比较 ===");
    
    let test_data = create_test_data();
    
    // 先进行峰检测
    let mut serializable_data = SerializableDataContainer::from_data_container(&test_data);
    let detection_pipeline = PipelineManager::new()
        .add_peak_detection("cwt", json!({
            "sensitivity": 0.7,
            "threshold_multiplier": 3.0
        }));
    
    let detection_result = detection_pipeline.execute(serializable_data).await?;
    serializable_data = detection_result.container;
    
    // 测试不同的拟合方法
    let methods = vec![
        ("gaussian", "高斯拟合"),
        ("lorentzian", "洛伦兹拟合"),
        ("pseudo_voigt", "伪Voigt拟合")
    ];
    
    for (method, description) in methods {
        println!("\n--- 使用 {} ({}) ---", description, method);
        
        let mut data_for_fitting = serializable_data.clone();
        let fitting_pipeline = PipelineManager::new()
            .add_peak_fitting(method, json!({
                "fit_quality_threshold": 0.8
            }));
        
        let result = fitting_pipeline.execute(data_for_fitting).await?;
        println!("拟合了 {} 个峰", result.container.peak_count());
        println!("执行时间: {}ms", result.execution_time);
        
        // 输出拟合质量信息
        let mut good_fits = 0;
        for peak in &result.container.peaks {
            if peak.rsquared > 0.8 {
                good_fits += 1;
            }
        }
        println!("高质量拟合峰数量: {}", good_fits);
    }
    
    Ok(())
}

/// 示例5: 基线校正流水线
pub async fn example_baseline_correction_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    println!("\n=== 示例5: 基线校正流水线 ===");
    
    let test_data = create_test_data();
    let serializable_data = SerializableDataContainer::from_data_container(&test_data);
    
    // 测试不同的基线校正方法
    let methods = vec![
        ("linear", "线性基线校正"),
        ("polynomial", "多项式基线校正"),
        ("moving_average", "移动平均基线校正"),
        ("asymmetric_least_squares", "非对称最小二乘基线校正")
    ];
    
    for (method, description) in methods {
        println!("\n--- 使用 {} ({}) ---", description, method);
        
        let pipeline = PipelineManager::new()
            .add_baseline_correction(method, json!({
                "method": method
            }));
        
        let result = pipeline.execute(serializable_data.clone()).await?;
        println!("校正了 {} 条曲线", result.container.curve_count());
        println!("执行时间: {}ms", result.execution_time);
    }
    
    Ok(())
}

/// 创建测试数据
fn create_test_data() -> DataContainer {
    let mut container = DataContainer::new();
    
    // 创建测试曲线
    let curve = create_test_curve();
    container.curves.push(curve);
    
    container
}

/// 创建测试曲线
fn create_test_curve() -> Curve {
    let mut x_values = Vec::new();
    let mut y_values = Vec::new();
    
    // 生成包含多个峰的测试数据
    for i in 0..200 {
        let x = 100.0 + i as f64 * 0.5;
        let mut y = 10.0; // 基线
        
        // 添加几个高斯峰
        y += 1000.0 * (-((x - 130.0).powi(2) / (2.0 * 5.0.powi(2)))).exp();
        y += 800.0 * (-((x - 150.0).powi(2) / (2.0 * 4.0.powi(2)))).exp();
        y += 600.0 * (-((x - 170.0).powi(2) / (2.0 * 6.0.powi(2)))).exp();
        
        x_values.push(x);
        y_values.push(y);
    }
    
    Curve {
        id: "test_curve".to_string(),
        curve_type: CurveType::MassSpectrum,
        x_values,
        y_values,
        x_label: "m/z".to_string(),
        y_label: "Intensity".to_string(),
        x_unit: "Da".to_string(),
        y_unit: "counts".to_string(),
        metadata: HashMap::new(),
    }
}

/// 运行所有示例
pub async fn run_all_examples() -> Result<(), Box<dyn std::error::Error>> {
    example_basic_pipeline().await?;
    example_complete_pipeline().await?;
    example_detection_methods_comparison().await?;
    example_fitting_methods_comparison().await?;
    example_baseline_correction_pipeline().await?;
    
    println!("\n=== 所有示例运行完成 ===");
    Ok(())
}