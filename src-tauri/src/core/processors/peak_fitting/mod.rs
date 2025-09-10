//! 峰拟合模块
//! 
//! 包含多种峰拟合算法的实现

pub mod gaussian_fitter;
pub mod lorentzian_fitter;
pub mod pseudo_voigt_fitter;
pub mod multi_peak_fitter;
pub mod emg_fitter;
pub mod bi_gaussian_fitter;
pub mod voigt_exponential_tail_fitter;
pub mod pearson_iv_fitter;
pub mod nlc_fitter;
pub mod gmg_bayesian_fitter;

use crate::core::data::{Curve, Peak, ProcessingError};
use serde_json::Value;

/// 峰拟合器trait
pub trait PeakFitter {
    fn name(&self) -> &str;
    fn fit_peak(&self, peak: &Peak, curve: &Curve, config: &Value) -> Result<Peak, ProcessingError>;
}

/// 峰拟合器枚举
#[derive(Debug)]
pub enum PeakFitterEnum {
    Gaussian(gaussian_fitter::GaussianFitter),
    Lorentzian(lorentzian_fitter::LorentzianFitter),
    PseudoVoigt(pseudo_voigt_fitter::PseudoVoigtFitter),
    MultiPeak(multi_peak_fitter::MultiPeakFitter),
    EMG(emg_fitter::EMGFitter),
    BiGaussian(bi_gaussian_fitter::BiGaussianFitter),
    VoigtExponentialTail(voigt_exponential_tail_fitter::VoigtExponentialTailFitter),
    PearsonIV(pearson_iv_fitter::PearsonIVFitter),
    NLC(nlc_fitter::NLCFitter),
    GMGBayesian(gmg_bayesian_fitter::GMGBayesianFitter),
}

impl PeakFitter for PeakFitterEnum {
    fn name(&self) -> &str {
        match self {
            PeakFitterEnum::Gaussian(f) => f.name(),
            PeakFitterEnum::Lorentzian(f) => f.name(),
            PeakFitterEnum::PseudoVoigt(f) => f.name(),
            PeakFitterEnum::MultiPeak(f) => f.name(),
            PeakFitterEnum::EMG(f) => f.name(),
            PeakFitterEnum::BiGaussian(f) => f.name(),
            PeakFitterEnum::VoigtExponentialTail(f) => f.name(),
            PeakFitterEnum::PearsonIV(f) => f.name(),
            PeakFitterEnum::NLC(f) => f.name(),
            PeakFitterEnum::GMGBayesian(f) => f.name(),
        }
    }

    fn fit_peak(&self, peak: &Peak, curve: &Curve, config: &Value) -> Result<Peak, ProcessingError> {
        match self {
            PeakFitterEnum::Gaussian(f) => f.fit_peak(peak, curve, config),
            PeakFitterEnum::Lorentzian(f) => f.fit_peak(peak, curve, config),
            PeakFitterEnum::PseudoVoigt(f) => f.fit_peak(peak, curve, config),
            PeakFitterEnum::MultiPeak(f) => f.fit_peak(peak, curve, config),
            PeakFitterEnum::EMG(f) => f.fit_peak(peak, curve, config),
            PeakFitterEnum::BiGaussian(f) => f.fit_peak(peak, curve, config),
            PeakFitterEnum::VoigtExponentialTail(f) => f.fit_peak(peak, curve, config),
            PeakFitterEnum::PearsonIV(f) => f.fit_peak(peak, curve, config),
            PeakFitterEnum::NLC(f) => f.fit_peak(peak, curve, config),
            PeakFitterEnum::GMGBayesian(f) => f.fit_peak(peak, curve, config),
        }
    }
}

/// 创建峰拟合器
pub fn create_fitter(method: &str) -> Result<PeakFitterEnum, ProcessingError> {
    match method {
        "gaussian" => Ok(PeakFitterEnum::Gaussian(gaussian_fitter::GaussianFitter)),
        "lorentzian" => Ok(PeakFitterEnum::Lorentzian(lorentzian_fitter::LorentzianFitter)),
        "pseudo_voigt" => Ok(PeakFitterEnum::PseudoVoigt(pseudo_voigt_fitter::PseudoVoigtFitter)),
        "multi_peak" => Ok(PeakFitterEnum::MultiPeak(multi_peak_fitter::MultiPeakFitter)),
        "emg" => Ok(PeakFitterEnum::EMG(emg_fitter::EMGFitter)),
        "bi_gaussian" => Ok(PeakFitterEnum::BiGaussian(bi_gaussian_fitter::BiGaussianFitter)),
        "voigt_exponential_tail" => Ok(PeakFitterEnum::VoigtExponentialTail(voigt_exponential_tail_fitter::VoigtExponentialTailFitter)),
        "pearson_iv" => Ok(PeakFitterEnum::PearsonIV(pearson_iv_fitter::PearsonIVFitter::new())),
        "nlc" => Ok(PeakFitterEnum::NLC(nlc_fitter::NLCFitter::new())),
        "gmg_bayesian" => Ok(PeakFitterEnum::GMGBayesian(gmg_bayesian_fitter::GMGBayesianFitter::new())),
        _ => Err(ProcessingError::ConfigError(format!("不支持的拟合方法: {}", method))),
    }
}
