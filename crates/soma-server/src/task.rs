/// 任务队列调度模块
///
/// 实现带并发限制和排队机制的任务调度器。当当前执行任务数未达到最大并发数时，
/// 新任务立即在新线程中启动执行；超出并发限制时任务进入等待队列；
/// 队列也满时返回错误。任务完成后自动从队列中取下一个任务执行。

use soma_core::error::SomaError;
use soma_core::models::{TaskStatus, VideoParams};
use crate::state;
use crate::service;
use std::sync::Mutex;
use std::collections::VecDeque;

lazy_static! {
    /// 全局任务队列实例，默认并发数5、排队数100，可通过 init_queue 重新初始化
    pub static ref TASK_QUEUE: Mutex<TaskQueue> = Mutex::new(TaskQueue::new(5, 100));
}

/// 任务队列结构体，管理并发执行和排队等待的任务
pub struct TaskQueue {
    /// 最大并发执行任务数
    max_concurrent: usize,
    /// 最大排队等待任务数
    max_queued: usize,
    /// 当前正在执行的任务数
    current: usize,
    /// 等待队列（FIFO）
    queue: VecDeque<QueuedTask>,
}

/// 排队中的任务信息
struct QueuedTask {
    /// 任务唯一 ID
    task_id: String,
    /// 视频生成参数
    params: VideoParams,
    /// 流水线停止步骤（如 "script"、"terms"、"audio" 等，用于调试/预览）
    stop_at: String,
}

impl TaskQueue {
    /// 创建新的任务队列
    ///
    /// 参数：
    /// - `max_concurrent`: 最大并发执行数
    /// - `max_queued`: 最大排队等待数
    pub fn new(max_concurrent: usize, max_queued: usize) -> Self {
        Self { max_concurrent, max_queued, current: 0, queue: VecDeque::new() }
    }

    /// 添加任务到队列
    ///
    /// 逻辑：
    /// - 若当前并发数 < max_concurrent，立即在新线程中启动任务执行
    /// - 否则若排队数 < max_queued，将任务加入等待队列
    /// - 否则返回 TaskQueueFull 错误
    ///
    /// 返回：成功返回 Ok(())，队列满返回 SomaError::TaskQueueFull
    pub fn add_task(&mut self, task_id: String, params: VideoParams, stop_at: String) -> Result<(), SomaError> {
        if self.current < self.max_concurrent {
            // 并发数未满，直接启动执行
            self.current += 1;
            let tid = task_id.clone();
            let p = params.clone();
            let sa = stop_at.clone();
            // 在新线程中执行流水线任务
            std::thread::spawn(move || {
                let result = service::pipeline::run_task(&tid, &p, &sa);
                if let Err(e) = result {
                    log!("task {} failed: {:?}", tid, e);
                    // 执行失败时更新任务状态为 Failed
                    state::update_task(&tid, Some(TaskStatus::Failed.as_i32()), None);
                }
                // 任务完成（无论成功失败），通知队列释放并发槽位
                TASK_QUEUE.lock().unwrap().task_done();
            });
            Ok(())
        } else if self.queue.len() < self.max_queued {
            // 并发已满但排队未满，加入等待队列
            self.queue.push_back(QueuedTask { task_id, params, stop_at });
            Ok(())
        } else {
            // 队列也满，拒绝新任务
            Err(SomaError::TaskQueueFull)
        }
    }

    /// 任务完成回调，释放并发槽位并检查是否有排队任务可启动
    fn task_done(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
        // 尝试从等待队列中取下一个任务执行
        self.check_queue();
    }

    /// 检查等待队列，若并发有空位则启动下一个排队任务
    fn check_queue(&mut self) {
        if self.current < self.max_concurrent {
            if let Some(task) = self.queue.pop_front() {
                self.current += 1;
                let tid = task.task_id.clone();
                let p = task.params.clone();
                let sa = task.stop_at.clone();
                std::thread::spawn(move || {
                    let result = service::pipeline::run_task(&tid, &p, &sa);
                    if let Err(e) = result {
                        log!("task {} failed: {:?}", tid, e);
                        state::update_task(&tid, Some(TaskStatus::Failed.as_i32()), None);
                    }
                    TASK_QUEUE.lock().unwrap().task_done();
                });
            }
        }
    }
}

/// 初始化任务队列，设置最大并发数和排队数
///
/// 应在服务启动时调用，根据配置设置队列参数
pub fn init_queue(max_concurrent: usize, max_queued: usize) {
    *TASK_QUEUE.lock().unwrap() = TaskQueue::new(max_concurrent, max_queued);
}

/// 向全局任务队列添加新任务
///
/// 参数：
/// - `task_id`: 任务 ID
/// - `params`: 视频生成参数
/// - `stop_at`: 流水线停止步骤
///
/// 返回：成功返回 Ok(())，队列满返回错误
pub fn add_task(task_id: &str, params: &VideoParams, stop_at: &str) -> Result<(), SomaError> {
    TASK_QUEUE.lock().unwrap().add_task(task_id.to_string(), params.clone(), stop_at.to_string())
}
