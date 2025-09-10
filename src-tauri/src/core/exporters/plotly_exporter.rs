use async_trait::async_trait;
use serde_json::Value;
use crate::core::data::{DataContainer, ProcessingError, PeakType, Curve, Peak};
use super::base::{Exporter, ExportResult, ExportConfig, helpers};

/// Plotly exporter for interactive visualization of mass spectrometry data
pub struct PlotlyExporter;

#[async_trait]
impl Exporter for PlotlyExporter {
    fn name(&self) -> &str {
        "plotly_exporter"
    }

    fn description(&self) -> &str {
        "Export mass spectrometry data to Plotly JSON format for interactive visualization"
    }

    fn file_extension(&self) -> &str {
        "json"
    }

    fn mime_type(&self) -> &str {
        "application/json"
    }

    fn config_schema(&self) -> Value {
        serde_json::json!({
            "type": "object",
            "properties": {
                "include_curves": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include curve data in the visualization"
                },
                "include_peaks": {
                    "type": "boolean",
                    "default": true,
                    "description": "Include peak annotations in the visualization"
                },
                "chart_type": {
                    "type": "string",
                    "enum": ["line", "scatter", "bar", "combined"],
                    "default": "combined",
                    "description": "Type of chart to generate"
                },
                "show_peaks": {
                    "type": "boolean",
                    "default": true,
                    "description": "Show peak markers on the chart"
                },
                "show_fit": {
                    "type": "boolean",
                    "default": false,
                    "description": "Show fitted curves"
                },
                "title": {
                    "type": "string",
                    "default": "IMS Data Visualization",
                    "description": "Chart title"
                },
                "x_axis_title": {
                    "type": "string",
                    "default": "Time",
                    "description": "X-axis title"
                },
                "y_axis_title": {
                    "type": "string",
                    "default": "Intensity",
                    "description": "Y-axis title"
                },
                "width": {
                    "type": "integer",
                    "default": 800,
                    "description": "Chart width in pixels"
                },
                "height": {
                    "type": "integer",
                    "default": 600,
                    "description": "Chart height in pixels"
                }
            }
        })
    }

    async fn export(
        &self,
        data: &DataContainer,
        config: Value,
    ) -> Result<ExportResult, ProcessingError> {
        let export_config: ExportConfig = serde_json::from_value(config.clone())
            .unwrap_or_default();
        
        let chart_type = config["chart_type"].as_str().unwrap_or("combined");
        let show_peaks = config["show_peaks"].as_bool().unwrap_or(true);
        let show_fit = config["show_fit"].as_bool().unwrap_or(false);
        let title = config["title"].as_str().unwrap_or("IMS Data Visualization");
        let x_axis_title = config["x_axis_title"].as_str().unwrap_or("Time");
        let y_axis_title = config["y_axis_title"].as_str().unwrap_or("Intensity");
        let width = config["width"].as_u64().unwrap_or(800);
        let height = config["height"].as_u64().unwrap_or(600);
        
        let plotly_data = self.create_plotly_data(data, &export_config, chart_type, show_peaks, show_fit)?;
        let layout = self.create_layout(title, x_axis_title, y_axis_title, width, height);
        
        let plotly_json = serde_json::json!({
            "data": plotly_data,
            "layout": layout,
            "config": {
                "displayModeBar": true,
                "displaylogo": false,
                "modeBarButtonsToRemove": ["pan2d", "lasso2d", "select2d"]
            }
        });
        
        let content = serde_json::to_string_pretty(&plotly_json)
            .map_err(|e| ProcessingError::ProcessError(format!("JSON serialization failed: {}", e)))?;
        
        let filename = format!("ims_plot_{}.json", helpers::generate_timestamp());
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

impl PlotlyExporter {
    /// Create Plotly data traces
    fn create_plotly_data(
        &self,
        data: &DataContainer,
        config: &ExportConfig,
        chart_type: &str,
        show_peaks: bool,
        show_fit: bool,
    ) -> Result<Vec<Value>, ProcessingError> {
        let mut traces = Vec::new();
        
        // Add curve traces
        if config.include_curves {
            for (i, curve) in data.curves.iter().enumerate() {
                let trace = self.create_curve_trace(curve, i, chart_type)?;
                traces.push(trace);
                
                // Add fitted curve if requested
                if show_fit {
                    if let Ok(fit_trace) = self.create_fit_trace(curve, i) {
                        traces.push(fit_trace);
                    }
                }
            }
        }
        
        // Add peak annotations
        if config.include_peaks && show_peaks {
            let peak_trace = self.create_peak_trace(&data.peaks)?;
            traces.push(peak_trace);
        }
        
        Ok(traces)
    }
    
    /// Create a curve trace
    fn create_curve_trace(&self, curve: &Curve, index: usize, chart_type: &str) -> Result<Value, ProcessingError> {
        let trace_type = match chart_type {
            "bar" => "bar",
            "scatter" => "scatter",
            _ => "scatter",
        };
        
        let mode = match chart_type {
            "line" => "lines",
            "scatter" => "markers",
            "bar" => "",
            _ => "lines+markers",
        };
        
        let color = self.get_color_for_index(index);
        let name = format!("{} ({})", curve.curve_type, curve.id);
        
        let mut trace = serde_json::json!({
            "x": curve.x_values,
            "y": curve.y_values,
            "type": trace_type,
            "mode": mode,
            "name": name,
            "line": {
                "color": color,
                "width": 2
            },
            "marker": {
                "color": color,
                "size": 4
            },
            "hovertemplate": format!(
                "<b>{}</b><br>X: %{{x:.6f}}<br>Y: %{{y:.3f}}<br>Curve ID: {}<br>Points: {}<br><extra></extra>",
                curve.curve_type, curve.id, curve.point_count
            )
        });
        
        // Remove mode for bar charts
        if chart_type == "bar" {
            trace.as_object_mut().unwrap().remove("mode");
        }
        
        Ok(trace)
    }
    
    /// Create a fitted curve trace
    fn create_fit_trace(&self, curve: &Curve, index: usize) -> Result<Value, ProcessingError> {
        // For now, create a simple fitted curve based on peak data
        // In a real implementation, you would use the actual fitted parameters
        let color = self.get_color_for_index(index);
        
        Ok(serde_json::json!({
            "x": curve.x_values,
            "y": curve.y_values, // This should be the fitted values
            "type": "scatter",
            "mode": "lines",
            "name": format!("{} (Fitted)", curve.curve_type),
            "line": {
                "color": color,
                "width": 1,
                "dash": "dash"
            },
            "opacity": 0.7,
            "showlegend": true
        }))
    }
    
    /// Create peak annotations trace
    fn create_peak_trace(&self, peaks: &[Peak]) -> Result<Value, ProcessingError> {
        let mut peak_x = Vec::new();
        let mut peak_y = Vec::new();
        let mut peak_text = Vec::new();
        let mut peak_colors = Vec::new();
        
        for peak in peaks {
            peak_x.push(peak.center);
            peak_y.push(peak.amplitude);
            
            let peak_info = format!(
                "Peak: {}<br>Center: {:.6}<br>Amplitude: {:.3}<br>FWHM: {:.6}<br>Area: {:.3}<br>R²: {:.4}<br>Type: {}",
                peak.id,
                peak.center,
                peak.amplitude,
                peak.fwhm,
                peak.area,
                peak.rsquared,
                self.format_peak_type(&peak.peak_type)
            );
            peak_text.push(peak_info);
            
            // Color based on peak type
            let color = match peak.peak_type {
                PeakType::Gaussian => "#FF6B6B",
                PeakType::Lorentzian => "#4ECDC4",
                PeakType::PseudoVoigt => "#45B7D1",
                PeakType::AsymmetricGaussian => "#96CEB4",
                PeakType::Custom(_) => "#FFEAA7",
                PeakType::EMG => "#A29BFE",
                PeakType::BiGaussian => "#6C5CE7",
                PeakType::VoigtExponentialTail => "#FD79A8",
                PeakType::PearsonIV => "#FDCB6E",
                PeakType::NLC => "#E17055",
                PeakType::GMGBayesian => "#00B894",
            };
            peak_colors.push(color);
        }
        
        Ok(serde_json::json!({
            "x": peak_x,
            "y": peak_y,
            "type": "scatter",
            "mode": "markers",
            "name": "Peaks",
            "marker": {
                "color": peak_colors,
                "size": 8,
                "symbol": "diamond",
                "line": {
                    "color": "white",
                    "width": 1
                }
            },
            "text": peak_text,
            "hovertemplate": "%{text}<extra></extra>",
            "showlegend": true
        }))
    }
    
    /// Create Plotly layout
    fn create_layout(&self, title: &str, x_axis_title: &str, y_axis_title: &str, width: u64, height: u64) -> Value {
        serde_json::json!({
            "title": {
                "text": title,
                "x": 0.5,
                "font": {
                    "size": 16
                }
            },
            "xaxis": {
                "title": {
                    "text": x_axis_title,
                    "font": {
                        "size": 14
                    }
                },
                "showgrid": true,
                "gridcolor": "#E5E5E5",
                "zeroline": false
            },
            "yaxis": {
                "title": {
                    "text": y_axis_title,
                    "font": {
                        "size": 14
                    }
                },
                "showgrid": true,
                "gridcolor": "#E5E5E5",
                "zeroline": false
            },
            "width": width,
            "height": height,
            "margin": {
                "l": 60,
                "r": 60,
                "t": 60,
                "b": 60
            },
            "plot_bgcolor": "white",
            "paper_bgcolor": "white",
            "font": {
                "family": "Arial, sans-serif",
                "size": 12
            },
            "legend": {
                "x": 1.02,
                "y": 1,
                "bgcolor": "rgba(255,255,255,0.8)",
                "bordercolor": "#CCCCCC",
                "borderwidth": 1
            },
            "hovermode": "closest"
        })
    }
    
    /// Get color for trace index
    fn get_color_for_index(&self, index: usize) -> &'static str {
        let colors = [
            "#1f77b4", "#ff7f0e", "#2ca02c", "#d62728", "#9467bd",
            "#8c564b", "#e377c2", "#7f7f7f", "#bcbd22", "#17becf"
        ];
        colors[index % colors.len()]
    }
    
    /// Format peak type for display
    fn format_peak_type(&self, peak_type: &PeakType) -> String {
        match peak_type {
            PeakType::Gaussian => "Gaussian".to_string(),
            PeakType::Lorentzian => "Lorentzian".to_string(),
            PeakType::PseudoVoigt => "Pseudo-Voigt".to_string(),
            PeakType::AsymmetricGaussian => "Asymmetric Gaussian".to_string(),
            PeakType::Custom(name) => format!("Custom ({})", name),
            PeakType::EMG => "EMG".to_string(),
            PeakType::BiGaussian => "BiGaussian".to_string(),
            PeakType::VoigtExponentialTail => "Voigt+ExpTail".to_string(),
            PeakType::PearsonIV => "PearsonIV".to_string(),
            PeakType::NLC => "NLC".to_string(),
            PeakType::GMGBayesian => "GMGBayesian".to_string(),
        }
    }
}
