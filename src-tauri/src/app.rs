//! Tauri 桌面应用程序核心配置与窗口生命周期初始化

use tauri::{Manager, WindowEvent};
use crate::commands::*;
use crate::storage::{init_app_workspace, get_token_secret};

pub fn run_desktop_app() {
    // 1. 程序启动时自动在程序所在文件夹创建设置配置文件与数据缓存文件
    init_app_workspace();

    // 2. 启动后台轻量代理自动同步守护任务 (线程常驻，极低开销)
    std::thread::spawn(|| {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(3));
            if crate::storage::load_config().auto_sync_system_proxy {
                crate::antigravity::check_and_auto_sync_system_proxy();
            }
        }
    });

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
            drag_window,
            drag_floating_ball,
            get_window_position,
            set_window_position,
            show_floating_context_menu,
            detect_proxy_timezone,
            get_chatgpt_status,
            get_available_timezone_presets,
            inject_chatgpt_thread_timezone,
            restore_chatgpt_thread_timezone,
            launch_chatgpt_isolated_timezone,
            inject_chatgpt_timezone_cdp,
            set_system_timezone_align,
            restore_system_timezone,
            diagnose_antigravity_status,
            fix_antigravity_proxy_action,
            sync_system_proxy_action,
            clear_antigravity_proxy_action,
            launch_antigravity_action,
            test_antigravity_google_api,
            close_to_floating_ball,
            restore_from_floating_ball,
            exit_app,
            get_app_storage_location,
        ])
        .on_menu_event(|app, event| {
            // 处理悬浮球原生右键菜单触发的事件
            match event.id().as_ref() {
                "sync_proxy_tz" => {
                    tauri::async_runtime::spawn(async move {
                        if let Ok(info) = crate::timezone::detect_proxy_geo().await {
                            let _ = crate::timezone::inject_chatgpt_memory_timezone(&info.timezone_id);
                        }
                    });
                }
                "restore_sys_tz" => {
                    let _ = crate::timezone::restore_original_system_timezone();
                }
                "fix_antigravity_proxy" => {
                    let _ = crate::antigravity::fix_antigravity_proxy(None);
                }
                "copy_default_token" => {
                    if let Ok(token) = get_token_secret(None) {
                        #[cfg(target_os = "windows")]
                        {
                            let _ = crate::create_hidden_command("powershell")
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
