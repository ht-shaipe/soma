# Soma 桌面应用（Tauri v2 宿主）

0.1.2 升级计划 M2.2/M2.3/M2.4 产物：桌面模式直接复用 `soma-server` 业务层，
Vue 前端零改动运行在浏览器（HTTP）与桌面（invoke）两种环境；
配置与存储已迁移至系统目录，全新机器首次启动开箱可用。

## 运行

前置：`cargo tauri-cli` 已安装（`cargo tauri --version` ≥ 2.9）。

```bash
# 桌面应用（自包含：前端产物已内嵌，无需 vite / soma-server）
cargo run -p soma-app
# 或
cd crates/soma-app && cargo tauri dev
```

桌面模式下 API 请求走 `invoke('api', { module, method, payload })`，
由桌面宿主内置的分发命令直达 handler/service 层（与 HTTP 同构的信封），
无需另起 soma-server，也不依赖任何 node 进程。

修改前端后需要两步重建：`npm --prefix web run build`（产出 web/dist）→ `cargo build -p soma-app`（重新内嵌）。
需要浏览器联调时再手动起 `npm --prefix web run dev`（vite 代理 /api → :8090，需另起 soma-server）。

## 配置与存储位置（M2.4）

| 内容 | 桌面模式 | 服务器模式 |
|------|---------|-----------|
| 配置文件 | 系统配置目录 `config.toml`（macOS: `~/Library/Application Support/com.soma.desktop/`） | 项目内 `conf/config.toml` |
| 任务/产物存储 | 系统数据目录 `storage/`（macOS: `~/Library/Application Support/com.soma.desktop/storage/`） | 项目内 `./storage/` |

- 首次启动自动初始化：优先迁移项目内既有 `conf/config.toml`（保留密钥等配置），
  其次复制 `conf/config.toml.example` 模板，最后写出厂默认配置
- 旧任务历史（媒体文件）不做静默拷贝，仍保留在项目 `./storage/`，可经服务器模式访问
- 开发态桌面运行仍会将工作目录切到项目根，保证 `resource/` 相对路径（字体/歌曲/推理脚本）可用

## 架构说明

```
Vue3 前端（web/，api/index.ts Transport 双轨）
  ├─ 浏览器：axios → vite 代理 → soma-server (HTTP)
  └─ 桌面：  invoke('api') → crates/soma-app 通用分发命令
                              └─ soma-server handler/service/state（直接复用）
                                   └─ soma-feature 功能点 / 任务队列 / SQLite
```

- `src/main.rs`：后端初始化在 Tauri `setup` 钩子中执行——`init_backend_system`
  解析系统目录并做首启迁移，`init_backend_common` 复刻服务端启动序列
  （配置→代理→队列→存储目录→SQLite）；`api` 通用命令与 `router.rs` 路由表
  一一对应（handler 的 future 非 Send，在独立原生线程 + LocalSet 上执行）
- `soma-core::utils` 路径覆盖层：桌面宿主注入 `set_storage_root`，全部
  `storage_dir()/resource_dir()` 调用随之指向系统目录；服务器模式不注入，行为不变
- `tauri.conf.json`：`frontendDist` 指向 `web/dist`（内嵌完整 Vue 界面，运行零外部依赖）
- `capabilities/default.json`：主窗口能力（core:default + dialog）

## 当前已知限制

- `upload` 模块（multipart 文件上传）在桌面模式返回明确提示；工作台文件字段
  已走系统文件对话框，其余上传入口待 M3.3 素材库管理统一解决
- 任务视频在线播放依赖 `/storage` 静态服务：桌面模式下如需播放任务视频，
  需同时运行 soma-server，或等 M3 接入 Tauri asset 协议（`convertFileSrc`）
- `resource/` 相对路径（推理脚本/字体/歌曲）在打包态需 M3.4 资源打包
  （`set_resource_root` 覆盖层已预留）；当前打包态由 M2.5 预检给出引导
