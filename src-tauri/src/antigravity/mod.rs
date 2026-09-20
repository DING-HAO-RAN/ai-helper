//! Google Antigravity 代理诊断与一键全自动修复核心模块
//! 专用于解决 Antigravity (Electron + Go + Node) 不走系统代理、网络请求超时与断联的问题。

pub mod types;

use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

#[cfg(target_os = "windows")]
use windows_sys::Win32::System::Registry::{
    RegCloseKey, RegDeleteValueW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY,
    HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_DWORD, REG_SZ,
};

use self::types::{AntigravityDiagnostic, AntigravityFixResult, GoogleApiTestResult};

const PROXY_ENVIRONMENT_NAMES: [&str; 6] = [
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "http_proxy",
    "https_proxy",
    "all_proxy",
];
const NO_PROXY_ENVIRONMENT_NAMES: [&str; 2] = ["NO_PROXY", "no_proxy"];
const NO_PROXY_VALUE: &str = "localhost,127.0.0.1,::1,local.home";
const NODE_USE_ENV_PROXY: &str = "NODE_USE_ENV_PROXY";

/// 生成 Antigravity 及其派生 Node 插件需要继承的完整代理环境。
fn plugin_proxy_environment(proxy_target: &str) -> Vec<(&'static str, String)> {
    let mut variables = PROXY_ENVIRONMENT_NAMES
        .iter()
        .map(|name| (*name, proxy_target.to_string()))
        .collect::<Vec<_>>();
    variables.extend(
        NO_PROXY_ENVIRONMENT_NAMES
            .iter()
            .map(|name| (*name, NO_PROXY_VALUE.to_string())),
    );
    variables.push((NODE_USE_ENV_PROXY, "1".to_string()));
    variables
}

fn is_google_api_success(status: reqwest::StatusCode) -> bool {
    status.is_success()
}

/// 检测 Antigravity.exe 的实际安装路径
pub fn detect_antigravity_path() -> Option<String> {
    // 1. 默认用户安装目录
    if let Some(local_app_data) = dirs::data_local_dir() {
        let p = local_app_data
            .join("Programs")
            .join("antigravity")
            .join("Antigravity.exe");
        if p.exists() {
            return Some(p.to_string_lossy().to_string());
        }
    }

    // 2. 检查桌面快捷方式的目标
    if let Some(target) = get_shortcut_target_path(&get_desktop_shortcut_path()) {
        if Path::new(&target).exists() {
            return Some(target);
        }
    }

    None
}

/// 获取桌面 Antigravity 快捷方式路径
pub fn get_desktop_shortcut_path() -> PathBuf {
    // 优先检查 D:\桌面\Antigravity.lnk
    let d_desktop = PathBuf::from("D:\\桌面\\Antigravity.lnk");
    if d_desktop.exists() {
        return d_desktop;
    }

    // 检查标准用户桌面
    if let Some(user_profile) = dirs::home_dir() {
        let user_desktop = user_profile.join("Desktop").join("Antigravity.lnk");
        if user_desktop.exists() {
            return user_desktop;
        }
        let cn_desktop = user_profile.join("桌面").join("Antigravity.lnk");
        if cn_desktop.exists() {
            return cn_desktop;
        }
    }

    d_desktop
}

/// 读取 Windows 注册表 HKCU 字符串值
#[cfg(target_os = "windows")]
fn get_reg_hkcu_string(subkey: &str, value_name: &str) -> Option<String> {
    unsafe {
        let subkey_w: Vec<u16> = format!("{}\0", subkey).encode_utf16().collect();
        let value_w: Vec<u16> = format!("{}\0", value_name).encode_utf16().collect();
        let mut hkey: HKEY = std::ptr::null_mut();

        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey_w.as_ptr(), 0, KEY_READ, &mut hkey) != 0
            || hkey.is_null()
        {
            return None;
        }

        let mut buf = [0u16; 512];
        let mut len = (buf.len() * 2) as u32;
        let mut dtype = 0u32;
        let status = RegQueryValueExW(
            hkey,
            value_w.as_ptr(),
            std::ptr::null_mut(),
            &mut dtype,
            buf.as_mut_ptr() as *mut u8,
            &mut len,
        );

        RegCloseKey(hkey);

        if status == 0 && (dtype == REG_SZ || dtype == 2 /* REG_EXPAND_SZ */) {
            let str_len = (len / 2).saturating_sub(1) as usize;
            return Some(String::from_utf16_lossy(&buf[..str_len]));
        }
    }
    None
}

/// 读取 Windows 注册表 HKCU DWORD 值
#[cfg(target_os = "windows")]
fn get_reg_hkcu_dword(subkey: &str, value_name: &str) -> Option<u32> {
    unsafe {
        let subkey_w: Vec<u16> = format!("{}\0", subkey).encode_utf16().collect();
        let value_w: Vec<u16> = format!("{}\0", value_name).encode_utf16().collect();
        let mut hkey: HKEY = std::ptr::null_mut();

        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey_w.as_ptr(), 0, KEY_READ, &mut hkey) != 0
            || hkey.is_null()
        {
            return None;
        }

        let mut val = 0u32;
        let mut len = 4u32;
        let mut dtype = 0u32;
        let status = RegQueryValueExW(
            hkey,
            value_w.as_ptr(),
            std::ptr::null_mut(),
            &mut dtype,
            &mut val as *mut u32 as *mut u8,
            &mut len,
        );

        RegCloseKey(hkey);

        if status == 0 && dtype == REG_DWORD {
            return Some(val);
        }
    }
    None
}

/// 写入 Windows 注册表 HKCU 字符串值
#[cfg(target_os = "windows")]
fn set_reg_hkcu_string(subkey: &str, value_name: &str, val: &str) -> Result<(), String> {
    unsafe {
        let subkey_w: Vec<u16> = format!("{}\0", subkey).encode_utf16().collect();
        let value_w: Vec<u16> = format!("{}\0", value_name).encode_utf16().collect();
        let val_w: Vec<u16> = format!("{}\0", val).encode_utf16().collect();
        let mut hkey: HKEY = std::ptr::null_mut();

        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey_w.as_ptr(), 0, KEY_WRITE, &mut hkey) != 0
            || hkey.is_null()
        {
            return Err("打开注册表项失败".to_string());
        }

        let status = RegSetValueExW(
            hkey,
            value_w.as_ptr(),
            0,
            REG_SZ,
            val_w.as_ptr() as *const u8,
            (val_w.len() * 2) as u32,
        );

        RegCloseKey(hkey);

        if status == 0 {
            Ok(())
        } else {
            Err(format!("写入注册表失败, code: {}", status))
        }
    }
}

/// 删除 Windows 注册表 HKCU 字符串值
#[cfg(target_os = "windows")]
fn delete_reg_hkcu_value(subkey: &str, value_name: &str) -> Result<(), String> {
    unsafe {
        let subkey_w: Vec<u16> = format!("{}\0", subkey).encode_utf16().collect();
        let value_w: Vec<u16> = format!("{}\0", value_name).encode_utf16().collect();
        let mut hkey: HKEY = std::ptr::null_mut();

        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey_w.as_ptr(), 0, KEY_WRITE, &mut hkey) != 0
            || hkey.is_null()
        {
            return Ok(());
        }

        let _ = RegDeleteValueW(hkey, value_w.as_ptr());
        RegCloseKey(hkey);
        Ok(())
    }
}

/// 写入 Windows 注册表 HKCU DWORD 值
#[cfg(target_os = "windows")]
fn set_reg_hkcu_dword(subkey: &str, value_name: &str, val: u32) -> Result<(), String> {
    unsafe {
        let subkey_w: Vec<u16> = format!("{}\0", subkey).encode_utf16().collect();
        let value_w: Vec<u16> = format!("{}\0", value_name).encode_utf16().collect();
        let mut hkey: HKEY = std::ptr::null_mut();

        if RegOpenKeyExW(HKEY_CURRENT_USER, subkey_w.as_ptr(), 0, KEY_WRITE, &mut hkey) != 0
            || hkey.is_null()
        {
            return Err("打开注册表项失败".to_string());
        }

        let status = RegSetValueExW(
            hkey,
            value_w.as_ptr(),
            0,
            REG_DWORD,
            &val as *const u32 as *const u8,
            4,
        );

        RegCloseKey(hkey);

        if status == 0 {
            Ok(())
        } else {
            Err(format!("写入注册表失败, code: {}", status))
        }
    }
}

/// 解析与规范化 Windows 注册表中的 ProxyServer 格式 (例如 "127.0.0.1:7897" 或 "http=127.0.0.1:7897;https=127.0.0.1:7897")
pub fn normalize_proxy_server_string(raw: &str) -> Option<String> {
    let clean = raw.trim();
    if clean.is_empty() {
        return None;
    }

    if clean.contains(';') {
        for part in clean.split(';') {
            let part = part.trim();
            if part.starts_with("http=") || part.starts_with("https=") {
                let addr = part.split('=').nth(1).unwrap_or("").trim();
                if !addr.is_empty() {
                    return Some(if addr.starts_with("http://") || addr.starts_with("https://") || addr.starts_with("socks5://") {
                        addr.to_string()
                    } else {
                        format!("http://{}", addr)
                    });
                }
            }
        }
    }

    if clean.starts_with("http://") || clean.starts_with("https://") || clean.starts_with("socks5://") {
        Some(clean.to_string())
    } else {
        Some(format!("http://{}", clean))
    }
}

/// 获取 Windows 系统代理开关与地址
pub fn get_windows_system_proxy_info() -> (bool, Option<String>) {
    #[cfg(target_os = "windows")]
    {
        let enabled = get_reg_hkcu_dword(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            "ProxyEnable",
        )
        .map(|v| v == 1)
        .unwrap_or(false);

        let raw_proxy = get_reg_hkcu_string(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            "ProxyServer",
        );

        let normalized = raw_proxy.as_deref().and_then(normalize_proxy_server_string);
        (enabled, normalized)
    }

    #[cfg(not(target_os = "windows"))]
    {
        (false, None)
    }
}

/// 自动跟随 Windows 系统代理的单次核对维护
pub fn check_and_auto_sync_system_proxy() {
    let (enabled, proxy_opt) = get_windows_system_proxy_info();
    let current_env_proxy = std::env::var("HTTP_PROXY").ok().or_else(|| {
        #[cfg(target_os = "windows")]
        {
            get_reg_hkcu_string("Environment", "HTTP_PROXY")
        }
        #[cfg(not(target_os = "windows"))]
        None
    });

    if enabled {
        if let Some(target) = proxy_opt {
            // 如果环境变量中的代理与系统代理不同，自动同步
            if current_env_proxy.as_deref() != Some(&target) {
                let _ = fix_antigravity_proxy(Some(target));
            }
        }
    }
}

/// 一键将 Windows 当前系统代理设置无缝同步为全局环境代理
pub fn sync_system_proxy_to_env() -> Result<AntigravityFixResult, String> {
    let (enabled, proxy_opt) = get_windows_system_proxy_info();
    if !enabled {
        return Err("检测到当前 Windows 系统代理处于关闭状态 (ProxyEnable=0)。请先在代理软件 (如 Clash/Verge/v2rayN) 中开启「系统代理」开关，然后再点击同步！".to_string());
    }

    let proxy_url = proxy_opt.ok_or_else(|| "Windows 系统代理虽然开启，但未检测到有效的 ProxyServer 代理地址".to_string())?;

    fix_antigravity_proxy(Some(proxy_url))
}

/// 自动侦测本机当前正在活跃监听的本地代理端口 (例如 127.0.0.1:7897, 7890 等)
pub fn detect_active_local_proxy() -> Option<String> {
    // 候选常用代理端口 (Mihomo/Clash/V2Ray/Singbox)
    let candidate_ports = [7897, 7890, 10808, 10809, 2080, 2081, 1080, 8080];

    for port in candidate_ports {
        if TcpStream::connect_timeout(
            &std::net::SocketAddr::from(([127, 0, 0, 1], port)),
            Duration::from_millis(150),
        )
        .is_ok()
        {
            return Some(format!("http://127.0.0.1:{}", port));
        }
    }

    // 从系统注册表 Internet Settings 获取
    let (_, sys_proxy) = get_windows_system_proxy_info();
    if let Some(p) = sys_proxy {
        return Some(p);
    }

    None
}

/// 通过 PowerShell 静默读取快捷方式目标
fn get_shortcut_target_path(shortcut_path: &Path) -> Option<String> {
    if !shortcut_path.exists() {
        return None;
    }

    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "$sh = New-Object -ComObject WScript.Shell; $s = $sh.CreateShortcut('{}'); $s.TargetPath",
            shortcut_path.to_string_lossy().replace('\'', "''")
        );
        let out = crate::create_hidden_command("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
            .ok()?;
        let res = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !res.is_empty() {
            return Some(res);
        }
    }

    None
}

/// 通过 PowerShell 静默读取快捷方式参数
fn get_shortcut_arguments(shortcut_path: &Path) -> String {
    if !shortcut_path.exists() {
        return String::new();
    }

    #[cfg(target_os = "windows")]
    {
        let script = format!(
            "$sh = New-Object -ComObject WScript.Shell; $s = $sh.CreateShortcut('{}'); $s.Arguments",
            shortcut_path.to_string_lossy().replace('\'', "''")
        );
        if let Ok(out) = crate::create_hidden_command("powershell")
            .args(["-NoProfile", "-Command", &script])
            .output()
        {
            return String::from_utf8_lossy(&out.stdout).trim().to_string();
        }
    }

    String::new()
}

/// 获取正在运行的 Antigravity 进程 ID
pub fn get_running_antigravity_pids() -> Vec<u32> {
    #[cfg(target_os = "windows")]
    {
        let out = crate::create_hidden_command("powershell")
            .args([
                "-NoProfile",
                "-Command",
                "Get-Process -Name 'Antigravity' -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Id",
            ])
            .output();

        if let Ok(o) = out {
            let s = String::from_utf8_lossy(&o.stdout);
            return s
                .lines()
                .filter_map(|l| l.trim().parse::<u32>().ok())
                .collect();
        }
    }

    Vec::new()
}

/// 执行全面诊断
pub fn diagnose_antigravity() -> AntigravityDiagnostic {
    let exe_path = detect_antigravity_path();
    let installed = exe_path.is_some();
    let pids = get_running_antigravity_pids();
    let is_running = !pids.is_empty();

    let detected_local_proxy = detect_active_local_proxy();

    let env_http_proxy = std::env::var("HTTP_PROXY").ok().or_else(|| {
        #[cfg(target_os = "windows")]
        {
            get_reg_hkcu_string("Environment", "HTTP_PROXY")
        }
        #[cfg(not(target_os = "windows"))]
        None
    });

    let env_https_proxy = std::env::var("HTTPS_PROXY").ok().or_else(|| {
        #[cfg(target_os = "windows")]
        {
            get_reg_hkcu_string("Environment", "HTTPS_PROXY")
        }
        #[cfg(not(target_os = "windows"))]
        None
    });

    let env_all_proxy = std::env::var("ALL_PROXY").ok().or_else(|| {
        #[cfg(target_os = "windows")]
        {
            get_reg_hkcu_string("Environment", "ALL_PROXY")
        }
        #[cfg(not(target_os = "windows"))]
        None
    });

    #[cfg(target_os = "windows")]
    let system_proxy_enable = get_reg_hkcu_dword(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
        "ProxyEnable",
    )
    .map(|v| v == 1)
    .unwrap_or(false);
    #[cfg(not(target_os = "windows"))]
    let system_proxy_enable = false;

    #[cfg(target_os = "windows")]
    let system_proxy_server = get_reg_hkcu_string(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
        "ProxyServer",
    );
    #[cfg(not(target_os = "windows"))]
    let system_proxy_server = None;

    let shortcut_path = get_desktop_shortcut_path();
    let desktop_shortcut_found = shortcut_path.exists();
    let shortcut_args = get_shortcut_arguments(&shortcut_path);
    let shortcut_has_proxy_arg = shortcut_args.contains("--proxy-server");

    AntigravityDiagnostic {
        installed,
        exe_path,
        is_running,
        pids,
        detected_local_proxy,
        env_http_proxy,
        env_https_proxy,
        env_all_proxy,
        system_proxy_enable,
        system_proxy_server,
        desktop_shortcut_found,
        shortcut_has_proxy_arg,
        shortcut_path: if desktop_shortcut_found {
            Some(shortcut_path.to_string_lossy().to_string())
        } else {
            None
        },
    }
}

/// 广播 Windows WM_SETTINGCHANGE 消息，通知系统所有进程刷新环境变量
#[cfg(target_os = "windows")]
fn broadcast_environment_change() -> bool {
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };

    let env_w: Vec<u16> = "Environment\0".encode_utf16().collect();
    let mut result = 0usize;

    unsafe {
        let res = SendMessageTimeoutW(
            HWND_BROADCAST as HWND,
            WM_SETTINGCHANGE,
            0,
            env_w.as_ptr() as isize,
            SMTO_ABORTIFHUNG,
            1500,
            &mut result,
        );
        res != 0
    }
}

/// 一键全自动修复 Antigravity 代理
pub fn fix_antigravity_proxy(custom_proxy: Option<String>) -> Result<AntigravityFixResult, String> {
    let proxy_target = match custom_proxy {
        Some(p) if !p.trim().is_empty() => p.trim().to_string(),
        _ => detect_active_local_proxy()
            .unwrap_or_else(|| "http://127.0.0.1:7897".to_string()),
    };

    let mut env_fixed = false;
    let mut shortcut_fixed = false;
    let system_proxy_fixed;

    // 1. 写入用户注册表 HKCU\Environment
    #[cfg(target_os = "windows")]
    {
        let plugin_environment = plugin_proxy_environment(&proxy_target);
        let mut all_environment_values_written = true;
        for (name, value) in &plugin_environment {
            if set_reg_hkcu_string("Environment", name, value).is_err() {
                all_environment_values_written = false;
            }
        }

        if all_environment_values_written {
            env_fixed = true;
            // 同步当前进程，后续启动的 Antigravity 与 Node 插件立即继承代理。
            for (name, value) in &plugin_environment {
                std::env::set_var(name, value);
            }
        }

        // 2. 修复 Windows 系统代理开关注册表
        let reg_proxy_addr = proxy_target.trim_start_matches("http://");
        let _ = set_reg_hkcu_string(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            "ProxyServer",
            reg_proxy_addr,
        );
        let _ = set_reg_hkcu_dword(
            "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
            "ProxyEnable",
            1,
        );
        system_proxy_fixed = true;
    }

    // 3. 广播全局设置变更
    #[cfg(target_os = "windows")]
    let broadcast_sent = broadcast_environment_change();
    #[cfg(not(target_os = "windows"))]
    let broadcast_sent = true;

    // 4. 修复桌面快捷方式参数 (追加 --proxy-server=xxx)
    let shortcut_path = get_desktop_shortcut_path();
    if shortcut_path.exists() {
        #[cfg(target_os = "windows")]
        {
            let script = format!(
                "$sh = New-Object -ComObject WScript.Shell; \
                $s = $sh.CreateShortcut('{}'); \
                $s.Arguments = '--proxy-server=\"{}\"'; \
                $s.Save()",
                shortcut_path.to_string_lossy().replace('\'', "''"),
                proxy_target
            );
            let out = crate::create_hidden_command("powershell")
                .args(["-NoProfile", "-Command", &script])
                .status();
            if let Ok(st) = out {
                if st.success() {
                    shortcut_fixed = true;
                }
            }
        }
    }

    Ok(AntigravityFixResult {
        success: true,
        proxy_applied: proxy_target.clone(),
        env_fixed,
        shortcut_fixed,
        system_proxy_fixed,
        broadcast_sent,
        message: format!(
            "修复成功！已将代理 [{}] 写入大小写全局代理变量，并启用 NODE_USE_ENV_PROXY；Antigravity 及其启动的 Node 插件将自动继承代理。快捷方式与系统代理注册表也已同步更新！",
            proxy_target
        ),
    })
}

/// 一键清除与还原 Antigravity 代理环境变量
pub fn clear_antigravity_proxy() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        for name in PROXY_ENVIRONMENT_NAMES
            .iter()
            .chain(NO_PROXY_ENVIRONMENT_NAMES.iter())
            .chain(std::iter::once(&NODE_USE_ENV_PROXY))
        {
            let _ = delete_reg_hkcu_value("Environment", name);
            std::env::remove_var(name);
        }

        broadcast_environment_change();

        // 还原快捷方式
        let shortcut_path = get_desktop_shortcut_path();
        if shortcut_path.exists() {
            let script = format!(
                "$sh = New-Object -ComObject WScript.Shell; \
                $s = $sh.CreateShortcut('{}'); \
                $s.Arguments = ''; \
                $s.Save()",
                shortcut_path.to_string_lossy().replace('\'', "''")
            );
            let _ = crate::create_hidden_command("powershell")
                .args(["-NoProfile", "-Command", &script])
                .status();
        }
    }

    Ok("已成功清除大小写代理环境变量、NODE_USE_ENV_PROXY 与快捷方式附加参数".to_string())
}

/// 以完全代理模式直接拉起 Antigravity
pub fn launch_antigravity_with_proxy(custom_proxy: Option<String>) -> Result<String, String> {
    let exe = detect_antigravity_path()
        .ok_or_else(|| "未能在系统中找到 Antigravity.exe 安装程序".to_string())?;

    let proxy_target = match custom_proxy {
        Some(p) if !p.trim().is_empty() => p.trim().to_string(),
        _ => detect_active_local_proxy()
            .unwrap_or_else(|| "http://127.0.0.1:7897".to_string()),
    };

    #[cfg(target_os = "windows")]
    {
        let mut cmd = crate::create_hidden_command(&exe);
        cmd.arg(format!("--proxy-server={}", proxy_target));
        for (name, value) in plugin_proxy_environment(&proxy_target) {
            cmd.env(name, value);
        }
        cmd.spawn()
            .map_err(|e| format!("拉起 Antigravity 失败: {}", e))?;
    }

    Ok(format!(
        "已成功以强制代理模式启动 Antigravity [代理: {}]",
        proxy_target
    ))
}

/// 测试 Antigravity Google API 连通性
pub async fn test_google_api(custom_proxy: Option<String>) -> Result<GoogleApiTestResult, String> {
    // Gemini Discovery 接口无需 API Key，正常响应必须为 2xx，适合作为真实连通性探针。
    let target_url = "https://generativelanguage.googleapis.com/$discovery/rest?version=v1beta";
    let start_time = Instant::now();

    let mut client_builder = reqwest::Client::builder()
        .timeout(Duration::from_secs(6));

    if let Some(proxy_str) = custom_proxy.or_else(detect_active_local_proxy) {
        if let Ok(p) = reqwest::Proxy::all(&proxy_str) {
            client_builder = client_builder.proxy(p);
        }
    }

    let client = client_builder
        .build()
        .map_err(|e| format!("构建网络测试客户端失败: {}", e))?;

    match client.get(target_url).send().await {
        Ok(resp) => {
            let latency_ms = start_time.elapsed().as_millis() as u64;
            let status = resp.status().as_u16();
            let is_success = is_google_api_success(resp.status());
            Ok(GoogleApiTestResult {
                success: is_success,
                latency_ms,
                status_code: status,
                target_url: target_url.to_string(),
                error_msg: if is_success {
                    None
                } else {
                    Some(format!("Google API 返回 HTTP {}", status))
                },
            })
        }
        Err(err) => {
            let latency_ms = start_time.elapsed().as_millis() as u64;
            Ok(GoogleApiTestResult {
                success: false,
                latency_ms,
                status_code: 0,
                target_url: target_url.to_string(),
                error_msg: Some(err.to_string()),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_active_proxy_detection() {
        let proxy = detect_active_local_proxy();
        assert!(proxy.is_some());
        let p_str = proxy.unwrap();
        assert!(p_str.starts_with("http://127.0.0.1:"));
    }

    #[test]
    fn plugin_proxy_environment_contains_node_and_lowercase_variables() {
        let variables = plugin_proxy_environment("http://127.0.0.1:7897");

        assert!(variables.contains(&("HTTP_PROXY", "http://127.0.0.1:7897".to_string())));
        assert!(variables.contains(&("HTTPS_PROXY", "http://127.0.0.1:7897".to_string())));
        assert!(variables.contains(&("ALL_PROXY", "http://127.0.0.1:7897".to_string())));
        assert!(variables.contains(&("http_proxy", "http://127.0.0.1:7897".to_string())));
        assert!(variables.contains(&("https_proxy", "http://127.0.0.1:7897".to_string())));
        assert!(variables.contains(&("all_proxy", "http://127.0.0.1:7897".to_string())));
        assert!(variables.contains(&("NODE_USE_ENV_PROXY", "1".to_string())));
    }

    #[test]
    fn google_api_test_accepts_only_success_status_codes() {
        assert!(is_google_api_success(reqwest::StatusCode::OK));
        assert!(!is_google_api_success(reqwest::StatusCode::NOT_FOUND));
        assert!(!is_google_api_success(reqwest::StatusCode::UNAUTHORIZED));
        assert!(!is_google_api_success(
            reqwest::StatusCode::INTERNAL_SERVER_ERROR
        ));
    }
}
