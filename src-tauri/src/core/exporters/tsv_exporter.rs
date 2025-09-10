use async_trait::async_trait;
use serde_json::Value;
use crate::core::data::{DataContainer, ProcessingError, PeakType, DetectionAlgorithm, Peak, Curve};
use super::base::{Exporter, ExportResult, ExportConfig, helpers};

/// TSV (Tab-Separated Values) exporter for mass spectrometry data
pub struct TsvExporter;

#[async_trait]
impl Exporter for TsvExporter {
    fn name(&self) -> &str {
        "tsv_exporter"
    }

    fn description(&self) -> &str {
        "Export mass spectrometry data to TSV format with comprehensive peak and curve information"
    }

    fn file_extension(&self) -> &str {
        "tsv"
    }

    fn mime_type(&self) -> &str {
        "text/tab-separated-values"
    }

    fn config_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "include_header": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include header row in the export"
                },
                "decimal_precision": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 15,
                    "default": 6,
                    "description": "Decimal precision for numeric values"
                },
                "include_metadata": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include metadata in the export"
                },
                "include_curves": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include curve data in the export"
                },
                "include_peaks": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include peak data in the export"
                },
                "export_format": {
                    "type": "string",
                    "enum": ["peaks_only", "curves_only", "combined", "summary", "fitted_curves"],
                    "default": "combined",
                    "description": "Export format type"
                },
                "include_fitted_curves": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include fitted peak curves for visualization"
                },
                "fitted_curve_points": {
                    "type": "integer",
                    "minimum": 10,
                    "maximum": 1000,
                    "default": 100,
                    "description": "Number of points for fitted curves"
                }
            }
        })
    }

    async fn export(
        &self,
        data: &DataContainer,
        config: Value,
    ) -> Result<ExportResult, ProcessingError> {
        let export_format = config["export_format"]
            .as_str()
            .unwrap_or("combined")
            .to_string();
        
        let export_config: ExportConfig = serde_json::from_value(config)
            .unwrap_or_default();
        
        let content = match export_format.as_str() {
            "peaks_only" => self.export_peaks_only(data, &export_config)?,
            "curves_only" => self.export_curves_only(data, &export_config)?,
            "combined" => self.export_combined(data, &export_config)?,
            "summary" => self.export_summary(data, &export_config)?,
            "fitted_curves" => self.export_fitted_curves(data, &export_config)?,
            _ => {
                return Err(ProcessingError::ConfigError(
                    format!("Unsupported export format: {}", export_format)
                ));
            }
        };
        
        let filename = format!("ims_data_{}.tsv", helpers::generate_timestamp());
        let metadata = helpers::create_export_metadata(
            self.name(),
            data.curves.len(),
            data.peaks.len(),
            &export_config,
        );
        
        Ok(ExportResult {
            data: content.into_bytes(),
            filename,
            mime_type: self.mime_type().to_string(),
            metadata,
        })
    }
}

impl TsvExporter {
    /// Export peaks only
    fn export_peaks_only(&self, data: &DataContainer, config: &ExportConfig) -> Result<String, ProcessingError> {
        let mut content = String::new();
        
        if config.include_header {
            content.push_str("Peak_ID\tCurve_ID\tCenter\tAmplitude\tArea\tFWHM\tHWHM\tSigma\tGamma\t");
            content.push_str("Left_HWHM\tRight_HWHM\tAsymmetry_Factor\tLeft_Boundary\tRight_Boundary\tPeak_Span\t");
            content.push_str("R_Squared\tResidual_Sum_Squares\tStandard_Error\tParameter_Count\tPeak_Type\t");
            content.push_str("Mixing_Parameter\tSignal_to_Baseline_Ratio\tArea_Percentage\tIntensity_Percentage\t");
            content.push_str("Left_Derivative\tRight_Derivative\tDerivative_Ratio\tMZ\tRetention_Time\t");
            content.push_str("Drift_Time\tMS_Level\tDetection_Algorithm\tDetection_Threshold\tConfidence\t");
            content.push_str("Fit_Parameters\tFit_Parameter_Errors\n");
        }
        
        for peak in &data.peaks {
            content.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t",
                peak.id,
                peak.curve_id,
                helpers::format_float(peak.center, config.decimal_precision),
                helpers::format_float(peak.amplitude, config.decimal_precision),
                helpers::format_float(peak.area, config.decimal_precision),
                helpers::format_float(peak.fwhm, config.decimal_precision),
                helpers::format_float(peak.hwhm, config.decimal_precision),
                helpers::format_float(peak.sigma, config.decimal_precision),
                helpers::format_float(peak.gamma, config.decimal_precision),
            ));
            
            content.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\t",
                helpers::format_float(peak.left_hwhm, config.decimal_precision),
                helpers::format_float(peak.right_hwhm, config.decimal_precision),
                helpers::format_float(peak.asymmetry_factor, config.decimal_precision),
                helpers::format_float(peak.left_boundary, config.decimal_precision),
                helpers::format_float(peak.right_boundary, config.decimal_precision),
                helpers::format_float(peak.peak_span, config.decimal_precision),
            ));
            
            content.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\t",
                helpers::format_float(peak.rsquared, config.decimal_precision),
                helpers::format_float(peak.residual_sum_squares, config.decimal_precision),
                helpers::format_float(peak.standard_error, config.decimal_precision),
                peak.parameter_count,
                self.format_peak_type(&peak.peak_type),
                helpers::format_float(peak.mixing_parameter, config.decimal_precision),
            ));
            
            content.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
                helpers::format_float(peak.signal_to_baseline_ratio, config.decimal_precision),
                helpers::format_float(peak.area_percentage, config.decimal_precision),
                helpers::format_float(peak.intensity_percentage, config.decimal_precision),
                helpers::format_float(peak.left_derivative, config.decimal_precision),
                helpers::format_float(peak.right_derivative, config.decimal_precision),
                helpers::format_float(peak.derivative_ratio, config.decimal_precision),
                peak.mz.map(|v| helpers::format_float(v, config.decimal_precision)).unwrap_or("".to_string()),
                peak.retention_time.map(|v| helpers::format_float(v, config.decimal_precision)).unwrap_or("".to_string()),
                peak.drift_time.map(|v| helpers::format_float(v, config.decimal_precision)).unwrap_or("".to_string()),
                peak.ms_level.map(|v| v.to_string()).unwrap_or("".to_string()),
                self.format_detection_algorithm(&peak.detection_algorithm),
                helpers::format_float(peak.detection_threshold, config.decimal_precision),
                helpers::format_float(peak.confidence, config.decimal_precision)
            ));
            
            // Fit parameters
            let fit_params = peak.fit_parameters.iter()
                .map(|p| helpers::format_float(*p, config.decimal_precision))
                .collect::<Vec<_>>()
                .join(",");
            let fit_errors = peak.fit_parameter_errors.iter()
                .map(|e| helpers::format_float(*e, config.decimal_precision))
                .collect::<Vec<_>>()
                .join(",");
            
            content.push_str(&format!("{}\t{}\n", fit_params, fit_errors));
        }
        
        Ok(content)
    }
    
    /// Export curves only
    fn export_curves_only(&self, data: &DataContainer, config: &ExportConfig) -> Result<String, ProcessingError> {
        let mut content = String::new();
        
        if config.include_header {
            content.push_str("Curve_ID\tCurve_Type\tX_Label\tY_Label\tX_Unit\tY_Unit\t");
            content.push_str("X_Min\tX_Max\tY_Min\tY_Max\tPoint_Count\tTotal_Ion_Current\t");
            content.push_str("Mean_Intensity\tIntensity_Std\tBaseline_Intensity\tSignal_to_Noise_Ratio\t");
            content.push_str("MZ_Range_Min\tMZ_Range_Max\tRT_Range_Min\tRT_Range_Max\tDT_Range_Min\tDT_Range_Max\t");
            content.push_str("MS_Level\tSmoothing_Factor\tBaseline_Correction\tNoise_Level\t");
            content.push_str("Detection_Threshold\tQuality_Score\tCompleteness\tHas_Missing_Points\n");
        }
        
        for curve in &data.curves {
            content.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\t",
                curve.id,
                curve.curve_type,
                curve.x_label,
                curve.y_label,
                curve.x_unit,
                curve.y_unit,
            ));
            
            content.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\t",
                helpers::format_float(curve.x_min, config.decimal_precision),
                helpers::format_float(curve.x_max, config.decimal_precision),
                helpers::format_float(curve.y_min, config.decimal_precision),
                helpers::format_float(curve.y_max, config.decimal_precision),
                curve.point_count,
                helpers::format_float(curve.total_ion_current, config.decimal_precision),
            ));
            
            content.push_str(&format!("{}\t{}\t{}\t{}\t",
                helpers::format_float(curve.mean_intensity, config.decimal_precision),
                helpers::format_float(curve.intensity_std, config.decimal_precision),
                helpers::format_float(curve.baseline_intensity, config.decimal_precision),
                helpers::format_float(curve.signal_to_noise_ratio, config.decimal_precision),
            ));
            
            // Ranges
            let mz_range = curve.mz_range.map(|(min, max)| 
                format!("{}\t{}", helpers::format_float(min, config.decimal_precision), helpers::format_float(max, config.decimal_precision))
            ).unwrap_or("\t".to_string());
            let rt_range = curve.rt_range.map(|(min, max)| 
                format!("{}\t{}", helpers::format_float(min, config.decimal_precision), helpers::format_float(max, config.decimal_precision))
            ).unwrap_or("\t".to_string());
            let dt_range = curve.dt_range.map(|(min, max)| 
                format!("{}\t{}", helpers::format_float(min, config.decimal_precision), helpers::format_float(max, config.decimal_precision))
            ).unwrap_or("\t".to_string());
            
            content.push_str(&format!("{}\t{}\t{}\t",
                mz_range,
                rt_range,
                dt_range,
            ));
            
            content.push_str(&format!("{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
                curve.ms_level.map(|v| v.to_string()).unwrap_or("".to_string()),
                curve.smoothing_factor.map(|v| helpers::format_float(v, config.decimal_precision)).unwrap_or("".to_string()),
                curve.baseline_correction.as_ref().unwrap_or(&"".to_string()),
                helpers::format_float(curve.noise_level, config.decimal_precision),
                helpers::format_float(curve.detection_threshold, config.decimal_precision),
                helpers::format_float(curve.quality_score, config.decimal_precision),
                helpers::format_float(curve.completeness, config.decimal_precision),
                curve.has_missing_points,
            ));
        }
        
        Ok(content)
    }
    
    /// Export combined data
    fn export_combined(&self, data: &DataContainer, config: &ExportConfig) -> Result<String, ProcessingError> {
        let mut content = String::new();
        
        // Add metadata section
        if config.include_metadata {
            content.push_str("# IMS Data Export\n");
            content.push_str(&format!("# Export Time: {}\n", helpers::generate_timestamp()));
            content.push_str(&format!("# Curves: {}\n", data.curves.len()));
            content.push_str(&format!("# Peaks: {}\n", data.peaks.len()));
            content.push_str("#\n");
        }
        
        // Export curves
        if config.include_curves {
            content.push_str("# === CURVES ===\n");
            content.push_str(&self.export_curves_only(data, config)?);
            content.push_str("\n");
        }
        
        // Export peaks
        if config.include_peaks {
            content.push_str("# === PEAKS ===\n");
            content.push_str(&self.export_peaks_only(data, config)?);
            content.push_str("\n");
        }
        
        // Export fitted curves for visualization
        if config.include_fitted_curves.unwrap_or(false) {
            content.push_str("# === FITTED CURVES FOR VISUALIZATION ===\n");
            content.push_str(&self.export_fitted_curves(data, config)?);
        }
        
        Ok(content)
    }
    
    /// Export summary data
    fn export_summary(&self, data: &DataContainer, config: &ExportConfig) -> Result<String, ProcessingError> {
        let mut content = String::new();
        
        if config.include_header {
            content.push_str("Metric\tValue\n");
        }
        
        // Basic statistics
        content.push_str(&format!("Total_Curves\t{}\n", data.curves.len()));
        content.push_str(&format!("Total_Peaks\t{}\n", data.peaks.len()));
        
        if !data.curves.is_empty() {
            let total_points: usize = data.curves.iter().map(|c| c.point_count).sum();
            content.push_str(&format!("Total_Data_Points\t{}\n", total_points));
            
            let avg_curve_length = total_points as f64 / data.curves.len() as f64;
            content.push_str(&format!("Average_Curve_Length\t{}\n", 
                helpers::format_float(avg_curve_length, config.decimal_precision)));
        }
        
        if !data.peaks.is_empty() {
            let avg_amplitude: f64 = data.peaks.iter().map(|p| p.amplitude).sum::<f64>() / data.peaks.len() as f64;
            let avg_fwhm: f64 = data.peaks.iter().map(|p| p.fwhm).sum::<f64>() / data.peaks.len() as f64;
            let avg_rsquared: f64 = data.peaks.iter().map(|p| p.rsquared).sum::<f64>() / data.peaks.len() as f64;
            
            content.push_str(&format!("Average_Peak_Amplitude\t{}\n", 
                helpers::format_float(avg_amplitude, config.decimal_precision)));
            content.push_str(&format!("Average_Peak_FWHM\t{}\n", 
                helpers::format_float(avg_fwhm, config.decimal_precision)));
            content.push_str(&format!("Average_Peak_R_Squared\t{}\n", 
                helpers::format_float(avg_rsquared, config.decimal_precision)));
        }
        
        Ok(content)
    }
    
    /// Export fitted curves for visualization
    fn export_fitted_curves(&self, data: &DataContainer, config: &ExportConfig) -> Result<String, ProcessingError> {
        let mut content = String::new();
        
        if config.include_header {
            content.push_str("Curve_Type\tCurve_ID\tX_Value\tY_Value\tPeak_ID\n");
        }
        
        for curve in &data.curves {
            // 导出原始曲线
            for (_i, (&x, &y)) in curve.x_values.iter().zip(curve.y_values.iter()).enumerate() {
                content.push_str(&format!("Original\t{}\t{}\t{}\t\n",
                    curve.id,
                    helpers::format_float(x, config.decimal_precision),
                    helpers::format_float(y, config.decimal_precision)
                ));
            }
            
            // 导出每个峰的拟合曲线
            let curve_peaks: Vec<&Peak> = data.peaks.iter()
                .filter(|peak| peak.curve_id == curve.id)
                .collect();
            
            for peak in curve_peaks {
                let fitted_curve = self.generate_fitted_curve(peak, curve, config)?;
                for (x, y) in fitted_curve {
                    content.push_str(&format!("Fitted\t{}\t{}\t{}\t{}\n",
                        curve.id,
                        helpers::format_float(x, config.decimal_precision),
                        helpers::format_float(y, config.decimal_precision),
                        peak.id
                    ));
                }
            }
        }
        
        Ok(content)
    }
    
    /// Generate fitted curve points for a peak
    fn generate_fitted_curve(&self, peak: &Peak, _curve: &Curve, config: &ExportConfig) -> Result<Vec<(f64, f64)>, ProcessingError> {
        let num_points = config.fitted_curve_points.unwrap_or(100);
        let mut fitted_points = Vec::new();
        
        // 计算拟合曲线的范围（峰中心 ± 3*sigma）
        let range = peak.sigma * 6.0;
        let start_x = peak.center - range / 2.0;
        let end_x = peak.center + range / 2.0;
        
        for i in 0..num_points {
            let x = start_x + (end_x - start_x) * i as f64 / (num_points - 1) as f64;
            let y = self.calculate_fitted_y(x, peak)?;
            fitted_points.push((x, y));
        }
        
        Ok(fitted_points)
    }
    
    /// Calculate fitted Y value for a given X using peak parameters
    fn calculate_fitted_y(&self, x: f64, peak: &Peak) -> Result<f64, ProcessingError> {
        match peak.peak_type {
            PeakType::Gaussian => {
                // Gaussian: y = A * exp(-0.5 * ((x - center) / sigma)^2)
                let amplitude = peak.amplitude;
                let center = peak.center;
                let sigma = peak.sigma;
                
                if sigma <= 0.0 {
                    return Err(ProcessingError::ProcessError("Invalid sigma value".to_string()));
                }
                
                let exponent = -0.5 * ((x - center) / sigma).powi(2);
                Ok(amplitude * exponent.exp())
            },
            PeakType::Lorentzian => {
                // Lorentzian: y = A / (1 + ((x - center) / gamma)^2)
                let amplitude = peak.amplitude;
                let center = peak.center;
                let gamma = peak.gamma.max(peak.sigma); // Use gamma or fallback to sigma
                
                if gamma <= 0.0 {
                    return Err(ProcessingError::ProcessError("Invalid gamma value".to_string()));
                }
                
                Ok(amplitude / (1.0 + ((x - center) / gamma).powi(2)))
            },
            PeakType::PseudoVoigt => {
                // Pseudo-Voigt: y = A * (m * L + (1-m) * G)
                let amplitude = peak.amplitude;
                let center = peak.center;
                let sigma = peak.sigma;
                let mixing = peak.mixing_parameter;
                
                if sigma <= 0.0 {
                    return Err(ProcessingError::ProcessError("Invalid sigma value".to_string()));
                }
                
                // Gaussian component
                let gaussian = (-0.5 * ((x - center) / sigma).powi(2)).exp();
                
                // Lorentzian component
                let lorentzian = 1.0 / (1.0 + ((x - center) / sigma).powi(2));
                
                Ok(amplitude * (mixing * lorentzian + (1.0 - mixing) * gaussian))
            },
            _ => {
                // For other peak types, use Gaussian as approximation
                let amplitude = peak.amplitude;
                let center = peak.center;
                let sigma = peak.sigma;
                
                if sigma <= 0.0 {
                    return Err(ProcessingError::ProcessError("Invalid sigma value".to_string()));
                }
                
                let exponent = -0.5 * ((x - center) / sigma).powi(2);
                Ok(amplitude * exponent.exp())
            }
        }
    }
    
    /// Format peak type for export
    fn format_peak_type(&self, peak_type: &PeakType) -> String {
        match peak_type {
            PeakType::Gaussian => "Gaussian".to_string(),
            PeakType::Lorentzian => "Lorentzian".to_string(),
            PeakType::PseudoVoigt => "PseudoVoigt".to_string(),
            PeakType::AsymmetricGaussian => "AsymmetricGaussian".to_string(),
            PeakType::Custom(name) => format!("Custom({})", name),
            PeakType::EMG => "EMG".to_string(),
            PeakType::BiGaussian => "BiGaussian".to_string(),
            PeakType::VoigtExponentialTail => "VoigtExponentialTail".to_string(),
            PeakType::PearsonIV => "PearsonIV".to_string(),
            PeakType::NLC => "NLC".to_string(),
            PeakType::GMGBayesian => "GMGBayesian".to_string(),
        }
    }
    
    /// Format detection algorithm for export
    fn format_detection_algorithm(&self, algorithm: &DetectionAlgorithm) -> String {
        match algorithm {
            DetectionAlgorithm::CWT => "CWT".to_string(),
            DetectionAlgorithm::PeakFinder => "PeakFinder".to_string(),
            DetectionAlgorithm::Simple => "Simple".to_string(),
            DetectionAlgorithm::SavitzkyGolay => "SavitzkyGolay".to_string(),
            DetectionAlgorithm::Custom(name) => format!("Custom({})", name),
        }
    }
}
