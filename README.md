# AI Helper // Cyber Matrix (赛博朋克 AI 辅助集成工具)

一款极度轻量、超低性能消耗、现代化赛博朋克风格的 AI 辅助桌面工具与命令行终端接口。

![AI Helper Core](/public/cyber-core.png)

## ✨ 核心特性

1. **双模合一架构（GUI + CLI）**：
   - **CLI 终端极速响应**：为终端脚本与 AI Agent 提供专用的命令行接口。例如 `ai-helper token get`，不启动图形界面，毫秒级直接输出解密后的 GitHub 令牌明文，管道调用极速无感知。
   - **赛博朋克现代 GUI**：深渊黑夜底色搭配霓虹青蓝、电光粉红、数码绿光影；全自绘赛博标题栏（无 Windows 默认原生边框）；左右分栏清晰布局。

2. **提示词整理宝库 (Prompt Vault)**：
   - 顶部大文本框快速编写/编辑，支持一键清空与剪贴板快速粘贴。
   - 中下方长条形提示词卡片列表，支持上下平滑垂直滚动。
   - 具备【复制】（带发光反馈）、【修改】（内容与名称一键回填至顶部编辑框）、【删除】（二次安全确认）三联操作。

3. **GitHub 令牌本地加密保险库 (Token Vault)**：
   - **操作系统级加密**：采用 Windows 原生 DPAPI（`CryptProtectData` / `CryptUnprotectData`）与 Base64 本地加密持久化，即使文件被盗也无法被其他机器或非当前用户解密。
   - **UI 交互**：多令牌别名管理、密钥掩码展示、一键切换显隐、明文复制、连通性实时检测（调用 GitHub `/user` 验证）、设置默认活动令牌。
   - **AI 工具与终端调用示例**：
     ```powershell
     # 获取默认 GitHub Token
     ai-helper token get

     # 获取指定别名 Token
     ai-helper token get work

     # 列出已保存的全部令牌
     ai-helper token list

     # 一键注入 PowerShell 环境变量
     $env:GITHUB_TOKEN = (ai-helper token get)

     # 一键注入 Bash / Git Bash 环境变量
     export GITHUB_TOKEN=$(ai-helper token get)
     ```

4. **AGENT.md 智能扫描与一键批量注入 (Agent Matrix)**：
   - **智能全盘扫描**：多线程遍历驱动器，自动剪枝跳过 `Windows`、`node_modules`、`.git`、`Temp` 等大型目录，秒级索引电脑中所有 AI 工具（Cursor、Windsurf、Cline、Continue 等）及项目工作区的 `agent.md` / `agents.md`。
   - **管理列表**：长条卡片呈现工具分类、文件大小、修改日期、内容摘要，提供【定位目录】（在 Windows 资源管理器中打开）、【查看/编辑】（内置 Markdown 编辑弹窗）。
   - **一键写入所有 AGENT.md**：一键将安全的 GitHub 访问凭据调用说明批量写入所有扫描出的 AGENT.md 中，采用专用防重注释标记块（`<!-- AI-HELPER:GITHUB-CREDENTIALS-START -->`），再次写入时自动更新，绝不破坏原有内容或重复追加。

5. **后台运行与桌面赛博悬浮球 (Cyber Floating Orb)**：
   - 关闭主窗口后默认隐入后台持续待命，并在屏幕右侧浮现 68×68 像素的置顶半透明赛博悬浮球。
   - **全屏拖拽**：支持在 Windows 桌面上任意拖拽位置。
   - **左键单击**：瞬间唤醒主控制台窗口并自动置顶聚焦。
   - **右键单击**：呼出赛博风格快捷上下文菜单，可直接一键复制默认 GitHub Token、打开主界面或彻底退出程序。

---

## 🛠️ 技术架构

- **后端**：Rust 2021 + Tauri v2 + Windows DPAPI + Tokio/Tauri Async Runtime + Walkdir
- **前端**：Vue 3 + Vite + TypeScript + Lucide Icons + Cyberpunk CSS
- **资源占用**：发布包体积仅约 15MB，运行时内存仅 30~50MB，无 Chromium 臃肿内核。

---

## 🚀 开发与编译

### 前置条件
- [Node.js](https://nodejs.org/) (>= 18) 与 [pnpm](https://pnpm.io/)
- [Rust](https://rustup.rs/) (>= 1.75)

### 本地运行
```bash
# 安装前端依赖
pnpm install

# 启动开发调试
pnpm tauri dev
```

### 生产打包
```bash
pnpm tauri build
```
编译产物位于 `src-tauri/target/release/ai-helper.exe`。
