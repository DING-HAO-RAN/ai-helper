//! Codex / ChatGPT 时区伪装与代理时区智能同步模块
//! 支持自动识别当前代理出口 IP 所在地的地理时区，
//! 并提供基于 Chromium CDP 协议对 ChatGPT 客户端单独注入指定时区（不影响系统全局时区），
//! 以及 Windows 系统时区一键对齐与防遗忘自动恢复能力。

pub mod cdp;
pub mod types;

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Mutex;
use std::time::Duration;
use serde_json::Value;

use self::cdp::send_cdp_timezone_override;
use self::types::{ChatGPTStatus, ProxyGeoInfo, TimezonePreset};

/// 全局记录已注入到 ChatGPT 的时区标识与原始系统时区
static ACTIVE_OVERRIDE_TZ: Mutex<Option<String>> = Mutex::new(None);
static ORIGINAL_SYS_TZ: Mutex<Option<String>> = Mutex::new(None);

/// IANA 时区标识与 Windows 注册表/tzutil 标准时区名称对应表
pub fn iana_to_windows_tz(iana: &str) -> &'static str {
    match iana {
        // 美国时区
        "America/Los_Angeles" | "PST8PDT" | "US/Pacific" => "Pacific Standard Time",
        "America/Denver" | "MST7MDT" | "US/Mountain" => "Mountain Standard Time",
        "America/Chicago" | "CST6CDT" | "US/Central" => "Central Standard Time",
        "America/New_York" | "EST5EDT" | "US/Eastern" => "Eastern Standard Time",
        "America/Phoenix" => "US Mountain Standard Time",
        "America/Anchorage" => "Alaskan Standard Time",
        "Pacific/Honolulu" => "Hawaiian Standard Time",

        // 亚太时区
        "Asia/Shanghai" | "Asia/Chongqing" | "Asia/Harbin" | "Asia/Hong_Kong" | "Asia/Taipei" => {
            "China Standard Time"
        }
        "Asia/Tokyo" => "Tokyo Standard Time",
        "Asia/Seoul" => "Korea Standard Time",
        "Asia/Singapore" => "Singapore Standard Time",
        "Asia/Kuala_Lumpur" => "Singapore Standard Time",
        "Asia/Bangkok" => "SE Asia Standard Time",
        "Asia/Jakarta" => "SE Asia Standard Time",
        "Asia/Kolkata" => "India Standard Time",
        "Asia/Dubai" => "Arabian Standard Time",

        // 欧洲时区
        "Europe/London" | "GB" | "UTC" | "Etc/UTC" => "GMT Standard Time",
        "Europe/Berlin" | "Europe/Frankfurt" | "Europe/Rome" | "Europe/Stockholm" => {
            "W. Europe Standard Time"
        }
        "Europe/Paris" | "Europe/Madrid" | "Europe/Brussels" => "Romance Standard Time",
        "Europe/Amsterdam" => "W. Europe Standard Time",
        "Europe/Moscow" => "Russian Standard Time",

        // 大洋洲
        "Australia/Sydney" | "Australia/Melbourne" => "AUS Eastern Standard Time",

        _ => {
            // 模糊前缀匹配
            if iana.starts_with("America/") {
                "Pacific Standard Time"
            } else if iana.starts_with("Europe/") {
                "W. Europe Standard Time"
            } else if iana.starts_with("Asia/") {
                "Tokyo Standard Time"
            } else {
                "Pacific Standard Time"
            }
        }
    }
}

/// 获取常用海外节点时区预设
pub fn get_timezone_presets() -> Vec<TimezonePreset> {
    vec![
        TimezonePreset {
            id: "America/Los_Angeles".to_string(),
            label: "美西 / 洛杉矶 (PST/PDT)".to_string(),
            windows_name: "Pacific Standard Time".to_string(),
            utc_offset: "UTC-07:00".to_string(),
            region: "美国".to_string(),
        },
        TimezonePreset {
            id: "America/New_York".to_string(),
            label: "美东 / 纽约 (EST/EDT)".to_string(),
            windows_name: "Eastern Standard Time".to_string(),
            utc_offset: "UTC-04:00".to_string(),
            region: "美国".to_string(),
        },
        TimezonePreset {
            id: "America/Chicago".to_string(),
            label: "美中 / 芝加哥 (CST/CDT)".to_string(),
            windows_name: "Central Standard Time".to_string(),
            utc_offset: "UTC-05:00".to_string(),
            region: "美国".to_string(),
        },
        TimezonePreset {
            id: "Asia/Tokyo".to_string(),
            label: "日本 / 东京 (JST)".to_string(),
            windows_name: "Tokyo Standard Time".to_string(),
            utc_offset: "UTC+09:00".to_string(),
            region: "亚太".to_string(),
        },
        TimezonePreset {
            id: "Asia/Singapore".to_string(),
            label: "新加坡 (SGT)".to_string(),
            windows_name: "Singapore Standard Time".to_string(),
            utc_offset: "UTC+08:00".to_string(),
            region: "亚太".to_string(),
        },
        TimezonePreset {
            id: "Europe/London".to_string(),
            label: "英国 / 伦敦 (GMT/BST)".to_string(),
            windows_name: "GMT Standard Time".to_string(),
            utc_offset: "UTC+01:00".to_string(),
            region: "欧洲".to_string(),
        },
        TimezonePreset {
            id: "Europe/Berlin".to_string(),
            label: "德国 / 法兰克福 (CET/CEST)".to_string(),
            windows_name: "W. Europe Standard Time".to_string(),
            utc_offset: "UTC+02:00".to_string(),
            region: "欧洲".to_string(),
        },
    ]
}

/// 自动向网络侦测接口请求，获取当前代理出口 IP、地理位置与所属时区
pub async fn detect_proxy_geo() -> Result<ProxyGeoInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(5))
        .build()
        .map_err(|e| format!("初始化网络探测失败: {}", e))?;

    // 优先尝试 ip-api.com
    if let Ok(resp) = client.get("http://ip-api.com/json").send().await {
        if resp.status().is_success() {
            if let Ok(val) = resp.json::<Value>().await {
                if val["status"].as_str() == Some("success") {
                    let ip = val["query"].as_str().unwrap_or("未知").to_string();
                    let country = val["country"].as_str().unwrap_or("未知").to_string();
                    let country_code = val["countryCode"].as_str().unwrap_or("UN").to_string();
                    let city = val["city"].as_str().unwrap_or("未知").to_string();
                    let timezone_id = val["timezone"].as_str().unwrap_or("America/Los_Angeles").to_string();
                    let isp = val["isp"].as_str().unwrap_or("").to_string();

                    let win_tz = iana_to_windows_tz(&timezone_id).to_string();

                    return Ok(ProxyGeoInfo {
                        ip,
                        country,
                        country_code,
                        city,
                        timezone_id,
                        windows_tz_name: win_tz,
                        utc_offset_str: "自动计算".to_string(),
                        isp,
                    });
                }
            }
        }
    }

    // 备用接口: ipapi.co
    if let Ok(resp) = client.get("https://ipapi.co/json/").send().await {
        if resp.status().is_success() {
            if let Ok(val) = resp.json::<Value>().await {
                let ip = val["ip"].as_str().unwrap_or("未知").to_string();
                let country = val["country_name"].as_str().unwrap_or("未知").to_string();
                let country_code = val["country_code"].as_str().unwrap_or("UN").to_string();
                let city = val["city"].as_str().unwrap_or("未知").to_string();
                let timezone_id = val["timezone"].as_str().unwrap_or("America/Los_Angeles").to_string();
                let isp = val["org"].as_str().unwrap_or("").to_string();

                let win_tz = iana_to_windows_tz(&timezone_id).to_string();

                return Ok(ProxyGeoInfo {
                    ip,
                    country,
                    country_code,
                    city,
                    timezone_id,
                    windows_tz_name: win_tz,
                    utc_offset_str: "自动计算".to_string(),
                    isp,
                });
            }
        }
    }

    Err("无法通过当前网络获取代理出口 IP 归属地，请检查代理工具或网络连接".to_string())
}

/// 获取 Windows 当前系统全局时区名称（如 "China Standard Time"）
pub fn get_system_current_timezone() -> String {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("tzutil").arg("/g").output();
        if let Ok(out) = output {
            let res = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !res.is_empty() {
                // 缓存原始时区
                let mut orig = ORIGINAL_SYS_TZ.lock().unwrap();
                if orig.is_none() {
                    *orig = Some(res.clone());
                }
                return res;
            }
        }
    }

    "China Standard Time".to_string()
}

/// 修改 Windows 当前系统全局时区
pub fn set_system_timezone(windows_tz_name: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        // 先记录原始系统时区
        let current = get_system_current_timezone();
        let mut orig = ORIGINAL_SYS_TZ.lock().unwrap();
        if orig.is_none() {
            *orig = Some(current);
        }

        let status = Command::new("tzutil")
            .args(["/s", windows_tz_name])
            .status()
            .map_err(|e| format!("执行 tzutil 失败: {}", e))?;

        if status.success() {
            return Ok(());
        } else {
            return Err("切换系统时区失败，可能需要管理员权限".to_string());
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        Ok(())
    }
}

/// 一键恢复 Windows 原始系统时区
pub fn restore_original_system_timezone() -> Result<String, String> {
    let orig = {
        let guard = ORIGINAL_SYS_TZ.lock().unwrap();
        guard.clone().unwrap_or_else(|| "China Standard Time".to_string())
    };

    set_system_timezone(&orig)?;
    Ok(orig)
}

/// 自动探测电脑中 ChatGPT.exe 的实际物理安装路径
pub fn detect_chatgpt_executable_path() -> Option<String> {
    #[cfg(target_os = "windows")]
    {
        // 1. 通过 PowerShell 检索 AppxPackage OpenAI.Codex
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "(Get-AppxPackage *openai*).InstallLocation",
            ])
            .output();

        if let Ok(out) = output {
            let loc = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !loc.is_empty() {
                let candidate = Path::new(&loc).join("app").join("ChatGPT.exe");
                if candidate.exists() {
                    return Some(candidate.to_string_lossy().to_string());
                }
            }
        }

        // 2. 遍历 C:\Program Files\WindowsApps 目录检索
        let winapps = Path::new("C:\\Program Files\\WindowsApps");
        if winapps.exists() {
            if let Ok(entries) = fs::read_dir(winapps) {
                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if name.starts_with("OpenAI.Codex_") {
                        let candidate = entry.path().join("app").join("ChatGPT.exe");
                        if candidate.exists() {
                            return Some(candidate.to_string_lossy().to_string());
                        }
                    }
                }
            }
        }
    }

    None
}

/// 检索正在运行的 ChatGPT.exe 进程 ID 列表
pub fn get_running_chatgpt_pids() -> Vec<u32> {
    #[cfg(target_os = "windows")]
    {
        let output = Command::new("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-Process -Name 'ChatGPT' -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id",
            ])
            .output();

        if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout);
            return s
                .lines()
                .filter_map(|line| line.trim().parse::<u32>().ok())
                .collect();
        }
    }

    Vec::new()
}

/// 检查 9222 远程调试端口是否就绪
pub async fn check_cdp_port_ready(port: u16) -> bool {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_millis(500))
        .build();

    if let Ok(c) = client {
        let uri = format!("http://127.0.0.1:{}/json/version", port);
        if let Ok(resp) = c.get(&uri).send().await {
            return resp.status().is_success();
        }
    }
    false
}

/// 针对已开启 9222 调试端口的 ChatGPT 实例注入时区覆盖
pub async fn inject_running_chatgpt_cdp(
    port: u16,
    timezone_id: &str,
) -> Result<usize, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .map_err(|e| e.to_string())?;

    let list_url = format!("http://127.0.0.1:{}/json", port);
    let resp = client
        .get(&list_url)
        .send()
        .await
        .map_err(|e| format!("请求 CDP 页面列表失败: {}", e))?;

    let pages = resp
        .json::<Vec<Value>>()
        .await
        .map_err(|e| format!("解析 CDP 目标列表失败: {}", e))?;

    let mut injected = 0;
    for p in pages {
        if let Some(ws_url) = p["webSocketDebuggerUrl"].as_str() {
            if send_cdp_timezone_override(ws_url, timezone_id, Duration::from_millis(600)).is_ok() {
                injected += 1;
            }
        }
    }

    if injected > 0 {
        let mut active = ACTIVE_OVERRIDE_TZ.lock().unwrap();
        *active = Some(timezone_id.to_string());
        Ok(injected)
    } else {
        Err("未能连接到任何有效的 ChatGPT 网页或渲染窗口进行注入".to_string())
    }
}

/// 以专用时区覆盖模式（CDP 9222 调试端口）启动 ChatGPT 并自动注入代理时区
pub async fn launch_chatgpt_with_timezone(timezone_id: &str) -> Result<String, String> {
    let chatgpt_path = detect_chatgpt_executable_path()
        .ok_or_else(|| "未能在系统中定位到 ChatGPT.exe (OpenAI.Codex)".to_string())?;

    // 1. 若当前有旧的普通 ChatGPT 进程在运行，先关闭以释放单例互斥锁
    let pids = get_running_chatgpt_pids();
    if !pids.is_empty() {
        #[cfg(target_os = "windows")]
        {
            let _ = Command::new("powershell")
                .args([
                    "-NoProfile",
                    "-Command",
                    "Stop-Process -Name 'ChatGPT' -Force -ErrorAction SilentlyContinue",
                ])
                .output();
            std::thread::sleep(Duration::from_millis(800));
        }
    }

    // 2. 带 --remote-debugging-port=9222 参数启动 ChatGPT
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new(&chatgpt_path)
            .args(["--remote-debugging-port=9222"])
            .spawn()
            .map_err(|e| format!("启动 ChatGPT 失败: {}", e))?;
    }

    // 3. 等待 9222 调试端口就绪并执行注入
    let mut ready = false;
    for _ in 0..12 {
        std::thread::sleep(Duration::from_millis(500));
        if check_cdp_port_ready(9222).await {
            ready = true;
            break;
        }
    }

    if !ready {
        return Err("ChatGPT 已启动，但未能成功开启 CDP 调试通道".to_string());
    }

    // 4. 执行注入
    let count = inject_running_chatgpt_cdp(9222, timezone_id).await?;

    Ok(format!(
        "ChatGPT 启动成功！已成功将时区注入至 {} 个渲染页面 [时区: {}]",
        count, timezone_id
    ))
}

/// 获取完整的当前运行与时区状态
pub async fn get_chatgpt_full_status() -> ChatGPTStatus {
    let pids = get_running_chatgpt_pids();
    let is_running = !pids.is_empty();
    let cdp_ready = check_cdp_port_ready(9222).await;
    let app_path = detect_chatgpt_executable_path();
    let system_tz = get_system_current_timezone();
    let active_tz = {
        let guard = ACTIVE_OVERRIDE_TZ.lock().unwrap();
        guard.clone()
    };

    ChatGPTStatus {
        is_running,
        pids,
        cdp_available: cdp_ready,
        active_timezone: active_tz,
        app_path,
        system_timezone: system_tz,
    }
}
