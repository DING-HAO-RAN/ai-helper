//! 前后端通信 Tauri Commands 模块
//! 将所有业务能力（提示词、令牌、DPAPI、Agent扫描与注入、窗口生命周期）封装为 Tauri RPC 命令供前端调用。

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, WebviewWindow};
use crate::scanner::injector::{inject_all_files, InjectSummary};
use crate::scanner::{scan_all_agents, AgentFileInfo};
use crate::storage::types::{PromptItem, TokenDisplayView};
use crate::storage::{
    delete_prompt as store_delete_prompt, delete_token as store_delete_token,
    get_storage_path, get_token_secret as store_get_secret, list_token_views, load_data,
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

// ================= AGENT.md 相关命令 =================

#[tauri::command]
pub async fn scan_agents() -> Vec<AgentFileInfo> {
    // 使用 Tauri 异步任务池在后台执行全盘扫描，避免阻塞 GUI 主线程
    tauri::async_runtime::spawn_blocking(scan_all_agents)
        .await
        .unwrap_or_default()
}

#[tauri::command]
pub fn inject_agents(paths: Vec<String>) -> InjectSummary {
    inject_all_files(&paths)
}

#[tauri::command]
pub fn open_in_explorer(path: String) -> Result<(), String> {
    let p = Path::new(&path);
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

// ================= 窗口与生命周期相关命令 =================

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

#[tauri::command]
pub fn close_to_floating_ball(app: AppHandle) -> Result<(), String> {
    if let Some(main_win) = app.get_webview_window("main") {
        let _ = main_win.hide();
    }

    if let Some(ball_win) = app.get_webview_window("floating_ball") {
        // 定位在屏幕右侧居中偏上
        if let Ok(Some(monitor)) = ball_win.primary_monitor() {
            let screen_size = monitor.size();
            let x = (screen_size.width as i32) - 80;
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
