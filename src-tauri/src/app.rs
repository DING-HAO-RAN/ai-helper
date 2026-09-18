//! Tauri 桌面应用程序核心配置与窗口生命周期初始化

use tauri::{Manager, WindowEvent};
use crate::commands::*;

pub fn run_desktop_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_prompts,
            save_prompt,
            delete_prompt,
            get_tokens,
            save_token,
            delete_token,
            set_default_token,
            get_token_plain_text,
            test_github_token,
            scan_agents,
            inject_agents,
            open_in_explorer,
            read_file_content,
            save_file_content,
            minimize_main_window,
            maximize_or_restore_main_window,
            is_main_maximized,
            close_to_floating_ball,
            restore_from_floating_ball,
            exit_app,
            get_app_storage_location,
        ])
        .on_window_event(|window, event| {
            // 拦截主窗口关闭事件，默认隐入后台并弹出桌面悬浮球
            if let WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "main" {
                    api.prevent_close();
                    let _ = close_to_floating_ball(window.app_handle().clone());
                }
            }
        })
        .run(tauri::generate_context!())
        .expect("启动 AI Helper 桌面程序失败");
}
