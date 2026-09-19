//! Antigravity 代理诊断与修复数据模型

use serde::{Deserialize, Serialize};

/// Antigravity 诊断状态报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigravityDiagnostic {
    pub installed: bool,
    pub exe_path: Option<String>,
    pub is_running: bool,
    pub pids: Vec<u32>,
    pub detected_local_proxy: Option<String>,
    pub env_http_proxy: Option<String>,
    pub env_https_proxy: Option<String>,
    pub env_all_proxy: Option<String>,
    pub system_proxy_enable: bool,
    pub system_proxy_server: Option<String>,
    pub desktop_shortcut_found: bool,
    pub shortcut_has_proxy_arg: bool,
    pub shortcut_path: Option<String>,
}

/// 修复操作结果摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntigravityFixResult {
    pub success: bool,
    pub proxy_applied: String,
    pub env_fixed: bool,
    pub shortcut_fixed: bool,
    pub system_proxy_fixed: bool,
    pub broadcast_sent: bool,
    pub message: String,
}

/// Google API 连通性测试报告
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoogleApiTestResult {
    pub success: bool,
    pub latency_ms: u64,
    pub status_code: u16,
    pub target_url: String,
    pub error_msg: Option<String>,
}
