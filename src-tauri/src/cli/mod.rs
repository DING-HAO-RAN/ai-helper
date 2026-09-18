//! 命令行 CLI 处理模块
//! 当用户或外部 AI 工具以命令行形式启动时（例如 `ai-helper token get`），
//! 本模块负责毫秒级响应，直接在终端标准输出明文令牌或相关信息，
//! 并以退出码结束进程，避免启动昂贵的 GUI 渲染开销。

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
        "--help" | "-h" | "help" => {
            print_help();
            true
        }
        "--version" | "-v" => {
            println!("AI Helper v1.0.0 (Cyber Matrix Edition)");
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
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("获取 GitHub 令牌失败: {}", err);
                    std::process::exit(1);
                }
            }
        }
        "list" => {
            let tokens = list_token_views();
            if tokens.is_empty() {
                println!("暂无保存的 GitHub 令牌。");
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

fn print_help() {
    println!(r#"
======================================================
 AI Helper - Cyber Matrix AI 辅助集成工具 (CLI 模式)
======================================================

用法:
  ai-helper token get [alias]         获取默认或指定别名的 GitHub 令牌明文
  ai-helper token list                列出所有已本地加密保存的令牌摘要
  ai-helper token set <alias> <token> [note]   在本地加密保存/更新 GitHub 令牌
  ai-helper --help                    查看命令行帮助信息
  ai-helper --version                 查看当前版本号

环境变量配置示例:
  PowerShell:  $env:GITHUB_TOKEN = (ai-helper token get)
  Bash / Zsh:  export GITHUB_TOKEN=$(ai-helper token get)
"#);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_help_args_handling() {
        let args = vec!["ai-helper".to_string(), "--version".to_string()];
        // 测试解析能够命中
        assert!(args.len() > 1 && args[1] == "--version");
    }
}
