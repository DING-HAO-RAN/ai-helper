# AI Helper 辅助工具系统架构与详细设计规范

## 1. 项目背景与目标
本项目旨在打造一个轻量、极低性能开销、赛博朋克现代视觉风格的 AI 辅助工具桌面应用，兼备命令行 AI 访问集成能力。

### 核心目标与成功标准
1. **极轻量与低资源占用**：
   - 基于 Tauri v2 + Rust 后端 + Vue 3 / Vite 现代前端，内存占用控制在 30~60MB，安装包体小（~15MB），启动秒级。
2. **双模合一（GUI + CLI）**：
   - GUI 模式：无 Windows 原生标题栏，自定义赛博朋克控制台，分栏布局。
   - CLI 模式：同一二进制程序通过命令行参数（如 `ai-helper token get`）直接静默输出解密令牌，专供 AI Agent 与终端脚本快速安全调用。
3. **提示词库整理（Prompt Vault）**：
   - 顶部大文本框快速输入/编辑，支持一键清空、粘贴。
   - 中下方可垂直滚动提示词列表，长条卡片含标题、预览、一键复制、修改回填、删除三联按钮。
4. **GitHub 令牌安全存储（GitHub Token Vault）**：
   - 使用 Windows DPAPI 操作系统级安全加密，本地存储加密数据。
   - 支持多令牌别名管理、掩码展示、复制、连通性检测、默认激活项。
5. **AGENT.md 智能扫描与管理（Agent Matrix）**：
   - 快速全盘扫描（多线程智能排除系统目录与体积庞大的缓存文件夹）。
   - 识别 AI 工具及项目工作区的 `agent.md`。
   - 提供长条卡片查看、定位及**一键将 GitHub 访问令牌指引安全写入所有 AGENT.md**（采用防重防破坏的标记块）。
6. **后台运行与赛博悬浮球（Cyber Floating Orb）**：
   - 关闭主窗口后默认最小化常驻后台，并唤起置顶半透明赛博悬浮球。
   - 悬浮球全屏拖拽，左键点击呼回主窗口，右键呼出赛博快捷菜单。

---

## 2. 详细技术架构与模块设计

### 2.1 目录结构与技术栈
- **后端架构**：Rust (Tauri v2 + serde + windows-dpapi + walkdir / rayon + directories)
- **前端架构**：Vue 3 + Vite + TypeScript + 赛博朋克定制 CSS / Lucide 图标
- **工程结构**：
  ```
  AI helper/
  ├── src-tauri/               # Rust 后端
  │   ├── src/
  │   │   ├── main.rs          # 启动入口与 CLI 参数分支解析
  │   │   ├── app.rs           # Tauri 运行时与主应用配置
  │   │   ├── crypto/          # Windows DPAPI 安全加解密模块
  │   │   ├── storage/         # 提示词与令牌数据存储
  │   │   ├── scanner/         # AGENT.md 快速全盘扫描与批量注入模块
  │   │   └── commands/        # Tauri 命令暴露层
  │   ├── Cargo.toml
  │   └── tauri.conf.json      # 双窗口配置（main + floating_ball）
  ├── src/                     # Vue 3 前端
  │   ├── components/
  │   │   ├── TitleBar.vue     # 自定义赛博朋克标题栏
  │   │   ├── Sidebar.vue      # 左侧导航栏与系统设置
  │   │   ├── PromptVault.vue  # 提示词整理主面板
  │   │   ├── TokenVault.vue   # GitHub 令牌管理面板
  │   │   ├── AgentManager.vue # AGENT.md 扫描与一键注入面板
  │   │   └── FloatingBall.vue # 独立悬浮球视图
  │   ├── styles/
  │   │   └── cyberpunk.css    # 赛博朋克全局科技感样式
  │   ├── App.vue              # 主窗口容器
  │   └── main.ts
  └── package.json
  ```

### 2.2 CLI 模式设计
- 执行格式：
  - `ai-helper token get` -> 输出当前默认 token 明文（无多余文本，便 AI 工具直接管道读取）
  - `ai-helper token get <alias>` -> 输出指定别名 token
  - `ai-helper token list` -> 输出 JSON 或紧凑列表
- CLI 拦截逻辑：在 `main.rs` 最起始处检测 `std::env::args()`，若匹配子命令则直接执行 Rust 解密后打印退出，避免启动 GUI WebView2，保证在 20ms 内完成执行。

### 2.3 Windows DPAPI 安全加密规范
- 调用 Windows 原生 `CryptProtectData` 与 `CryptUnprotectData` API。
- 加密密文保存在本地应用数据目录的 `credentials.enc` 或 JSON 结构中。
- 只有当前计算机上的当前 Windows 账户能够成功解密。

### 2.4 AGENT.md 扫描与注入规范
- **排除规则**：跳过 `C:\Windows`, `Program Files`, `node_modules`, `.git`, `AppData\Local\Temp`, `target`, `dist` 等。
- **匹配规则**：匹配文件名大小写不敏感 `agent.md` 与 `agents.md`。
- **写入规范**：
  ```markdown
  <!-- AI-HELPER:GITHUB-CREDENTIALS-START -->
  ## GitHub Access Guide
  To access GitHub repositories or APIs with authorized credentials:
  Use local helper command to retrieve the active token:
  ```powershell
  $env:GITHUB_TOKEN = (ai-helper token get)
  ```
  Or in Bash:
  ```bash
  export GITHUB_TOKEN=$(ai-helper token get)
  ```
  <!-- AI-HELPER:GITHUB-CREDENTIALS-END -->
  ```

### 2.5 悬浮球与窗口生命周期
- **启动时**：加载主窗口，悬浮球窗口预加载但初始设为隐藏。
- **主窗口关闭事件（on_window_event CloseRequested）**：
  - 拦截关闭事件，执行 `main_window.hide()`。
  - 获取屏幕工作区尺寸，将悬浮球定位在屏幕右上边缘（如 x: width - 80, y: 150），并调用 `floating_window.show()`。
- **悬浮球操作**：
  - 左键点击：`floating_window.hide()`，`main_window.show()`，`main_window.set_focus()`。
  - 右键点击：展示上下文菜单。
