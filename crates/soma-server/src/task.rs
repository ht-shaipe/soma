use crate::service;
use crate::state;
/// 任务队列调度模块
///
/// 实现带并发限制和排队机制的任务调度器。当当前执行任务数未达到最大并发数时，
/// 新任务立即在新线程中启动执行；超出并发限制时任务进入等待队列；
/// 队列也满时返回错误。任务完成后自动从队列中取下一个任务执行。
use soma_core::error::SomaError;
use soma_core::models::{DigitalHumanParams, ImageStoryParams, TaskStatus, VideoParams};
use std::collections::VecDeque;
use std::sync::Mutex;

lazy_static! {
    /// 全局任务队列实例，默认并发数5、排队数100，可通过 init_queue 重新初始化
    pub static ref TASK_QUEUE: Mutex<TaskQueue> = Mutex::new(TaskQueue::new(5, 100));
}

/// 安全获取 TASK_QUEUE 的 MutexGuard，中毒时恢复
fn lock_queue() -> std::sync::MutexGuard<'static, TaskQueue> {
    TASK_QUEUE.lock().unwrap_or_else(|e| {
        log::error!("TASK_QUEUE Mutex 中毒，强制恢复: {}", e);
        e.into_inner()
    })
}

/// 任务队列结构体，管理并发执行和排队等待的任务
pub struct TaskQueue {
    /// 最大并发执行任务数
    max_concurrent: usize,
    /// 最大排队等待任务数
    max_queued: usize,
    /// 当前正在执行的任务数
    current: usize,
    /// 等待队列（FIFO），支持视频任务和数字人任务
    queue: VecDeque<QueuedTask>,
}

/// 排队中的任务信息（视频任务、数字人任务或图片故事任务）
#[allow(clippy::large_enum_variant)] // Video 变体携带完整 VideoParams，装箱会拖累所有构造点
enum QueuedTask {
    Video {
        task_id: String,
        params: VideoParams,
        stop_at: String,
    },
    DigitalHuman {
        task_id: String,
        params: DigitalHumanParams,
    },
    ImageStory {
        task_id: String,
        params: ImageStoryParams,
    },
}

/// 在独立线程中执行任务，完成后释放并发槽位
fn spawn_task(task: QueuedTask) {
    match task {
        QueuedTask::Video {
            task_id,
            params,
            stop_at,
        } => {
            std::thread::spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    service::pipeline::run_task(&task_id, &params, &stop_at)
                }));
                handle_task_result(&task_id, result);
                lock_queue().task_done();
            });
        }
        QueuedTask::DigitalHuman { task_id, params } => {
            std::thread::spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    service::digital_human::run_task(&task_id, &params)
                }));
                handle_dh_task_result(&task_id, result);
                lock_queue().task_done();
            });
        }
        QueuedTask::ImageStory { task_id, params } => {
            std::thread::spawn(move || {
                let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    service::image_story::run_task(&task_id, &params)
                }));
                handle_image_story_task_result(&task_id, result);
                lock_queue().task_done();
            });
        }
    }
}

/// 处理视频任务执行结果
fn handle_task_result(task_id: &str, result: std::thread::Result<Result<(), SomaError>>) {
    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            log::error!("task {} failed: {:?}", task_id, e);
            state::update_task_data(
                task_id,
                &state::TaskUpdateData {
                    state: Some(TaskStatus::Failed.as_i32()),
                    error_message: Some(format!("{:?}", e)),
                    ..Default::default()
                },
            );
        }
        Err(panic_val) => {
            let msg = panic_msg(panic_val);
            log::error!("task {} panicked: {}", task_id, msg);
            state::update_task_data(
                task_id,
                &state::TaskUpdateData {
                    state: Some(TaskStatus::Failed.as_i32()),
                    error_message: Some(format!("pipeline panicked: {}", msg)),
                    ..Default::default()
                },
            );
        }
    }
}

/// 处理数字人任务执行结果
fn handle_dh_task_result(task_id: &str, result: std::thread::Result<Result<(), SomaError>>) {
    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            log::error!("dh task {} failed: {:?}", task_id, e);
            state::update_dh_task_data(
                task_id,
                &state::DhTaskUpdateData {
                    state: Some(TaskStatus::Failed.as_i32()),
                    error_message: Some(format!("{:?}", e)),
                    ..Default::default()
                },
            );
        }
        Err(panic_val) => {
            let msg = panic_msg(panic_val);
            log::error!("dh task {} panicked: {}", task_id, msg);
            state::update_dh_task_data(
                task_id,
                &state::DhTaskUpdateData {
                    state: Some(TaskStatus::Failed.as_i32()),
                    error_message: Some(format!("pipeline panicked: {}", msg)),
                    ..Default::default()
                },
            );
        }
    }
}

/// 处理图片故事任务执行结果
fn handle_image_story_task_result(
    task_id: &str,
    result: std::thread::Result<Result<(), SomaError>>,
) {
    match result {
        Ok(Ok(())) => {}
        Ok(Err(e)) => {
            log::error!("image story task {} failed: {:?}", task_id, e);
            state::update_image_story_task_data(
                task_id,
                Some(TaskStatus::Failed.as_i32()),
                None,
                None,
                Some(format!("{:?}", e)),
            );
        }
        Err(panic_val) => {
            let msg = panic_msg(panic_val);
            log::error!("image story task {} panicked: {}", task_id, msg);
            state::update_image_story_task_data(
                task_id,
                Some(TaskStatus::Failed.as_i32()),
                None,
                None,
                Some(format!("pipeline panicked: {}", msg)),
            );
        }
    }
}

/// 从 panic 值提取消息
fn panic_msg(panic_val: Box<dyn std::any::Any + Send>) -> String {
    if let Some(s) = panic_val.downcast_ref::<&str>() {
        s.to_string()
    } else if let Some(s) = panic_val.downcast_ref::<String>() {
        s.clone()
    } else {
        "unknown panic".to_string()
    }
}

impl TaskQueue {
    pub fn new(max_concurrent: usize, max_queued: usize) -> Self {
        Self {
            max_concurrent,
            max_queued,
            current: 0,
            queue: VecDeque::new(),
        }
    }

    /// 添加视频任务到队列
    pub fn add_task(
        &mut self,
        task_id: String,
        params: VideoParams,
        stop_at: String,
    ) -> Result<(), SomaError> {
        self.enqueue(QueuedTask::Video {
            task_id,
            params,
            stop_at,
        })
    }

    /// 添加数字人任务到队列
    pub fn add_dh_task(
        &mut self,
        task_id: String,
        params: DigitalHumanParams,
    ) -> Result<(), SomaError> {
        self.enqueue(QueuedTask::DigitalHuman { task_id, params })
    }

    /// 添加图片故事任务到队列
    pub fn add_image_story_task(
        &mut self,
        task_id: String,
        params: ImageStoryParams,
    ) -> Result<(), SomaError> {
        self.enqueue(QueuedTask::ImageStory { task_id, params })
    }

    /// 通用入队逻辑：并发有空位则立即执行，否则入队等待，队列满则报错
    fn enqueue(&mut self, task: QueuedTask) -> Result<(), SomaError> {
        if self.current < self.max_concurrent {
            self.current += 1;
            spawn_task(task);
            Ok(())
        } else if self.queue.len() < self.max_queued {
            self.queue.push_back(task);
            Ok(())
        } else {
            Err(SomaError::TaskQueueFull)
        }
    }

    /// 任务完成回调，释放并发槽位并检查是否有排队任务可启动
    fn task_done(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
        self.check_queue();
    }

    /// 检查等待队列，若并发有空位则启动下一个排队任务
    fn check_queue(&mut self) {
        while self.current < self.max_concurrent {
            if let Some(task) = self.queue.pop_front() {
                self.current += 1;
                spawn_task(task);
            } else {
                break;
            }
        }
    }
}

/// 初始化任务队列，设置最大并发数和排队数
pub fn init_queue(max_concurrent: usize, max_queued: usize) {
    *lock_queue() = TaskQueue::new(max_concurrent, max_queued);
}

/// 向全局任务队列添加新视频任务
pub fn add_task(task_id: &str, params: &VideoParams, stop_at: &str) -> Result<(), SomaError> {
    lock_queue().add_task(task_id.to_string(), params.clone(), stop_at.to_string())
}

/// 向全局任务队列添加新数字人任务
pub fn add_dh_task(task_id: &str, params: &DigitalHumanParams) -> Result<(), SomaError> {
    lock_queue().add_dh_task(task_id.to_string(), params.clone())
}

/// 向全局任务队列添加新图片故事任务
pub fn add_image_story_task(task_id: &str, params: &ImageStoryParams) -> Result<(), SomaError> {
    lock_queue().add_image_story_task(task_id.to_string(), params.clone())
}
