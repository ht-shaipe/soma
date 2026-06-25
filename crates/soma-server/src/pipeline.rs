use soma_core::models::VideoParams;
use tube::Result;

pub struct Pipeline;

impl Pipeline {
    pub fn new() -> Self {
        Self
    }

    pub async fn run(&self, _task_id: &str, _params: &VideoParams, _stop_at: &str) -> Result<()> {
        todo!("6步流水线编排实现")
    }
}
