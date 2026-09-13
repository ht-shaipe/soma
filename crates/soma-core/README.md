# soma-core

Soma 的核心基础库：**配置、数据模型、错误类型与通用工具**，被其余所有 crate 依赖。

## 职责

- **配置管理**（`config`）：`AppConfig` 及各配置段（LLM / TTS 提供商 / 素材站 / Whisper / 数字人 / UI 等），
  TOML 装载与带默认值的 Getter。可选配置段缺省时可解析（`#[serde(default)]`），部分配置文件不会导致整体失败。
- **数据模型**（`models`）：任务状态机 `TaskStatus`、视频参数 `VideoParams`、数字人参数 `DigitalHumanParams`、
  分镜 `StoryboardScene`、AI 视频日志 `AiVideoSegmentLog`、商户资产 `MerchantAsset` 等。
  模型派生 `JsonSchema`（schemars），供功能点 Schema 自动生成。
- **错误类型**（`error`）：统一错误枚举 `SomaError`（thiserror），覆盖配置/LLM/TTS/素材/FFmpeg/功能点等全部链路。
- **工具函数**（`utils`）：存储目录约定（`storage_dir` / `task_dir`）、路径安全校验、SRT 生成、敏感词过滤（`filter`）。

## 目录约定

产物统一落在工作目录下：`storage/tasks/{task_id}/`（任务）、`storage/features/{feature}/{run_id}/`（功能点）、
`storage/cache_videos/`（素材缓存）、`conf/config.toml`（配置）。

## 依赖关系

```
soma-core ← soma-tts / soma-stock / soma-video / soma-feature / soma-server / soma-app
```

只依赖 serde / schemars / chrono / thiserror 等基础库，不反向依赖任何业务 crate。

## 测试

```bash
cargo test -p soma-core
```
