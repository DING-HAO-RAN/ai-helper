//! 命令行 CLI 处理模块
//! 当用户或外部 AI 工具以命令行形式启动时（例如 `ai-helper token get`），
//! 本模块负责毫秒级响应，直接在终端标准输出明文令牌或相关信息，
//! 并以退出码结束进程，避免启动昂贵的 GUI 渲染开销。

use std::io::Write;
use crate::storage::{get_token_secret, list_token_views, save_token};

pub fn handle_cli_args(args: &[String]) -> bool {
    // 若只有程序自身名称，则说明是普通 GUI 启动
    if args.len() <= 1 {
        return false;
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
        "--help" | "-h" | "help" => {
            print_help();
            true
        }
        "--version" | "-v" => {
            println!("AI Helper v1.0.0 (Cyber Matrix Edition)");
            let _ = std::io::stdout().flush();
            true
        }
        _ => false,
    }
}

fn handle_token_subcommand(subargs: &[String]) {
    if subargs.is_empty() {
        eprintln!("错误: 缺少 token 子命令。可用命令: get, list, set");
        eprintln!("用法示例: ai-helper token get [alias]");
        std::process::exit(1);
    }

    match subargs[0].as_str() {
        "get" => {
            let alias = subargs.get(1).map(|s| s.as_str());
            match get_token_secret(alias) {
                Ok(token) => {
                    // 直接纯文本输出到 stdout，便于命令行管道捕获
                    print!("{}", token);
                    let _ = std::io::stdout().flush();
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("获取 GitHub 令牌失败: {}", err);
                    let _ = std::io::stderr().flush();
                    std::process::exit(1);
                }
            }
        }
        "list" => {
            let tokens = list_token_views();
            if tokens.is_empty() {
                println!("暂无保存的 GitHub 令牌。");
                let _ = std::io::stdout().flush();
                std::process::exit(0);
            }

            println!("{:<16} {:<18} {:<10} {}", "别名(Alias)", "状态/掩码", "默认", "备注");
            println!("{:-<60}", "");
            for t in tokens {
                let default_mark = if t.is_default { "★ [默认]" } else { "" };
                println!(
                    "{:<16} {:<18} {:<10} {}",
                    t.alias, t.masked_token, default_mark, t.note
                );
            }
            let _ = std::io::stdout().flush();
            std::process::exit(0);
        }
        "set" => {
            if subargs.len() < 3 {
                eprintln!("用法: ai-helper token set <alias> <token_value> [note]");
                std::process::exit(1);
            }
            let alias = subargs[1].clone();
            let token_val = subargs[2].clone();
            let note = subargs.get(3).cloned().unwrap_or_default();
            match save_token(alias.clone(), token_val, note, false) {
                Ok(view) => {
                    println!("成功保存令牌 '{}' [{}]", view.alias, view.masked_token);
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("保存令牌失败: {}", err);
                    std::process::exit(1);
                }
            }
        }
        other => {
            eprintln!("未知的 token 子命令: {}", other);
            std::process::exit(1);
        }
    }
}

fn handle_timezone_subcommand(subargs: &[String]) {
    if subargs.is_empty() {
        eprintln!("错误: 缺少 tz 子命令。可用命令: detect, status, apply, restore");
        eprintln!("用法示例: ai-helper tz detect");
        std::process::exit(1);
    }

    match subargs[0].as_str() {
        "detect" => {
            println!("正在通过网络侦测代理 IP 出口与归属时区...");
            match tauri::async_runtime::block_on(crate::timezone::detect_proxy_geo()) {
                Ok(info) => {
                    println!("--------------------------------------------------");
                    println!("代理出口 IP : {}", info.ip);
                    println!("国家与城市  : {}, {}", info.country, info.city);
                    println!("IANA 时区   : {}", info.timezone_id);
                    println!("Windows 时区: {}", info.windows_tz_name);
                    println!("--------------------------------------------------");
                    let _ = std::io::stdout().flush();
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("探测失败: {}", err);
                    let _ = std::io::stderr().flush();
                    std::process::exit(1);
                }
            }
        }
        "status" => {
            let status = tauri::async_runtime::block_on(crate::timezone::get_chatgpt_full_status());
            println!("--------------------------------------------------");
            println!("ChatGPT 运行状态 : {}", if status.is_running { "运行中" } else { "未启动" });
            if status.is_running {
                println!("进程 PID 列表    : {:?}", status.pids);
            }
            println!("CDP 调试端口     : {}", if status.cdp_available { "已开启 (9222)" } else { "未开启" });
            println!("当前注入时区     : {}", status.active_timezone.as_deref().unwrap_or("未单独注入 (使用默认)"));
            println!("Windows 系统时区 : {}", status.system_timezone);
            println!("--------------------------------------------------");
            let _ = std::io::stdout().flush();
            std::process::exit(0);
        }
        "apply" => {
            let tz = if let Some(target) = subargs.get(1) {
                target.clone()
            } else {
                println!("正在自动侦测代理 IP 所在地时区...");
                match tauri::async_runtime::block_on(crate::timezone::detect_proxy_geo()) {
                    Ok(info) => info.timezone_id,
                    Err(err) => {
                        eprintln!("自动侦测代理失败: {}", err);
                        std::process::exit(1);
                    }
                }
            };

            println!("正在以独立时区模式启动并注入 ChatGPT [目标时区: {}]...", tz);
            match tauri::async_runtime::block_on(crate::timezone::launch_chatgpt_with_timezone(&tz)) {
                Ok(msg) => {
                    println!("✓ {}", msg);
                    let _ = std::io::stdout().flush();
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("× 注入失败: {}", err);
                    let _ = std::io::stderr().flush();
                    std::process::exit(1);
                }
            }
        }
        "restore" => {
            match crate::timezone::restore_original_system_timezone() {
                Ok(tz) => {
                    println!("✓ 已成功恢复 Windows 系统时区为: {}", tz);
                    let _ = std::io::stdout().flush();
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("恢复失败: {}", err);
                    let _ = std::io::stderr().flush();
                    std::process::exit(1);
                }
            }
        }
        other => {
            eprintln!("未知的 tz 子命令: {}", other);
            std::process::exit(1);
        }
    }
}

fn print_help() {
    println!(r#"
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
  ai-helper --help                    查看命令行帮助信息
  ai-helper --version                 查看当前版本号

时区与令牌快速环境变量注入:
  PowerShell:  $env:GITHUB_TOKEN = (ai-helper token get)
  Bash / Zsh:  export GITHUB_TOKEN=$(ai-helper token get)
"#);
    let _ = std::io::stdout().flush();
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_help_args_handling() {
        let args = vec!["ai-helper".to_string(), "--version".to_string()];
        // 测试解析能够命中
        assert!(args.len() > 1 && args[1] == "--version");
    }
}
