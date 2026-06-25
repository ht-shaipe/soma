use soma_core::error::SomaError;
use soma_core::models::{TaskStatus, VideoParams, VideoConcatMode};
use crate::state;
use crate::service;
use std::sync::Mutex;
use std::collections::VecDeque;

lazy_static! {
    pub static ref TASK_QUEUE: Mutex<TaskQueue> = Mutex::new(TaskQueue::new(5, 100));
}

pub struct TaskQueue {
    max_concurrent: usize,
    max_queued: usize,
    current: usize,
    queue: VecDeque<QueuedTask>,
}

struct QueuedTask {
    task_id: String,
    params: VideoParams,
    stop_at: String,
}

impl TaskQueue {
    pub fn new(max_concurrent: usize, max_queued: usize) -> Self {
        Self { max_concurrent, max_queued, current: 0, queue: VecDeque::new() }
    }

    pub fn add_task(&mut self, task_id: String, params: VideoParams, stop_at: String) -> Result<(), SomaError> {
        if self.current < self.max_concurrent {
            self.current += 1;
            let tid = task_id.clone();
            let p = params.clone();
            let sa = stop_at.clone();
            std::thread::spawn(move || {
                let result = service::pipeline::run_task(&tid, &p, &sa);
                if let Err(e) = result {
                    log!("task {} failed: {:?}", tid, e);
                    state::update_task(&tid, Some(TaskStatus::Failed.as_i32()), None);
                }
                TASK_QUEUE.lock().unwrap().task_done();
            });
            Ok(())
        } else if self.queue.len() < self.max_queued {
            self.queue.push_back(QueuedTask { task_id, params, stop_at });
            Ok(())
        } else {
            Err(SomaError::TaskQueueFull)
        }
    }

    fn task_done(&mut self) {
        if self.current > 0 {
            self.current -= 1;
        }
        self.check_queue();
    }

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

pub fn init_queue(max_concurrent: usize, max_queued: usize) {
    *TASK_QUEUE.lock().unwrap() = TaskQueue::new(max_concurrent, max_queued);
}

pub fn add_task(task_id: &str, params: &VideoParams, stop_at: &str) -> Result<(), SomaError> {
    TASK_QUEUE.lock().unwrap().add_task(task_id.to_string(), params.clone(), stop_at.to_string())
}
