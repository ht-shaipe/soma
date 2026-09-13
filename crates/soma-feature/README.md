# soma-feature

**功能点（Feature）抽象层**——0.1.2 架构改造的核心：把流水线中的每个能力拆分为可独立注册、
独立调用、独立产出产物的"功能点"，供多个宿主（soma-server HTTP / soma-app 桌面）零复制复用。

## 核心概念

| 类型 | 说明 |
|------|------|
| `Feature` | 功能点抽象：JSON 信封输入输出，`descriptor()` 携带输入输出 Schema |
| `TypedFeature` | 强类型功能点：serde 结构体定义即 Schema（schemars 自动生成），blank impl 桥接为 `Feature` |
| `FeatureRegistry` | 注册表与统一运行器：定位 → 上下文 → 执行（捕获 panic）→ 产物登记 → 运行记录落盘 |
| `FeatureContext` | 单次运行上下文：AppConfig、产物目录、产物登记表 |
| `ProgressReporter` | 进度上报抽象；宿主各自实现推送（HTTP 轮询 / Tauri event） |

## 内置功能点（24 个）

- **LLM**：`llm.intent` / `llm.script` / `llm.storyboard` / `llm.narration` / `llm.terms` / `llm.social`
- **TTS**：`tts.synthesize`（多引擎路由 + 声音克隆）
- **音频**：`audio.concat`
- **字幕**：`subtitle.generate`
- **素材**：`material.generate` / `material.search` / `material.download`
- **AI 视频**：`aivideo.generate`
- **视频合成**：`video.compose`（一键全流程）
- **视频处理**：`video.concat` / `video.render` / `video.watermark` / `video.join` / `video.transition` / `video.clip_resize` / `video.info`
- **数字人**：由 soma-server 宿主注册（依赖任务状态存储）

## 产物与历史

每次运行归档到 `storage/features/{feature_id}/{run_id}/`：
产物文件 + `input.json`（原始入参）+ `run_record.json`（`FeatureOutput` 运行记录），
`registry.list_runs()` 直接读取目录即得历史，无需额外索引。

## 模块导航

- `llm.rs`：LLM 服务（供 `llm.*` 功能点与宿主兼容层共用）
- `features/`：全部内置功能点实现（按分类分子模块）
- `runtime.rs`：`block_on_async`（阻塞桥接 async）与 `retry`（指数退避）

## 测试

```bash
cargo test -p soma-feature
```
