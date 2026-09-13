# Soma Web 前端（Vue 3 + TypeScript + Element Plus）

Soma 的唯一前端代码库，同时服务两种宿主：浏览器（HTTP）与 Tauri 桌面（invoke）。

## 技术栈

Vue 3 `<script setup>` · TypeScript · Vite 8 · Element Plus · Pinia · Vue Router · Vue I18n（中/英） · Sass

## 双宿主 Transport（`src/api/index.ts`）

- 浏览器开发：axios → vite 代理（`/api` → `http://localhost:8090`）→ soma-server
- Tauri 桌面：`invoke('api', { module, method, payload })` → 桌面宿主内置分发命令（soma-app）
- 判定标志：`__TAURI_INTERNALS__`（v2 运行时无条件注入）；`FormData` 上传桌面模式暂不支持

业务 API 模块（`video.ts` / `llm.ts` / …）统一调用 `api.post('/{module}/{method}', payload)`
并经 `extractData` 解包 `{code, result, message}` 信封——**换宿主零改动**。

## 目录结构

```
src/
├── api/          API 封装（Transport 双轨 + 按模块拆分）
├── components/   业务组件（StepWizard 创作向导、设置面板、任务组件…）
├── layout/       应用壳（AppSidebar / AppHeader，Soma Studio 深色设计系统）
├── locales/      i18n 词条（zh-CN / en-US）
├── router/       路由（/ 视频 · /workbench 工作台 · /tasks 任务 · /settings 设置）
├── stores/       Pinia（config / task / material / videoParams）
├── styles/       设计令牌与全局样式（global.scss）
├── types/        领域类型（TaskInfo / VideoParams / StoryboardScene…）
└── views/        页面（HomeView 向导 · WorkbenchView 功能点工作台 · TasksView · SettingsView）
```

## 命令

```bash
npm install            # 安装依赖
npm run dev            # 开发（:5273，代理 /api 到 :8090）
npm run build          # 类型检查 + 产物构建（输出 dist/，供 Tauri 内嵌）
```

## 与 Tauri 桌面端的关系

`crates/soma-app` 内嵌本目录 `dist/` 作为界面。**改前端后需两步重建**：
`npm run build` → `cargo build -p soma-app`。桌面模式 API 不走 HTTP，
由 soma-app 的通用分发命令直达业务层，详见 `crates/soma-app/README.md`。
