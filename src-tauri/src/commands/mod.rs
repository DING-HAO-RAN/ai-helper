//! 前后端通信 Tauri Commands 模块
//! 将所有业务能力（提示词、令牌、DPAPI、Agent扫描与索引缓存、设置配置、窗口生命周期）封装为 Tauri RPC 命令供前端调用。

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tauri::menu::{ContextMenu, Menu, MenuItem, PredefinedMenuItem};
use tauri::{AppHandle, Manager, WebviewWindow};

use crate::scanner::injector::{inject_all_files, InjectSummary};
use crate::scanner::{
    load_agent_index, mark_agents_guide_status, scan_and_cache_all_agents,
    AgentFileInfo, AgentIndexCache,
};
use crate::storage::types::{AppConfig, PromptItem, TokenDisplayView};
use crate::timezone::types::{ChatGPTStatus, ProxyGeoInfo, TimezonePreset};
use crate::timezone::{
    detect_proxy_geo, get_chatgpt_full_status, get_timezone_presets,
    inject_running_chatgpt_cdp, launch_chatgpt_with_timezone,
    restore_original_system_timezone, set_system_timezone,
};
use crate::storage::{
    delete_prompt as store_delete_prompt, delete_token as store_delete_token,
    get_config_path, get_agent_index_path, get_storage_path, get_token_secret as store_get_secret,
    list_token_views, load_config, load_data, save_config as store_save_config,
    save_token as store_save_token, set_default_token as store_set_default,
    upsert_prompt as store_upsert_prompt,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubUserInfo {
    pub login: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub public_repos: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoragePathsInfo {
    pub app_dir: String,
    pub config_path: String,
    pub storage_path: String,
    pub agent_index_path: String,
}

// ================= 系统设置与存储路径相关命令 =================

#[tauri::command]
pub fn get_storage_paths_info() -> StoragePathsInfo {
    StoragePathsInfo {
        app_dir: crate::storage::get_app_dir().to_string_lossy().to_string(),
        config_path: get_config_path().to_string_lossy().to_string(),
        storage_path: get_storage_path().to_string_lossy().to_string(),
        agent_index_path: get_agent_index_path().to_string_lossy().to_string(),
    }
}

#[tauri::command]
pub fn get_app_config() -> AppConfig {
    load_config()
}

#[tauri::command]
pub fn save_app_config(config: AppConfig) -> Result<(), String> {
    store_save_config(&config)
}

// ================= 提示词相关命令 =================

#[tauri::command]
pub fn get_prompts() -> Vec<PromptItem> {
    load_data().prompts
}

#[tauri::command]
pub fn save_prompt(
    id: Option<String>,
    title: String,
    content: String,
    tags: Vec<String>,
) -> Result<PromptItem, String> {
    store_upsert_prompt(id, title, content, tags)
}

#[tauri::command]
pub fn delete_prompt(id: String) -> Result<(), String> {
    store_delete_prompt(&id)
}

// ================= GitHub 令牌相关命令 =================

#[tauri::command]
pub fn get_tokens() -> Vec<TokenDisplayView> {
    list_token_views()
}

#[tauri::command]
pub fn save_token(
    alias: String,
    plain_token: String,
    note: String,
    is_default: bool,
) -> Result<TokenDisplayView, String> {
    store_save_token(alias, plain_token, note, is_default)
}

#[tauri::command]
pub fn delete_token(id: String) -> Result<(), String> {
    store_delete_token(&id)
}

#[tauri::command]
pub fn set_default_token(id: String) -> Result<(), String> {
    store_set_default(&id)
}

#[tauri::command]
pub fn get_token_plain_text(alias: Option<String>) -> Result<String, String> {
    store_get_secret(alias.as_deref())
}

#[tauri::command]
pub async fn test_github_token(token: String) -> Result<GitHubUserInfo, String> {
    let client = reqwest::Client::builder()
        .user_agent("AI-Helper-CyberMatrix/1.0")
        .build()
        .map_err(|e| format!("初始化网络客户端失败: {}", e))?;

    let resp = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token.trim()))
        .header("Accept", "application/vnd.github+json")
        .send()
        .await
        .map_err(|e| format!("网络请求失败: {}", e))?;

    if !resp.status().is_success() {
        return Err(format!(
            "GitHub 验证失败: HTTP 状态码 {}",
            resp.status().as_u16()
        ));
    }

    #[derive(Deserialize)]
    struct GhRawUser {
        login: String,
        name: Option<String>,
        avatar_url: Option<String>,
        public_repos: Option<u64>,
    }

    let raw = resp
        .json::<GhRawUser>()
        .await
        .map_err(|e| format!("解析 GitHub 响应失败: {}", e))?;

    Ok(GitHubUserInfo {
        login: raw.login,
        name: raw.name,
        avatar_url: raw.avatar_url,
        public_repos: raw.public_repos,
    })
}

// ================= AGENT.md 与索引缓存相关命令 =================

/// 获取本地持久化索引缓存（秒级响应，无需每次重新扫描）
#[tauri::command]
pub fn get_cached_agent_index() -> AgentIndexCache {
    load_agent_index()
}

/// 重新全盘扫描并更新本地 agent_index.json 索引文件
#[tauri::command]
pub async fn scan_and_refresh_agents() -> AgentIndexCache {
    tauri::async_runtime::spawn_blocking(scan_and_cache_all_agents)
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub async fn scan_agents() -> Vec<AgentFileInfo> {
    let cache = scan_and_refresh_agents().await;
    cache.agents
}

#[tauri::command]
pub fn inject_agents(paths: Vec<String>) -> InjectSummary {
    let summary = inject_all_files(&paths);
    let success_paths: Vec<String> = summary
        .details
        .iter()
        .filter(|d| d.success)
        .map(|d| d.path.clone())
        .collect();

    if !success_paths.is_empty() {
        mark_agents_guide_status(&success_paths, true);
    }

    summary
}

#[tauri::command]
pub fn open_in_explorer(path: String) -> Result<(), String> {
    let p = Path::new(&path);

    #[cfg(target_os = "windows")]
    {
        if p.is_file() {
            // 在 Windows 资源管理器中直接定位并高亮选中该文件
            let res = std::process::Command::new("explorer.exe")
                .arg(format!("/select,{}", path))
                .spawn();
            if res.is_ok() {
                return Ok(());
            }
        }
    }

    let target = if p.is_file() {
        p.parent().unwrap_or(p)
    } else {
        p
    };

    open::that(target).map_err(|e| format!("打开系统目录失败: {}", e))
}

#[tauri::command]
pub fn read_file_content(path: String) -> Result<String, String> {
    fs::read_to_string(&path).map_err(|e| format!("读取文件内容失败: {}", e))
}

#[tauri::command]
pub fn save_file_content(path: String, content: String) -> Result<(), String> {
    fs::write(&path, content).map_err(|e| format!("保存文件失败: {}", e))
}

// ================= 时区管理与 ChatGPT/Codex 相关命令 =================

#[tauri::command]
pub async fn detect_proxy_timezone() -> Result<ProxyGeoInfo, String> {
    detect_proxy_geo().await
}

#[tauri::command]
pub async fn get_chatgpt_status() -> ChatGPTStatus {
    get_chatgpt_full_status().await
}

#[tauri::command]
pub fn get_available_timezone_presets() -> Vec<TimezonePreset> {
    get_timezone_presets()
}

#[tauri::command]
pub async fn launch_chatgpt_isolated_timezone(timezone_id: String) -> Result<String, String> {
    launch_chatgpt_with_timezone(&timezone_id).await
}

#[tauri::command]
pub async fn inject_chatgpt_timezone_cdp(timezone_id: String) -> Result<usize, String> {
    inject_running_chatgpt_cdp(9222, &timezone_id).await
}

#[tauri::command]
pub fn set_system_timezone_align(windows_tz: String) -> Result<(), String> {
    set_system_timezone(&windows_tz)
}

#[tauri::command]
pub fn restore_system_timezone() -> Result<String, String> {
    restore_original_system_timezone()
}

// ================= 窗口与悬浮球生命周期命令 =================

#[tauri::command]
pub fn minimize_main_window(window: WebviewWindow) -> Result<(), String> {
    window.minimize().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn maximize_or_restore_main_window(window: WebviewWindow) -> Result<bool, String> {
    let is_max = window.is_maximized().map_err(|e| e.to_string())?;
    if is_max {
        window.unmaximize().map_err(|e| e.to_string())?;
        Ok(false)
    } else {
        window.maximize().map_err(|e| e.to_string())?;
        Ok(true)
    }
}

#[tauri::command]
pub fn is_main_maximized(window: WebviewWindow) -> Result<bool, String> {
    window.is_maximized().map_err(|e| e.to_string())
}

/// 窗口原生拖动命令：调用 Windows 原生拖拽，完全摆脱前端权限限制
#[tauri::command]
pub fn drag_window(window: WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

/// 兼容保留悬浮球原生拖动命令别名
#[tauri::command]
pub fn drag_floating_ball(window: WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|e| e.to_string())
}

/// 获取当前窗口在屏幕上的绝对坐标 (x, y)
#[tauri::command]
pub fn get_window_position(window: WebviewWindow) -> Result<(i32, i32), String> {
    let pos = window.outer_position().map_err(|e| e.to_string())?;
    Ok((pos.x, pos.y))
}

/// 设置当前窗口在屏幕上的绝对物理坐标 (x, y)
#[tauri::command]
pub fn set_window_position(window: WebviewWindow, x: i32, y: i32) -> Result<(), String> {
    window
        .set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }))
        .map_err(|e| e.to_string())
}

/// 悬浮球原生右键上下文菜单弹出命令
#[tauri::command]
pub fn show_floating_context_menu(app: AppHandle, window: WebviewWindow) -> Result<(), String> {
    let sync_tz_item = MenuItem::with_id(
        &app,
        "sync_proxy_tz",
        "🌐 注入代理时区至 ChatGPT",
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;

    let restore_tz_item = MenuItem::with_id(
        &app,
        "restore_sys_tz",
        "🔄 恢复系统默认时区",
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;

    let copy_item = MenuItem::with_id(
        &app,
        "copy_default_token",
        "📋 复制默认 GitHub 令牌",
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;

    let restore_item = MenuItem::with_id(
        &app,
        "restore_main",
        "🖥️ 打开主控制台",
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;

    let sep1 = PredefinedMenuItem::separator(&app).map_err(|e| e.to_string())?;
    let sep2 = PredefinedMenuItem::separator(&app).map_err(|e| e.to_string())?;

    let exit_item = MenuItem::with_id(
        &app,
        "exit_app",
        "🚪 彻底退出程序",
        true,
        None::<&str>,
    )
    .map_err(|e| e.to_string())?;

    let menu = Menu::with_items(
        &app,
        &[
            &sync_tz_item,
            &restore_tz_item,
            &sep1,
            &copy_item,
            &restore_item,
            &sep2,
            &exit_item,
        ],
    )
    .map_err(|e| e.to_string())?;

    menu.popup(window.as_ref().window().clone()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn close_to_floating_ball(app: AppHandle) -> Result<(), String> {
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.hide();
    }

    if let Some(ball_win) = app.get_webview_window("floating_ball") {
        // 定位在屏幕右侧居中偏上
        if let Ok(Some(monitor)) = ball_win.primary_monitor() {
            let screen_size = monitor.size();
            let x = (screen_size.width as i32) - 90;
            let y = 160;
            let _ = ball_win.set_position(tauri::Position::Physical(tauri::PhysicalPosition { x, y }));
        }
        let _ = ball_win.show();
        let _ = ball_win.set_focus();
    }

    Ok(())
}

#[tauri::command]
pub fn restore_from_floating_ball(app: AppHandle) -> Result<(), String> {
    if let Some(ball_win) = app.get_webview_window("floating_ball") {
        let _ = ball_win.hide();
    }

    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.show();
        let _ = main_win.unminimize();
        let _ = main_win.set_focus();
    }

    Ok(())
}

#[tauri::command]
pub fn exit_app(app: AppHandle) {
    app.exit(0);
}

#[tauri::command]
pub fn get_app_storage_location() -> String {
    get_storage_path().to_string_lossy().to_string()
}
