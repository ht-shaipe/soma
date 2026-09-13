# soma-server

**HTTP API 宿主与任务编排器**：面向服务器/CI 部署模式，承载全部业务处理、任务队列与持久化。
桌面宿主（soma-app）直接复用本 crate 的 handler/service/state 层，不经过 HTTP。

## 模块地图

```
main.rs            启动序列：配置 → 代理 → 日志 → 队列 → 存储目录 → SQLite → HTTP 服务
router.rs          统一分发：POST /api/v1/{module}/{method} → handler
handler/           各模块 API 处理器（videos/tasks/llm/config/voices/features/digital_human…）
service/           业务层
  ├─ pipeline.rs       视频流水线编排器：按序调用功能点 + 断点续跑 + stop_at 断点
  ├─ registry.rs       服务端功能点注册表（内置 21 个 + 数字人 3 个）
  ├─ llm.rs            LLM 服务兼容薄封装（实现已迁至 soma-feature）
  ├─ digital_human.rs  数字人三阶段流水线（音频 → 口播视频 → 合成）
  ├─ heygem_merchant.rs / heygem_trainer.rs / live2d_model.rs   数字人资产管理
  └─ segment_dh_video.rs   长文案分段口播流程
state.rs + store.rs    任务状态管理（内存 / SQLite 双后端，WAL）
task.rs                任务队列：并发上限 + FIFO 排队，工作线程执行
```

## 功能点双宿主

`service/registry.rs` 持有全局 `FeatureRegistry`：内置功能点来自 soma-feature，
数字人功能点在服务端注册（依赖任务状态存储）。
`handler/features.rs` 暴露 `features/list | run | history` 三个端点。

## 兼容性约定

- 旧 API 全部保留：`service/llm.rs` 为原签名薄封装；`pipeline::generate_audio*` /
  `get_video_materials` 供数字人服务与 handler 复用
- 任务失败语义：队列层捕获 Err/panic → 任务标记 Failed + 错误信息落库

## 运行

```bash
cargo run -p soma-server            # 默认 conf/config.toml，端口 8090
cargo run -p soma-server -c 自定路径.toml
```
