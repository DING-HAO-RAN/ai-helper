//! Tauri 桌面应用程序核心配置与窗口生命周期初始化

use tauri::{Manager, WindowEvent};
use crate::commands::*;
use crate::storage::{init_app_workspace, get_token_secret};

pub fn run_desktop_app() {
    // 1. 程序启动时自动在程序所在文件夹创建设置配置文件与数据缓存文件
    init_app_workspace();

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_storage_paths_info,
            get_app_config,
            save_app_config,
            get_prompts,
            save_prompt,
            delete_prompt,
            get_tokens,
            save_token,
            delete_token,
            set_default_token,
            get_token_plain_text,
            test_github_token,
            get_cached_agent_index,
            scan_and_refresh_agents,
            scan_agents,
            inject_agents,
            open_in_explorer,
            read_file_content,
            save_file_content,
            minimize_main_window,
            maximize_or_restore_main_window,
            is_main_maximized,
            drag_floating_ball,
            show_floating_context_menu,
            close_to_floating_ball,
            restore_from_floating_ball,
            exit_app,
            get_app_storage_location,
        ])
        .on_menu_event(|app, event| {
            // 处理悬浮球原生右键菜单触发的事件
            match event.id().as_ref() {
                "copy_default_token" => {
                    if let Ok(token) = get_token_secret(None) {
                        #[cfg(target_os = "windows")]
                        {
                            use std::process::Command;
                            let _ = Command::new("powershell")
                                .args(["-NoProfile", "-Command", &format!("Set-Clipboard -Value '{}'", token)])
                                .spawn();
                        }
                    }
                }
                "restore_main" => {
                    let _ = restore_from_floating_ball(app.clone());
                }
                "exit_app" => {
                    app.exit(0);
                }
                _ => {}
            }
        })
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
