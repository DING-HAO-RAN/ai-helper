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
