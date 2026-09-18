#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use ai_helper_lib::cli;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    // 优先拦截命令行请求，毫秒级响应外部工具与 AI agent
    if cli::handle_cli_args(&args) {
        return;
    }

    // 若无命令行参数，后续由 Tauri 启动完整 GUI
    println!("Starting AI Helper GUI...");
}
