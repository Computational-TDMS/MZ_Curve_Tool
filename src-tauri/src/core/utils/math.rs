// 数学工具函数
pub fn gaussian(x: f64, mu: f64, sigma: f64) -> f64 {
    let coefficient = 1.0 / (sigma * (2.0 * std::f64::consts::PI).sqrt());
    let exponent = -0.5 * ((x - mu) / sigma).powi(2);
    coefficient * exponent.exp()
}

pub fn lorentzian(x: f64, mu: f64, gamma: f64) -> f64 {
    let coefficient = 1.0 / (std::f64::consts::PI * gamma);
    let denominator = 1.0 + ((x - mu) / gamma).powi(2);
    coefficient / denominator
}
