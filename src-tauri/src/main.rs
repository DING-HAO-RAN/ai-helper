#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ai_helper_lib::{app, cli};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // 优先拦截命令行请求，毫秒级响应外部工具与 AI agent
    if cli::handle_cli_args(&args) {
        return;
    }

    // 启动赛博朋克桌面主程序与悬浮球管理运行时
    app::run_desktop_app();
}
