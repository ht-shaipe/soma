# Soma 桌面应用（Tauri v2 宿主）

0.1.2 升级计划 M2.2/M2.3 产物：桌面模式直接复用 `soma-server` 业务层，
Vue 前端零改动运行在浏览器（HTTP）与桌面（invoke）两种环境。

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

## 架构说明

```
Vue3 前端（web/，api/index.ts Transport 双轨）
  ├─ 浏览器：axios → vite 代理 → soma-server (HTTP)
  └─ 桌面：  invoke('api') → crates/soma-app 通用分发命令
                              └─ soma-server handler/service/state（直接复用）
                                   └─ soma-feature 功能点 / 任务队列 / SQLite
```

- `src/main.rs`：`init_backend()` 复刻服务端启动序列（配置/代理/队列/SQLite），
  `api` 通用命令与 `router.rs` 路由表一一对应（handler 的 future 非 Send，
  在独立原生线程 + LocalSet 上执行）
- `tauri.conf.json`：`frontendDist` 指向 `web/dist`（内嵌完整 Vue 界面，运行零外部依赖）
- `capabilities/default.json`：主窗口默认能力（core:default）

## 当前已知限制

- 文件上传（素材/音乐/人像）在桌面模式返回明确提示，M2.4 通过本地文件对话框解决
- 任务视频在线播放依赖 `/storage` 静态服务：桌面模式下如需播放任务视频，
  需同时运行 soma-server，或等 M3 接入 Tauri asset 协议（`convertFileSrc`）
- 配置/存储路径仍为项目相对路径（`conf/`、`storage/`），M2.4 迁移到系统目录
