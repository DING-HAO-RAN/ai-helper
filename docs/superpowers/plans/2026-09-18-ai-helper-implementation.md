# AI Helper 赛博朋克桌面工具与 CLI 实现计划

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 开发一款超轻量、低资源占用、赛博朋克风格的 AI 辅助桌面工具，集提示词管理、GitHub 令牌加密存储与 CLI 访问、全盘 AGENT.md 扫描与批量写入、后台运行与桌面悬浮球于一体。

**Architecture:** 采用 Tauri v2 + Rust 后端 + Vue 3 / Vite 前端双模架构。Rust `main.rs` 优先拦截命令行参数，毫秒级响应 AI agent 对 GitHub Token 的读取请求；GUI 采用无边框多窗口架构，主窗口与赛博朋克悬浮球无缝联动，使用 Windows DPAPI 原生安全加密与 Rust 多线程高速全盘扫描。

**Tech Stack:** Rust 2021, Tauri v2, Vue 3, Vite, TypeScript, Lucide Icons, Cyberpunk CSS.

**Spec:** `docs/superpowers/specs/2026-09-18-ai-helper-design.md`

## Global Constraints
- 全程使用中文注释与说明
- 极低资源消耗：非运行时完全不占用额外内存，GUI 运行时内存约 30~50MB
- 无 Windows 原生标题栏，赛博朋克霓虹光影与科技黑风格统一
- 严格遵循简单优先原则，不过度工程化，优先完成完整可用程序
- 任何工作树外的思考与临时文件均放置于 D 盘

---

### Task 1: 项目基础骨架搭建 (Vite + Vue 3 + Tauri v2)

**Files:**
- Create: `package.json`
- Create: `vite.config.ts`
- Create: `tsconfig.json`
- Create: `index.html`
- Create: `src-tauri/Cargo.toml`
- Create: `src-tauri/tauri.conf.json`
- Create: `src-tauri/build.rs`

**Interfaces:**
- Consumes: None
- Produces: 完整可运行的前后端构建配置

- [ ] **Step 1: 初始化 package.json 与前端依赖配置**
在根目录生成 `package.json`，配置 Vue 3, Vite, TypeScript, `@tauri-apps/api`, `@tauri-apps/plugin-shell` 等基础依赖。

- [ ] **Step 2: 配置 vite.config.ts 与 index.html**
配置 Vite 支持 Tauri 的端口（1420）与多路由视图模式。

- [ ] **Step 3: 配置 src-tauri/Cargo.toml 与 tauri.conf.json**
在 `src-tauri` 中配置 Tauri 核心 crate、serde、windows-sys / dpapi 加密、walkdir、dirs 等轻量高性能依赖；配置双窗口（`main` 960x650 无边框，`floating_ball` 64x64 透明置顶）。

- [ ] **Step 4: 验证脚手架编译配置**
运行 `pnpm install` 确保前端依赖安装就绪。

---

### Task 2: Rust 核心模块 —— DPAPI 安全加解密与本地存储库

**Files:**
- Create: `src-tauri/src/crypto/mod.rs`
- Create: `src-tauri/src/storage/mod.rs`
- Create: `src-tauri/src/storage/types.rs`

**Interfaces:**
- Produces:
  - `crypto::encrypt_text(plain: &str) -> Result<Vec<u8>, String>`
  - `crypto::decrypt_text(cipher: &[u8]) -> Result<String, String>`
  - `storage::load_data() -> AppData`
  - `storage::save_data(data: &AppData) -> Result<(), String>`

- [ ] **Step 1: 编写加密与解密单元测试**
测试字符串加密后能正确还原，且密文非明文。

- [ ] **Step 2: 编写基于 Windows DPAPI / 本地密保的加解密实现**
调用 Windows `CryptProtectData` 与 `CryptUnprotectData` API，实现安全可信的数据加密。

- [ ] **Step 3: 编写数据持久化层**
在用户 AppData/ai-helper 目录下保存 `prompts.json` 与 `credentials.enc`，提供简单快速的序列化与反序列化接口。

- [ ] **Step 4: 运行 `cargo test` 验证存储与加密通过**

---

### Task 3: Rust CLI 接口 —— 终端毫秒级调用支持

**Files:**
- Create: `src-tauri/src/cli/mod.rs`
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Consumes: `crypto::decrypt_text`, `storage::load_data`
- Produces: 命令行参数拦截器 `cli::handle_cli_args(args: &[String]) -> bool`

- [ ] **Step 1: 编写 CLI 参数解析逻辑**
支持 `ai-helper token get`（输出默认 token）、`ai-helper token get <alias>`（输出指定 token）、`ai-helper token list`（紧凑输出别名列表）。

- [ ] **Step 2: 在 main.rs 顶层做静默拦截**
若发现有效命令行参数，直接在标准输出（stdout）打印明文 token 并使用 `std::process::exit(0)` 退出，跳过任何窗口加载，实现 < 20ms 的极速响应。

- [ ] **Step 3: 测试 CLI 命令输出**
模拟执行命令行传参，验证标准输出中包含正确的 token。

---

### Task 4: Rust 核心模块 —— AGENT.md 全盘扫描与一键批量安全写入

**Files:**
- Create: `src-tauri/src/scanner/mod.rs`
- Create: `src-tauri/src/scanner/injector.rs`

**Interfaces:**
- Produces:
  - `scanner::scan_all_agent_files() -> Vec<AgentFileInfo>`
  - `injector::inject_token_guide(file_path: &str, helper_cmd: &str) -> Result<bool, String>`
  - `injector::inject_all(file_paths: &[String]) -> InjectSummary`

- [ ] **Step 1: 编写扫描器智能过滤逻辑**
获取所有系统逻辑盘符（C:, D:, etc.），排除 `Windows`, `System32`, `node_modules`, `.git`, `AppData/Local/Temp` 等大目录，针对 `agent.md` 与 `agents.md` 进行递归匹配。

- [ ] **Step 2: 编写防重复标记块批量注入模块**
定义专用标记块 `<!-- AI-HELPER:GITHUB-CREDENTIALS-START -->` ... `<!-- AI-HELPER:GITHUB-CREDENTIALS-END -->`，对已存在标记的文件做内容无缝更新，对未标记的文件进行文末追加。

- [ ] **Step 3: 运行单元测试验证扫描与写入正确性**

---

### Task 5: Tauri 命令层与双窗口生命周期管理

**Files:**
- Create: `src-tauri/src/commands/mod.rs`
- Modify: `src-tauri/src/app.rs`
- Modify: `src-tauri/src/main.rs`

**Interfaces:**
- Produces Tauri Invokable Commands:
  - `get_prompts`, `save_prompt`, `delete_prompt`
  - `get_tokens`, `save_token`, `delete_token`, `set_default_token`, `test_github_token`
  - `scan_agents`, `inject_agents`
  - `show_main_window`, `hide_to_orb`, `exit_app`

- [ ] **Step 1: 注册所有前后端交互 RPC 命令**
将提示词增删改、Token 加解密操作、Agent.md 扫描与注入接口暴露给前端。

- [ ] **Step 2: 实现主窗口与悬浮球窗口的显示/隐藏生命周期**
在主窗口捕获 `CloseRequested`，不退出程序，调用 `main.hide()` 并展示 `floating_ball.show()`；悬浮球点击调用 `floating_ball.hide()` 与 `main.show()`。

---

### Task 6: 赛博朋克全局 UI 主题与基础组件 (TitleBar & Sidebar)

**Files:**
- Create: `src/styles/cyberpunk.css`
- Create: `src/components/TitleBar.vue`
- Create: `src/components/Sidebar.vue`
- Create: `src/App.vue`
- Create: `src/main.ts`

**Interfaces:**
- Produces: 现代化暗黑赛博朋克页面基础容器、自定义无边框可拖拽标题栏、左侧分栏导航组件。

- [ ] **Step 1: 编写赛博朋克设计系统 CSS**
定义深黑底色、霓虹青蓝 (`#00f0ff`)、电光粉紫 (`#ff0055`)、数码亮绿、发光阴影与毛玻璃效果。

- [ ] **Step 2: 编写无边框 TitleBar.vue**
集成窗口拖拽区域（`data-tauri-drag-region`）、赛博朋克动态呼吸灯、最小化、最大化与关闭按钮（触发隐藏到悬浮球）。

- [ ] **Step 3: 编写 Sidebar.vue**
顶部为功能区切换（提示词库、GitHub 令牌、AGENT.md 矩阵），底部为系统设置与快捷呼叫状态指示。

---

### Task 7: 提示词宝库 (Prompt Vault) 功能界面与交互

**Files:**
- Create: `src/components/PromptVault.vue`
- Create: `src/types/prompt.ts`

**Interfaces:**
- Consumes: Tauri `get_prompts`, `save_prompt`, `delete_prompt`

- [ ] **Step 1: 编写顶部大文本框输入区**
支持标题输入、多行提示词大文本框、快速粘贴按钮、清空按钮、保存/更新按钮。

- [ ] **Step 2: 编写中下方可滚动长条卡片列表**
展示提示词名称、前100字符预览，提供【复制】（带发光粒子/弹窗提示）、【修改】（内容回填到顶部大文本框）、【删除】三联按钮。

- [ ] **Step 3: 实现数据的保存、回填与删除交互并测试联动**

---

### Task 8: GitHub 令牌保险库 (Token Vault) 功能界面与交互

**Files:**
- Create: `src/components/TokenVault.vue`
- Create: `src/types/token.ts`

**Interfaces:**
- Consumes: Tauri `get_tokens`, `save_token`, `delete_token`, `set_default_token`, `test_github_token`

- [ ] **Step 1: 编写添加令牌交互卡片**
支持输入别名（如 `work`、`default`）、GitHub Personal Access Token（PAT）、备注，点击保存调用 DPAPI 加密存储。

- [ ] **Step 2: 编写令牌列表长条卡片**
显示别名、掩码后密钥（支持小眼睛一键显隐）、默认激活徽章、复制按钮、一键测试连通性按钮（在线验证是否有效）、删除按钮。

- [ ] **Step 3: 编写 AI 命令行使用说明指南卡片**
在界面显著位置提供 `ai-helper token get` 的一键复制命令示例与 AI 配置引导。

---

### Task 9: AGENT.md 管理与一键批量注入面板

**Files:**
- Create: `src/components/AgentManager.vue`
- Create: `src/types/agent.ts`

**Interfaces:**
- Consumes: Tauri `scan_agents`, `inject_agents`

- [ ] **Step 1: 编写全盘扫描状态与触发控制区**
提供雷达旋转扫描动效、扫描耗时统计、已找到的 agent.md 文件总数计数。

- [ ] **Step 2: 编写扫描结果列表与操作长条卡片**
展示路径、文件大小、最后修改日期、已注入标记状态徽标；卡片右侧提供【定位目录】（在 Windows 资源管理器中打开）、【查看预览】按钮。

- [ ] **Step 3: 编写「一键写入 GitHub 令牌使用指引」主按钮与注入反馈**
点击批量注入，调用 Rust 后端将标准凭据获取指引写入所有文件，并实时更新列表状态与成功统计。

---

### Task 10: 赛博桌面悬浮球视图 (Floating Orb)

**Files:**
- Create: `src/components/FloatingBall.vue`
- Create: `src/floating.html` 或路由分支

**Interfaces:**
- Consumes: Tauri `show_main_window`, `exit_app`

- [ ] **Step 1: 编写悬浮球 UI 与霓虹旋转脉冲光晕**
64x64 圆形透明区域，发光科技核心与微动画，支持鼠标在桌面全屏拖拽移动（调用 Tauri drag API）。

- [ ] **Step 2: 实现左键与右键交互**
- 左键点击：平滑唤醒主窗口并自动聚焦，隐藏悬浮球。
- 右键点击：弹出赛博风格悬浮快捷菜单（【快速复制默认 Token】、【打开主控制台】、【退出程序】）。

---

### Task 11: 综合联调、构建与验证

- [ ] **Step 1: 运行本地开发环境测试所有功能**
- [ ] **Step 2: 测试命令行 CLI 模式 (`ai-helper token get`)**
- [ ] **Step 3: 测试全盘 agent.md 扫描与标记块注入**
- [ ] **Step 4: 测试关闭主窗口转悬浮球与右键菜单恢复**
