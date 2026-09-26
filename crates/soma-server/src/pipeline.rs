use crate::service::pipeline as svc_pipeline;
/// 6步视频生成流水线编排模块
///
/// 定义了 Pipeline 结构体，委托实际的流水线逻辑到 service::pipeline::run_task。
/// 流水线步骤：
/// 1. 生成脚本（generate_script）
/// 2. 提取关键词（generate_terms）
/// 3. 生成音频（generate_audio）
/// 4. 生成字幕（generate_subtitle）
/// 5. 获取视频素材（get_video_materials）
/// 6. 生成最终视频（generate_final_videos）
use soma_core::models::VideoParams;
use tube::Result;

/// 流水线结构体，委托到 service::pipeline::run_task
pub struct Pipeline;

impl Default for Pipeline {
    fn default() -> Self {
        Self::new()
    }
}

impl Pipeline {
    pub fn new() -> Self {
        Self
    }

    /// 执行完整的6步视频生成流水线
    ///
    /// - `task_id`: 任务 ID
    /// - `params`: 视频生成参数
    /// - `stop_at`: 停止步骤（用于调试/预览中间结果）
    pub async fn run(&self, task_id: &str, params: &VideoParams, stop_at: &str) -> Result<()> {
        let _conf = crate::Config::get();
        svc_pipeline::run_task(task_id, params, stop_at).map_err(|e| tube::error!("{}", e))?;
        Ok(())
    }
}
