use async_trait::async_trait;
use serde_json::Value;
use std::collections::HashMap;
use uuid::Uuid;

use crate::core::data::{DataContainer, Curve, ProcessingError, ProcessingResult};
use crate::core::loaders::mzdata_loader::DataLoader;
use crate::core::processors::base::Processor;
use mzdata::prelude::{SpectrumLike, MZLocated, IntensityMeasurement};

/// DT提取器 - 专门负责DT曲线数据提取，不进行峰值检测
#[derive(Debug)]
pub struct DTExtractor;

#[async_trait]
impl Processor for DTExtractor {
    fn name(&self) -> &str {
        "dt_extractor"
    }

    fn description(&self) -> &str {
        "DT曲线提取器，专门负责离子迁移率数据的提取和曲线生成"
    }

    fn config_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "mz_range": {
                    "type": "string",
                    "pattern": "^[0-9]+(\\.[0-9]+)?-[0-9]+(\\.[0-9]+)?$",
                    "description": "m/z范围，格式：min-max"
                },
                "rt_range": {
                    "type": "string", 
                    "pattern": "^[0-9]+(\\.[0-9]+)?-[0-9]+(\\.[0-9]+)?$",
                    "description": "保留时间范围，格式：min-max"
                },
                "ms_level": {
                    "type": "integer",
                    "minimum": 1,
                    "description": "MS级别"
                },
            },
            "required": ["mz_range", "rt_range", "ms_level"]
        })
    }

    async fn process(
        &self,
        mut input: DataContainer,
        config: Value,
    ) -> Result<ProcessingResult, ProcessingError> {
        // 解析配置
        let mz_range = config["mz_range"]
            .as_str()
            .ok_or_else(|| ProcessingError::ConfigError("mz_range missing".to_string()))?;
        let rt_range = config["rt_range"]
            .as_str()
            .ok_or_else(|| ProcessingError::ConfigError("rt_range missing".to_string()))?;
        let ms_level = config["ms_level"]
            .as_u64()
            .ok_or_else(|| ProcessingError::ConfigError("ms_level missing".to_string()))? as u8;

        let (mz_min, mz_max) = parse_range(mz_range)?;
        let (rt_min, rt_max) = parse_range(rt_range)?;

        // 过滤光谱
        let filtered_spectra = DataLoader::filter_spectra(
            &input.spectra,
            Some(ms_level),
            Some(rt_min),
            Some(rt_max),
            Some(mz_min),
            Some(mz_max),
        );

        if filtered_spectra.is_empty() {
            return Err(ProcessingError::DataError(
                "No spectra found in the specified range".to_string(),
            ));
        }

        // 生成DT曲线
        let dt_curve = self.generate_dt_curve(&filtered_spectra, mz_min, mz_max)?;

        // 添加到数据容器
        input.curves.push(dt_curve.clone());

        // DT提取器不进行峰值检测，只返回曲线数据
        Ok(ProcessingResult {
            curves: vec![dt_curve],
            peaks: Vec::new(), // 不进行峰值检测
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("mz_range".to_string(), serde_json::json!([mz_min, mz_max]));
                meta.insert("rt_range".to_string(), serde_json::json!([rt_min, rt_max]));
                meta.insert("ms_level".to_string(), serde_json::json!(ms_level));
                meta.insert("spectra_count".to_string(), serde_json::json!(filtered_spectra.len()));
                meta
            },
        })
    }
}

impl DTExtractor {
    /// 生成DT曲线
    fn generate_dt_curve(
        &self,
        spectra: &[&mzdata::spectrum::Spectrum],
        mz_min: f64,
        mz_max: f64,
    ) -> Result<Curve, ProcessingError> {
        let mut dt_data: HashMap<u64, f64> = HashMap::new();

        for spectrum in spectra {
            // 使用正确的API获取离子迁移率数据
            if let Some(ion_mobility) = spectrum.ion_mobility() {
                let dt_key = (ion_mobility * 1000.0) as u64; // 精确到毫秒

                let peaks = spectrum.peaks();
                
                // 累加指定m/z范围内的强度
                for peak in peaks.iter() {
                    let mz = peak.mz();
                    if mz >= mz_min && mz <= mz_max {
                        *dt_data.entry(dt_key).or_insert(0.0) += peak.intensity() as f64;
                    }
                }
            }
        }

        if dt_data.is_empty() {
            return Err(ProcessingError::DataError(
                "No ion mobility data found in the specified range".to_string(),
            ));
        }

        // 排序并生成曲线数据
        let mut sorted_data: Vec<(u64, f64)> = dt_data.into_iter().collect();
        sorted_data.sort_by(|a, b| a.0.cmp(&b.0));
        
        let x_values: Vec<f64> = sorted_data.iter().map(|(k, _)| *k as f64 / 1000.0).collect();
        let y_values: Vec<f64> = sorted_data.iter().map(|(_, v)| *v).collect();

        let mut curve = Curve::new(
            format!("dt_curve_{}", Uuid::new_v4()),
            "DT".to_string(),
            x_values,
            y_values,
            "Drift Time".to_string(),
            "Intensity".to_string(),
            "ms".to_string(),
            "counts".to_string(),
        );
        
        curve.set_mz_range(mz_min, mz_max);
        curve.metadata.insert("data_points".to_string(), serde_json::json!(curve.point_count));
        
        Ok(curve)
    }

}

/// 解析范围字符串
fn parse_range(range_str: &str) -> Result<(f64, f64), ProcessingError> {
    let parts: Vec<&str> = range_str.split('-').collect();
    if parts.len() != 2 {
        return Err(ProcessingError::ConfigError(format!(
            "无效的范围格式: {}",
            range_str
        )));
    }

    let min = parts[0]
        .parse::<f64>()
        .map_err(|_| ProcessingError::ConfigError(format!("无效的数字: {}", parts[0])))?;
    let max = parts[1]
        .parse::<f64>()
        .map_err(|_| ProcessingError::ConfigError(format!("无效的数字: {}", parts[1])))?;

    Ok((min, max))
}
