//! 命令行 CLI 处理模块
//! 当用户或外部 AI 工具以命令行形式启动时（例如 `ai-helper token get`），
//! 本模块负责毫秒级响应，直接向终端输出明文令牌或相关信息，
//! 并以退出码结束进程，避免启动昂贵的 GUI 渲染开销。

use std::io::Write;
use crate::storage::{get_token_secret, list_token_views, save_token};

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::{INVALID_HANDLE_VALUE, HANDLE};
#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Console::{
    AttachConsole, GetStdHandle, ATTACH_PARENT_PROCESS, STD_OUTPUT_HANDLE,
};
#[cfg(target_os = "windows")]
use windows_sys::Win32::Storage::FileSystem::WriteFile;

/// 向当前控制台输出字符串（兼容 Windows GUI 子系统模式）
pub fn console_print(msg: &str) {
    #[cfg(target_os = "windows")]
    unsafe {
        let h_out: HANDLE = GetStdHandle(STD_OUTPUT_HANDLE);
        if !h_out.is_null() && h_out != INVALID_HANDLE_VALUE {
            let bytes = msg.as_bytes();
            let mut written = 0u32;
            if WriteFile(
                h_out,
                bytes.as_ptr(),
                bytes.len() as u32,
                &mut written,
                std::ptr::null_mut(),
            ) != 0
            {
                return;
            }
        }
    }

    print!("{}", msg);
    let _ = std::io::stdout().flush();
}

/// 向当前控制台输出行（兼容 Windows GUI 子系统模式）
pub fn console_println(msg: &str) {
    console_print(&format!("{}\n", msg));
}

pub fn handle_cli_args(args: &[String]) -> bool {
    // 若只有程序自身名称，则说明是普通 GUI 启动，直接返回 false 让 main 继续启动 WebView2
    if args.len() <= 1 {
        return false;
    }

    // Windows GUI 子系统模式下，当检测到命令行调用时，动态附着到调用者的父终端控制台
    #[cfg(target_os = "windows")]
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }

    let command = args[1].as_str();

    match command {
        "token" => {
            handle_token_subcommand(&args[2..]);
            true
        }
        "tz" | "timezone" => {
            handle_timezone_subcommand(&args[2..]);
            true
        }
        "agy" | "antigravity" => {
            handle_antigravity_subcommand(&args[2..]);
            true
        }
        "--help" | "-h" | "help" => {
            print_help();
            true
        }
        "--version" | "-v" => {
            console_println("AI Helper v1.0.0 (Cyber Matrix Edition)");
            true
        }
        _ => false,
    }
}

fn handle_token_subcommand(subargs: &[String]) {
    if subargs.is_empty() {
        console_println("错误: 缺少 token 子命令。可用命令: get, list, set");
        console_println("用法示例: ai-helper token get [alias]");
        std::process::exit(1);
    }

    match subargs[0].as_str() {
        "get" => {
            let alias = subargs.get(1).map(|s| s.as_str());
            match get_token_secret(alias) {
                Ok(token) => {
                    console_print(&token);
                    std::process::exit(0);
                }
                Err(err) => {
                    console_println(&format!("获取 GitHub 令牌失败: {}", err));
                    std::process::exit(1);
                }
            }
        }
        "list" => {
            let tokens = list_token_views();
            if tokens.is_empty() {
                console_println("暂无保存的 GitHub 令牌。");
                std::process::exit(0);
            }

            console_println(&format!("{:<16} {:<18} {:<10} {}", "别名(Alias)", "状态/掩码", "默认", "备注"));
            console_println(&format!("{:-<60}", ""));
            for t in tokens {
                let default_mark = if t.is_default { "★ [默认]" } else { "" };
                console_println(&format!(
                    "{:<16} {:<18} {:<10} {}",
                    t.alias, t.masked_token, default_mark, t.note
                ));
            }
            std::process::exit(0);
        }
        "set" => {
            if subargs.len() < 3 {
                console_println("用法: ai-helper token set <alias> <token_value> [note]");
                std::process::exit(1);
            }
            let alias = subargs[1].clone();
            let token_val = subargs[2].clone();
            let note = subargs.get(3).cloned().unwrap_or_default();
            match save_token(alias.clone(), token_val, note, false) {
                Ok(view) => {
                    console_println(&format!("成功保存令牌 '{}' [{}]", view.alias, view.masked_token));
                    std::process::exit(0);
                }
                Err(err) => {
                    console_println(&format!("保存令牌失败: {}", err));
                    std::process::exit(1);
                }
            }
        }
        other => {
            console_println(&format!("未知的 token 子命令: {}", other));
            std::process::exit(1);
        }
    }
}

fn handle_timezone_subcommand(subargs: &[String]) {
    if subargs.is_empty() {
        console_println("错误: 缺少 tz 子命令。可用命令: detect, status, apply, restore");
        console_println("用法示例: ai-helper tz detect");
        std::process::exit(1);
    }

    match subargs[0].as_str() {
        "detect" => {
            console_println("正在通过网络侦测代理 IP 出口与归属时区...");
            match tauri::async_runtime::block_on(crate::timezone::detect_proxy_geo()) {
                Ok(info) => {
                    console_println("--------------------------------------------------");
                    console_println(&format!("代理出口 IP : {}", info.ip));
                    console_println(&format!("国家与城市  : {}, {}", info.country, info.city));
                    console_println(&format!("IANA 时区   : {}", info.timezone_id));
                    console_println(&format!("Windows 时区: {}", info.windows_tz_name));
                    console_println("--------------------------------------------------");
                    std::process::exit(0);
                }
                Err(err) => {
                    console_println(&format!("探测失败: {}", err));
                    std::process::exit(1);
                }
            }
        }
        "status" => {
            let status = tauri::async_runtime::block_on(crate::timezone::get_chatgpt_full_status());
            console_println("--------------------------------------------------");
            console_println(&format!("ChatGPT 运行状态 : {}", if status.is_running { "运行中" } else { "未启动" }));
            if status.is_running {
                console_println(&format!("进程 PID 列表    : {:?}", status.pids));
            }
            console_println(&format!("CDP 调试端口     : {}", if status.cdp_available { "已开启 (9222)" } else { "未开启" }));
            console_println(&format!("当前注入时区     : {}", status.active_timezone.as_deref().unwrap_or("未单独注入 (使用默认)")));
            console_println(&format!("Windows 系统时区 : {}", status.system_timezone));
            console_println("--------------------------------------------------");
            std::process::exit(0);
        }
        "apply" => {
            let tz = if let Some(target) = subargs.get(1) {
                target.clone()
            } else {
                console_println("正在自动侦测代理 IP 所在地时区...");
                match tauri::async_runtime::block_on(crate::timezone::detect_proxy_geo()) {
                    Ok(info) => info.timezone_id,
                    Err(err) => {
                        console_println(&format!("自动侦测代理失败: {}", err));
                        std::process::exit(1);
                    }
                }
            };

            console_println(&format!("正在向 ChatGPT 进程与线程注入目标时区 [{} (不影响系统时区)]...", tz));
            match crate::timezone::inject_chatgpt_memory_timezone(&tz) {
                Ok(msg) => {
                    console_println(&format!("✓ {}", msg));
                    std::process::exit(0);
                }
                Err(err) => {
                    console_println(&format!("× 注入失败: {}", err));
                    std::process::exit(1);
                }
            }
        }
        "restore" => {
            match crate::timezone::restore_original_system_timezone() {
                Ok(tz) => {
                    console_println(&format!("✓ 已成功恢复 Windows 系统时区为: {}", tz));
                    std::process::exit(0);
                }
                Err(err) => {
                    console_println(&format!("恢复失败: {}", err));
                    std::process::exit(1);
                }
            }
        }
        other => {
            console_println(&format!("未知的 tz 子命令: {}", other));
            std::process::exit(1);
        }
    }
}

fn handle_antigravity_subcommand(subargs: &[String]) {
    if subargs.is_empty() {
        console_println("错误: 缺少 agy 子命令。可用命令: status, fix, clear, launch, test");
        console_println("用法示例: ai-helper agy fix");
        std::process::exit(1);
    }

    match subargs[0].as_str() {
        "status" => {
            let diag = crate::antigravity::diagnose_antigravity();
            console_println("--------------------------------------------------");
            console_println(&format!("Antigravity 安装状态 : {}", if diag.installed { "已安装" } else { "未找到" }));
            if let Some(p) = diag.exe_path {
                console_println(&format!("可执行文件路径       : {}", p));
            }
            console_println(&format!("进程运行状态         : {}", if diag.is_running { "运行中" } else { "未启动" }));
            console_println(&format!("检测到本地代理端口   : {}", diag.detected_local_proxy.as_deref().unwrap_or("未侦测到活跃端口")));
            console_println(&format!("环境变量 HTTP_PROXY  : {}", diag.env_http_proxy.as_deref().unwrap_or("未配置 (直连)")));
            console_println(&format!("环境变量 HTTPS_PROXY : {}", diag.env_https_proxy.as_deref().unwrap_or("未配置 (直连)")));
            console_println(&format!("快捷方式代理参数     : {}", if diag.shortcut_has_proxy_arg { "已配置代理" } else { "未配置" }));
            console_println(&format!("系统代理开关 (注册表): {}", if diag.system_proxy_enable { "开启" } else { "关闭" }));
            console_println("--------------------------------------------------");
            std::process::exit(0);
        }
        "fix" => {
            let proxy_arg = subargs.get(1).cloned();
            match crate::antigravity::fix_antigravity_proxy(proxy_arg) {
                Ok(res) => {
                    console_println(&format!("✓ {}", res.message));
                    std::process::exit(0);
                }
                Err(err) => {
                    console_println(&format!("× 修复失败: {}", err));
                    std::process::exit(1);
                }
            }
        }
        "clear" => {
            match crate::antigravity::clear_antigravity_proxy() {
                Ok(msg) => {
                    console_println(&format!("✓ {}", msg));
                    std::process::exit(0);
                }
                Err(err) => {
                    console_println(&format!("× 清除失败: {}", err));
                    std::process::exit(1);
                }
            }
        }
        "launch" => {
            let proxy_arg = subargs.get(1).cloned();
            match crate::antigravity::launch_antigravity_with_proxy(proxy_arg) {
                Ok(msg) => {
                    console_println(&format!("✓ {}", msg));
                    std::process::exit(0);
                }
                Err(err) => {
                    console_println(&format!("× 拉起失败: {}", err));
                    std::process::exit(1);
                }
            }
        }
        "test" => {
            let proxy_arg = subargs.get(1).cloned();
            console_println("正在测试通过代理连接 Google Gemini API...");
            match tauri::async_runtime::block_on(crate::antigravity::test_google_api(proxy_arg)) {
                Ok(res) => {
                    if res.success {
                        console_println(&format!("✓ 连通成功！耗时: {} ms | HTTP 响应码: {}", res.latency_ms, res.status_code));
                    } else {
                        console_println(&format!("× 连通失败！耗时: {} ms | 响应码: {} | 错误: {:?}", res.latency_ms, res.status_code, res.error_msg));
                    }
                    std::process::exit(0);
                }
                Err(err) => {
                    console_println(&format!("测试出错: {}", err));
                    std::process::exit(1);
                }
            }
        }
        other => {
            console_println(&format!("未知的 agy 子命令: {}", other));
            std::process::exit(1);
        }
    }
}

fn print_help() {
    console_println(r#"
======================================================
 AI Helper - Cyber Matrix AI 辅助集成工具 (CLI 模式)
======================================================

用法:
  ai-helper token get [alias]         获取默认或指定别名的 GitHub 令牌明文
  ai-helper token list                列出所有已本地加密保存的令牌摘要
  ai-helper token set <alias> <token> [note]   在本地加密保存/更新 GitHub 令牌
  ai-helper tz detect                 自动侦测代理 IP 所在地与推荐时区
  ai-helper tz status                 查看 ChatGPT 运行与时区注入状态
  ai-helper tz apply [timezone_id]    单独将时区注入 ChatGPT (不影响系统时区)
  ai-helper tz restore                一键恢复 Windows 系统原始时区
  ai-helper agy status                体检 Antigravity 客户端与代理环境变量
  ai-helper agy fix [proxy_url]       一键修复 Antigravity 代理 (写环境变量与快捷方式)
  ai-helper agy test [proxy_url]      测试通过代理连接 Google CloudCode API
  ai-helper agy launch [proxy_url]    以强制代理模式拉起 Antigravity
  ai-helper agy clear                 清除代理环境变量并还原快捷方式
  ai-helper --help                    查看命令行帮助信息
  ai-helper --version                 查看当前版本号

时区与令牌快速环境变量注入:
  PowerShell:  $env:GITHUB_TOKEN = (ai-helper token get)
  Bash / Zsh:  export GITHUB_TOKEN=$(ai-helper token get)
"#);
}
