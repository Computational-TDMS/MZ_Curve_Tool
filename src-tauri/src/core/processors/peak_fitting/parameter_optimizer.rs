//! 参数优化器
//! 
//! 对单个峰进行参数优化，支持多种优化算法

use crate::core::data::ProcessingError;
use crate::core::processors::peak_fitting::peak_shapes::PeakShapeParams;

/// 优化算法类型
#[derive(Debug, Clone)]
pub enum OptimizationAlgorithm {
    /// 网格搜索
    GridSearch {
        resolution: usize,
        max_iterations: usize,
    },
    /// 梯度下降
    GradientDescent {
        learning_rate: f64,
        max_iterations: usize,
        convergence_threshold: f64,
    },
    /// Levenberg-Marquardt
    LevenbergMarquardt {
        max_iterations: usize,
        convergence_threshold: f64,
        damping_factor: f64,
    },
    /// 模拟退火
    SimulatedAnnealing {
        initial_temperature: f64,
        cooling_rate: f64,
        max_iterations: usize,
    },
}

/// 优化结果
#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub optimized_params: PeakShapeParams,
    pub final_error: f64,
    pub iterations: usize,
    pub converged: bool,
    pub parameter_errors: Vec<f64>,
}

/// 参数优化器
#[derive(Debug)]
pub struct ParameterOptimizer {
    algorithm: OptimizationAlgorithm,
}

impl ParameterOptimizer {
    pub fn new(algorithm: OptimizationAlgorithm) -> Self {
        Self { algorithm }
    }
    
    /// 执行参数优化
    pub fn optimize<F>(
        &self,
        objective_function: F,
        initial_params: PeakShapeParams,
        x_data: &[f64],
        y_data: &[f64],
    ) -> Result<OptimizationResult, ProcessingError>
    where
        F: Fn(&[f64], &[f64], &PeakShapeParams) -> f64,
    {
        match &self.algorithm {
            OptimizationAlgorithm::GridSearch { resolution, max_iterations } => {
                self.grid_search_optimization(objective_function, initial_params, x_data, y_data, *resolution, *max_iterations)
            },
            OptimizationAlgorithm::GradientDescent { learning_rate, max_iterations, convergence_threshold } => {
                self.gradient_descent_optimization(objective_function, initial_params, x_data, y_data, *learning_rate, *max_iterations, *convergence_threshold)
            },
            OptimizationAlgorithm::LevenbergMarquardt { max_iterations, convergence_threshold, damping_factor } => {
                self.levenberg_marquardt_optimization(objective_function, initial_params, x_data, y_data, *max_iterations, *convergence_threshold, *damping_factor)
            },
            OptimizationAlgorithm::SimulatedAnnealing { initial_temperature, cooling_rate, max_iterations } => {
                self.simulated_annealing_optimization(objective_function, initial_params, x_data, y_data, *initial_temperature, *cooling_rate, *max_iterations)
            },
        }
    }
    
    /// 网格搜索优化
    fn grid_search_optimization<F>(
        &self,
        objective_function: F,
        initial_params: PeakShapeParams,
        x_data: &[f64],
        y_data: &[f64],
        resolution: usize,
        max_iterations: usize,
    ) -> Result<OptimizationResult, ProcessingError>
    where
        F: Fn(&[f64], &[f64], &PeakShapeParams) -> f64,
    {
        let mut best_params = initial_params.clone();
        let mut best_error = objective_function(x_data, y_data, &best_params);
        let mut iterations = 0;
        
        // 为每个参数创建搜索范围
        let mut param_ranges = Vec::new();
        for (i, &_param) in initial_params.parameters.iter().enumerate() {
            if i < initial_params.bounds.len() {
                let (min, max) = initial_params.bounds[i];
                let range_size = max - min;
                let step_size = range_size / resolution as f64;
                let mut range = Vec::new();
                
                for j in 0..=resolution {
                    let value = min + j as f64 * step_size;
                    range.push(value);
                }
                param_ranges.push(range);
            }
        }
        
        // 网格搜索
        let mut current_params = initial_params.clone();
        self.grid_search_recursive(
            &objective_function,
            x_data,
            y_data,
            &param_ranges,
            &mut current_params,
            0,
            &mut best_params,
            &mut best_error,
            &mut iterations,
            max_iterations,
        );
        
        let parameter_errors = self.estimate_parameter_errors(&objective_function, x_data, y_data, &best_params);
        
        Ok(OptimizationResult {
            optimized_params: best_params,
            final_error: best_error,
            iterations,
            converged: iterations < max_iterations,
            parameter_errors,
        })
    }
    
    /// 递归网格搜索
    fn grid_search_recursive<F>(
        &self,
        objective_function: &F,
        x_data: &[f64],
        y_data: &[f64],
        param_ranges: &[Vec<f64>],
        current_params: &mut PeakShapeParams,
        param_index: usize,
        best_params: &mut PeakShapeParams,
        best_error: &mut f64,
        iterations: &mut usize,
        max_iterations: usize,
    ) where
        F: Fn(&[f64], &[f64], &PeakShapeParams) -> f64,
    {
        if *iterations >= max_iterations || param_index >= param_ranges.len() {
            return;
        }
        
        for &value in &param_ranges[param_index] {
            current_params.parameters[param_index] = value;
            
            if param_index == param_ranges.len() - 1 {
                // 所有参数都已设置，计算目标函数
                *iterations += 1;
                let error = objective_function(x_data, y_data, current_params);
                
                if error < *best_error {
                    *best_error = error;
                    *best_params = current_params.clone();
                }
            } else {
                // 递归设置下一个参数
                self.grid_search_recursive(
                    objective_function,
                    x_data,
                    y_data,
                    param_ranges,
                    current_params,
                    param_index + 1,
                    best_params,
                    best_error,
                    iterations,
                    max_iterations,
                );
            }
        }
    }
    
    /// 梯度下降优化
    fn gradient_descent_optimization<F>(
        &self,
        objective_function: F,
        mut params: PeakShapeParams,
        x_data: &[f64],
        y_data: &[f64],
        learning_rate: f64,
        max_iterations: usize,
        convergence_threshold: f64,
    ) -> Result<OptimizationResult, ProcessingError>
    where
        F: Fn(&[f64], &[f64], &PeakShapeParams) -> f64,
    {
        let mut iterations = 0;
        let mut previous_error = f64::INFINITY;
        
        for _ in 0..max_iterations {
            iterations += 1;
            
            // 计算梯度
            let gradient = self.compute_gradient(&objective_function, x_data, y_data, &params);
            
            // 更新参数
            for (i, param) in params.parameters.iter_mut().enumerate() {
                *param -= learning_rate * gradient[i];
            }
            
            // 应用边界约束
            params.clamp_parameters();
            
            // 计算新的误差
            let current_error = objective_function(x_data, y_data, &params);
            
            // 检查收敛
            if (previous_error - current_error).abs() < convergence_threshold {
                break;
            }
            
            previous_error = current_error;
        }
        
        let final_error = objective_function(x_data, y_data, &params);
        let parameter_errors = self.estimate_parameter_errors(&objective_function, x_data, y_data, &params);
        
        Ok(OptimizationResult {
            optimized_params: params,
            final_error,
            iterations,
            converged: iterations < max_iterations,
            parameter_errors,
        })
    }
    
    /// Levenberg-Marquardt优化
    fn levenberg_marquardt_optimization<F>(
        &self,
        objective_function: F,
        mut params: PeakShapeParams,
        x_data: &[f64],
        y_data: &[f64],
        max_iterations: usize,
        convergence_threshold: f64,
        damping_factor: f64,
    ) -> Result<OptimizationResult, ProcessingError>
    where
        F: Fn(&[f64], &[f64], &PeakShapeParams) -> f64,
    {
        let mut iterations = 0;
        let mut lambda = damping_factor;
        
        for _ in 0..max_iterations {
            iterations += 1;
            
            // 计算残差和雅可比矩阵
            let (residuals, jacobian) = self.compute_residuals_and_jacobian(&objective_function, x_data, y_data, &params)?;
            
            // 计算参数更新
            let parameter_update = self.solve_linear_system(&jacobian, &residuals, lambda)?;
            
            // 更新参数
            let mut new_params = params.clone();
            for (i, param) in new_params.parameters.iter_mut().enumerate() {
                *param -= parameter_update[i];
            }
            
            // 应用边界约束
            new_params.clamp_parameters();
            
            // 计算新的误差
            let current_error = objective_function(x_data, y_data, &new_params);
            let previous_error = objective_function(x_data, y_data, &params);
            
            // 检查是否接受新参数
            if current_error < previous_error {
                params = new_params;
                lambda /= 2.0;
            } else {
                lambda *= 2.0;
            }
            
            // 检查收敛
            if parameter_update.iter().map(|&x| x.abs()).sum::<f64>() < convergence_threshold {
                break;
            }
        }
        
        let final_error = objective_function(x_data, y_data, &params);
        let parameter_errors = self.estimate_parameter_errors(&objective_function, x_data, y_data, &params);
        
        Ok(OptimizationResult {
            optimized_params: params,
            final_error,
            iterations,
            converged: iterations < max_iterations,
            parameter_errors,
        })
    }
    
    /// 模拟退火优化
    fn simulated_annealing_optimization<F>(
        &self,
        objective_function: F,
        mut params: PeakShapeParams,
        x_data: &[f64],
        y_data: &[f64],
        initial_temperature: f64,
        cooling_rate: f64,
        max_iterations: usize,
    ) -> Result<OptimizationResult, ProcessingError>
    where
        F: Fn(&[f64], &[f64], &PeakShapeParams) -> f64,
    {
        let mut iterations = 0;
        let mut temperature = initial_temperature;
        let mut best_params = params.clone();
        let mut best_error = objective_function(x_data, y_data, &best_params);
        
        for _ in 0..max_iterations {
            iterations += 1;
            
            // 生成邻域解
            let mut neighbor_params = params.clone();
            self.generate_neighbor(&mut neighbor_params, temperature);
            neighbor_params.clamp_parameters();
            
            // 计算目标函数值
            let neighbor_error = objective_function(x_data, y_data, &neighbor_params);
            let current_error = objective_function(x_data, y_data, &params);
            
            // 接受准则
            let delta = neighbor_error - current_error;
            if delta < 0.0 || rand::random::<f64>() < (-delta / temperature).exp() {
                params = neighbor_params;
                
                if neighbor_error < best_error {
                    best_params = params.clone();
                    best_error = neighbor_error;
                }
            }
            
            // 降温
            temperature *= cooling_rate;
            
            if temperature < 1e-6 {
                break;
            }
        }
        
        let parameter_errors = self.estimate_parameter_errors(&objective_function, x_data, y_data, &best_params);
        
        Ok(OptimizationResult {
            optimized_params: best_params,
            final_error: best_error,
            iterations,
            converged: temperature < 1e-6,
            parameter_errors,
        })
    }
    
    /// 计算梯度
    fn compute_gradient<F>(
        &self,
        objective_function: &F,
        x_data: &[f64],
        y_data: &[f64],
        params: &PeakShapeParams,
    ) -> Vec<f64>
    where
        F: Fn(&[f64], &[f64], &PeakShapeParams) -> f64,
    {
        let h = 1e-6;
        let mut gradient = Vec::new();
        
        for i in 0..params.parameters.len() {
            let mut params_plus = params.clone();
            let mut params_minus = params.clone();
            
            params_plus.parameters[i] += h;
            params_minus.parameters[i] -= h;
            
            let f_plus = objective_function(x_data, y_data, &params_plus);
            let f_minus = objective_function(x_data, y_data, &params_minus);
            
            gradient.push((f_plus - f_minus) / (2.0 * h));
        }
        
        gradient
    }
    
    /// 计算残差和雅可比矩阵
    fn compute_residuals_and_jacobian<F>(
        &self,
        _objective_function: &F,
        x_data: &[f64],
        y_data: &[f64],
        params: &PeakShapeParams,
    ) -> Result<(Vec<f64>, Vec<Vec<f64>>), ProcessingError>
    where
        F: Fn(&[f64], &[f64], &PeakShapeParams) -> f64,
    {
        let n_points = x_data.len();
        let n_params = params.parameters.len();
        
        let mut residuals = vec![0.0; n_points];
        let mut jacobian = vec![vec![0.0; n_params]; n_points];
        
        // 计算残差
        for i in 0..n_points {
            let predicted = self.predict_single_point(x_data[i], params);
            residuals[i] = y_data[i] - predicted;
        }
        
        // 计算雅可比矩阵
        let h = 1e-6;
        for i in 0..n_points {
            for j in 0..n_params {
                let mut params_plus = params.clone();
                let mut params_minus = params.clone();
                
                params_plus.parameters[j] += h;
                params_minus.parameters[j] -= h;
                
                let f_plus = self.predict_single_point(x_data[i], &params_plus);
                let f_minus = self.predict_single_point(x_data[i], &params_minus);
                
                jacobian[i][j] = (f_plus - f_minus) / (2.0 * h);
            }
        }
        
        Ok((residuals, jacobian))
    }
    
    /// 预测单个点的值
    fn predict_single_point(&self, x: f64, params: &PeakShapeParams) -> f64 {
        match params.shape_type {
            crate::core::processors::peak_fitting::peak_shapes::PeakShapeType::Gaussian => {
                let amplitude = params.get_parameter("amplitude").unwrap_or(0.0);
                let center = params.get_parameter("center").unwrap_or(0.0);
                let sigma = params.get_parameter("sigma").unwrap_or(1.0);
                
                let exponent = -((x - center).powi(2)) / (2.0 * sigma.powi(2));
                amplitude * exponent.exp()
            },
            crate::core::processors::peak_fitting::peak_shapes::PeakShapeType::Lorentzian => {
                let amplitude = params.get_parameter("amplitude").unwrap_or(0.0);
                let center = params.get_parameter("center").unwrap_or(0.0);
                let gamma = params.get_parameter("gamma").unwrap_or(1.0);
                
                let denominator = 1.0 + ((x - center) / gamma).powi(2);
                amplitude / denominator
            },
            _ => {
                // 默认使用高斯
                let amplitude = params.get_parameter("amplitude").unwrap_or(0.0);
                let center = params.get_parameter("center").unwrap_or(0.0);
                let sigma = params.get_parameter("sigma").unwrap_or(1.0);
                
                let exponent = -((x - center).powi(2)) / (2.0 * sigma.powi(2));
                amplitude * exponent.exp()
            }
        }
    }
    
    /// 求解线性方程组
    fn solve_linear_system(
        &self,
        jacobian: &[Vec<f64>],
        residuals: &[f64],
        lambda: f64,
    ) -> Result<Vec<f64>, ProcessingError> {
        let n_params = jacobian[0].len();
        let n_points = jacobian.len();
        
        // 计算正规方程: (J^T * J + λI) * Δp = J^T * r
        let mut jtj = vec![vec![0.0; n_params]; n_params];
        let mut jtr = vec![0.0; n_params];
        
        // 计算J^T * J
        for i in 0..n_params {
            for j in 0..n_params {
                for k in 0..n_points {
                    jtj[i][j] += jacobian[k][i] * jacobian[k][j];
                }
                // 添加阻尼项
                if i == j {
                    jtj[i][j] += lambda;
                }
            }
        }
        
        // 计算J^T * r
        for i in 0..n_params {
            for k in 0..n_points {
                jtr[i] += jacobian[k][i] * residuals[k];
            }
        }
        
        // 求解线性方程组
        self.gaussian_elimination(&jtj, &jtr)
    }
    
    /// 高斯消元法
    fn gaussian_elimination(&self, matrix: &[Vec<f64>], rhs: &[f64]) -> Result<Vec<f64>, ProcessingError> {
        let n = matrix.len();
        let mut a = matrix.to_vec();
        let mut b = rhs.to_vec();
        
        // 高斯消元
        for i in 0..n {
            // 寻找主元
            let mut max_row = i;
            for k in (i + 1)..n {
                if a[k][i].abs() > a[max_row][i].abs() {
                    max_row = k;
                }
            }
            
            // 交换行
            if max_row != i {
                a.swap(i, max_row);
                b.swap(i, max_row);
            }
            
            // 检查奇异矩阵
            if a[i][i].abs() < 1e-12 {
                return Err(ProcessingError::process_error("雅可比矩阵奇异，无法求解"));
            }
            
            // 消元
            for k in (i + 1)..n {
                let factor = a[k][i] / a[i][i];
                for j in i..n {
                    a[k][j] -= factor * a[i][j];
                }
                b[k] -= factor * b[i];
            }
        }
        
        // 回代
        let mut x = vec![0.0; n];
        for i in (0..n).rev() {
            x[i] = b[i];
            for j in (i + 1)..n {
                x[i] -= a[i][j] * x[j];
            }
            x[i] /= a[i][i];
        }
        
        Ok(x)
    }
    
    /// 生成邻域解
    fn generate_neighbor(&self, params: &mut PeakShapeParams, temperature: f64) {
        for (_i, param) in params.parameters.iter_mut().enumerate() {
            let noise = (rand::random::<f64>() - 0.5) * 2.0 * temperature;
            *param += noise;
        }
    }
    
    /// 估计参数误差
    fn estimate_parameter_errors<F>(
        &self,
        objective_function: &F,
        x_data: &[f64],
        y_data: &[f64],
        params: &PeakShapeParams,
    ) -> Vec<f64>
    where
        F: Fn(&[f64], &[f64], &PeakShapeParams) -> f64,
    {
        let h = 1e-4;
        let mut errors = Vec::new();
        
        for i in 0..params.parameters.len() {
            let mut params_plus = params.clone();
            let mut params_minus = params.clone();
            
            params_plus.parameters[i] += h;
            params_minus.parameters[i] -= h;
            
            let f_plus = objective_function(x_data, y_data, &params_plus);
            let f_minus = objective_function(x_data, y_data, &params_minus);
            let f_center = objective_function(x_data, y_data, params);
            
            // 使用二阶导数估计误差
            let second_derivative = (f_plus - 2.0 * f_center + f_minus) / (h * h);
            let error = if second_derivative > 0.0 {
                (2.0 * f_center / second_derivative).sqrt()
            } else {
                0.0
            };
            
            errors.push(error);
        }
        
        errors
    }
}
