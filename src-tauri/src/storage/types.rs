//! 数据模型定义：提示词与 GitHub 凭据

use serde::{Deserialize, Serialize};

/// 提示词数据项
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PromptItem {
    pub id: String,
    pub title: String,
    pub content: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// GitHub Token 凭据项
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GitHubTokenItem {
    pub id: String,
    pub alias: String,
    /// 经过 Windows DPAPI 加密并以 Base64 存储的密文字符串
    pub encrypted_token: String,
    pub note: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 前端展示使用的 Token 视图（对密文脱敏）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenDisplayView {
    pub id: String,
    pub alias: String,
    pub masked_token: String,
    pub note: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// 应用程序全局本地持久化数据载体
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AppData {
    #[serde(default)]
    pub prompts: Vec<PromptItem>,
    #[serde(default)]
    pub tokens: Vec<GitHubTokenItem>,
}

/// 系统设置配置数据项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub auto_minimize_to_orb: bool,
    pub default_token_alias: String,
    pub scan_exclude_dirs: Vec<String>,
    #[serde(default)]
    pub auto_sync_system_proxy: bool,
    pub created_at: String,
    pub updated_at: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        let now = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
        Self {
            auto_minimize_to_orb: true,
            default_token_alias: "default".to_string(),
            scan_exclude_dirs: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "Windows".to_string(),
                "target".to_string(),
                "dist".to_string(),
                "AppData".to_string(),
            ],
            auto_sync_system_proxy: false,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}
