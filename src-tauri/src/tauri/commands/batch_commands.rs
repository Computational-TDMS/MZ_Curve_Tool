//! 批量处理相关命令

use tauri::State;
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use std::collections::VecDeque;
use crate::tauri::state::{AppStateManager, ProcessingStatus};

/// 批量处理任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTask {
    pub id: String,
    pub file_path: String,
    pub params: BatchTaskParams,
    pub status: BatchTaskStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub started_at: Option<chrono::DateTime<chrono::Utc>>,
    pub completed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub error: Option<String>,
    pub result: Option<BatchTaskResult>,
}

/// 批量处理任务参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTaskParams {
    pub extraction: ExtractionParams,
    pub detection: DetectionParams,
    pub fitting: FittingParams,
}

/// 提取参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionParams {
    pub mz_range: String,
    pub rt_range: String,
    pub ms_level: u8,
    pub curve_type: String,
}

/// 检测参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionParams {
    pub method: String,
    pub sensitivity: f64,
    pub threshold_multiplier: f64,
    pub min_peak_width: f64,
    pub max_peak_width: f64,
}

/// 拟合参数
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FittingParams {
    pub method: String,
    pub overlapping_method: String,
    pub fit_quality_threshold: f64,
    pub max_iterations: u32,
}

/// 批量处理任务状态
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BatchTaskStatus {
    Pending,
    Processing,
    Completed,
    Failed,
    Cancelled,
}

/// 批量处理任务结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchTaskResult {
    pub curves_count: usize,
    pub peaks_count: usize,
    pub processing_time_ms: u64,
    pub quality_score: Option<f64>,
}

/// 批量处理队列
#[derive(Debug)]
pub struct BatchQueue {
    pub tasks: VecDeque<BatchTask>,
    pub current_task: Option<BatchTask>,
    pub is_processing: bool,
    pub max_concurrent: usize,
}

impl BatchQueue {
    pub fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
            current_task: None,
            is_processing: false,
            max_concurrent: 1, // 单线程处理，避免资源竞争
        }
    }

    pub fn add_task(&mut self, task: BatchTask) {
        self.tasks.push_back(task);
    }

    pub fn get_next_task(&mut self) -> Option<BatchTask> {
        self.tasks.pop_front()
    }

    pub fn get_queue_status(&self) -> QueueStatus {
        QueueStatus {
            total_tasks: self.tasks.len(),
            current_task: self.current_task.clone(),
            is_processing: self.is_processing,
            completed_tasks: 0, // 需要从外部维护
            failed_tasks: 0,    // 需要从外部维护
        }
    }
}

/// 队列状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStatus {
    pub total_tasks: usize,
    pub current_task: Option<BatchTask>,
    pub is_processing: bool,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
}

/// 批量处理管理器
#[derive(Debug)]
pub struct BatchProcessor {
    pub queue: Mutex<BatchQueue>,
    pub results: Mutex<Vec<BatchTask>>,
}

impl BatchProcessor {
    pub fn new() -> Self {
        Self {
            queue: Mutex::new(BatchQueue::new()),
            results: Mutex::new(Vec::new()),
        }
    }

    pub fn add_batch_tasks(&self, file_paths: Vec<String>, params: BatchTaskParams) -> Vec<String> {
        let mut queue = self.queue.lock().unwrap();
        let mut task_ids = Vec::new();

        for file_path in file_paths {
            let task_id = uuid::Uuid::new_v4().to_string();
            let task = BatchTask {
                id: task_id.clone(),
                file_path,
                params: params.clone(),
                status: BatchTaskStatus::Pending,
                created_at: chrono::Utc::now(),
                started_at: None,
                completed_at: None,
                error: None,
                result: None,
            };

            queue.add_task(task);
            task_ids.push(task_id);
        }

        task_ids
    }

    pub fn get_queue_status(&self) -> QueueStatus {
        let queue = self.queue.lock().unwrap();
        let results = self.results.lock().unwrap();
        
        let completed_tasks = results.iter().filter(|t| t.status == BatchTaskStatus::Completed).count();
        let failed_tasks = results.iter().filter(|t| t.status == BatchTaskStatus::Failed).count();

        QueueStatus {
            total_tasks: queue.tasks.len(),
            current_task: queue.current_task.clone(),
            is_processing: queue.is_processing,
            completed_tasks,
            failed_tasks,
        }
    }

    pub fn clear_queue(&self) {
        let mut queue = self.queue.lock().unwrap();
        queue.tasks.clear();
        queue.current_task = None;
        queue.is_processing = false;
    }

    pub fn clear_results(&self) {
        let mut results = self.results.lock().unwrap();
        results.clear();
    }
}

/// 添加批量处理任务
#[tauri::command]
pub async fn add_batch_tasks(
    file_paths: Vec<String>,
    params: BatchTaskParams,
    processor: State<'_, BatchProcessor>
) -> Result<Vec<String>, String> {
    let task_ids = processor.add_batch_tasks(file_paths, params);
    Ok(task_ids)
}

/// 获取队列状态
#[tauri::command]
pub async fn get_batch_queue_status(
    processor: State<'_, BatchProcessor>
) -> Result<QueueStatus, String> {
    Ok(processor.get_queue_status())
}

/// 清空队列
#[tauri::command]
pub async fn clear_batch_queue(
    processor: State<'_, BatchProcessor>
) -> Result<(), String> {
    processor.clear_queue();
    Ok(())
}

/// 清空结果
#[tauri::command]
pub async fn clear_batch_results(
    processor: State<'_, BatchProcessor>
) -> Result<(), String> {
    processor.clear_results();
    Ok(())
}

/// 开始批量处理
#[tauri::command]
pub async fn start_batch_processing(
    app: tauri::AppHandle,
    processor: State<'_, BatchProcessor>,
    state: State<'_, AppStateManager>
) -> Result<(), String> {
    let mut queue = processor.queue.lock().unwrap();
    
    if queue.is_processing {
        return Err("批量处理已在进行中".to_string());
    }

    if queue.tasks.is_empty() {
        return Err("队列中没有待处理的任务".to_string());
    }

    queue.is_processing = true;
    drop(queue); // 释放锁

    // 在后台任务中处理队列
    let processor_clone = processor.inner().clone();
    let app_clone = app.clone();
    let state_clone = state.inner().clone();

    tokio::spawn(async move {
        process_batch_queue(processor_clone, app_clone, state_clone).await;
    });

    Ok(())
}

/// 停止批量处理
#[tauri::command]
pub async fn stop_batch_processing(
    processor: State<'_, BatchProcessor>
) -> Result<(), String> {
    let mut queue = processor.queue.lock().unwrap();
    queue.is_processing = false;
    
    if let Some(mut current_task) = queue.current_task.take() {
        current_task.status = BatchTaskStatus::Cancelled;
        current_task.completed_at = Some(chrono::Utc::now());
        
        let mut results = processor.results.lock().unwrap();
        results.push(current_task);
    }

    Ok(())
}

/// 处理批量队列的后台任务
async fn process_batch_queue(
    processor: std::sync::Arc<BatchProcessor>,
    app: tauri::AppHandle,
    state: std::sync::Arc<AppStateManager>,
) {
    loop {
        // 获取下一个任务
        let task = {
            let mut queue = processor.queue.lock().unwrap();
            if !queue.is_processing || queue.tasks.is_empty() {
                break;
            }
            queue.current_task = queue.get_next_task();
            queue.current_task.clone()
        };

        let Some(mut task) = task else {
            break;
        };

        // 更新任务状态
        task.status = BatchTaskStatus::Processing;
        task.started_at = Some(chrono::Utc::now());

        // 发送进度更新
        state.emit_progress_update(&app, 0, 1, &format!("处理文件: {}", task.file_path));

        // 处理任务
        let result = process_single_batch_task(&task, &app, &state).await;

        // 更新任务结果
        task.completed_at = Some(chrono::Utc::now());
        match result {
            Ok(task_result) => {
                task.status = BatchTaskStatus::Completed;
                task.result = Some(task_result);
            }
            Err(error) => {
                task.status = BatchTaskStatus::Failed;
                task.error = Some(error);
            }
        }

        // 保存结果
        {
            let mut results = processor.results.lock().unwrap();
            results.push(task);
        }

        // 检查是否应该继续处理
        let should_continue = {
            let queue = processor.queue.lock().unwrap();
            queue.is_processing && !queue.tasks.is_empty()
        };

        if !should_continue {
            break;
        }
    }

    // 处理完成
    {
        let mut queue = processor.queue.lock().unwrap();
        queue.is_processing = false;
        queue.current_task = None;
    }

    state.emit_progress_update(&app, 1, 1, "批量处理完成");
}

/// 处理单个批量任务
async fn process_single_batch_task(
    task: &BatchTask,
    app: &tauri::AppHandle,
    state: &AppStateManager,
) -> Result<BatchTaskResult, String> {
    let start_time = std::time::Instant::now();

    // 这里应该调用实际的处理逻辑
    // 暂时返回模拟结果
    tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;

    let processing_time = start_time.elapsed().as_millis() as u64;

    Ok(BatchTaskResult {
        curves_count: 10, // 模拟数据
        peaks_count: 5,   // 模拟数据
        processing_time_ms: processing_time,
        quality_score: Some(0.95),
    })
}
