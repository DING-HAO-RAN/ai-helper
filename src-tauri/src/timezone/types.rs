//! 时区与代理网络信息数据结构定义

use serde::{Deserialize, Serialize};

/// 代理 IP 与地理时区信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProxyGeoInfo {
    pub ip: String,
    pub country: String,
    pub country_code: String,
    pub city: String,
    /// IANA 格式时区标识（如 "America/Los_Angeles"）
    pub timezone_id: String,
    /// 对应的 Windows 系统时区名称（如 "Pacific Standard Time"）
    pub windows_tz_name: String,
    /// UTC 偏移文本（如 "UTC-07:00"）
    pub utc_offset_str: String,
    pub isp: String,
}

/// ChatGPT / Codex 客户端运行与时区注入状态
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatGPTStatus {
    pub is_running: bool,
    pub pids: Vec<u32>,
    pub cdp_available: bool,
    pub active_timezone: Option<String>,
    pub app_path: Option<String>,
    pub system_timezone: String,
}

/// 预设常用海外时区项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimezonePreset {
    pub id: String,
    pub label: String,
    pub windows_name: String,
    pub utc_offset: String,
    pub region: String,
}
