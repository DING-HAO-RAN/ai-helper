//! 极轻量标准库 WebSocket 客户端与 Chrome DevTools Protocol (CDP) 通信实现
//! 专用于向 Chromium / Electron (ChatGPT.exe) 调试端口发送时区覆盖指令，
//! 无需引入重量级外部依赖，零开销、极速响应。

use std::io::{Read, Write};
use std::net::TcpStream;
use std::time::Duration;

/// 解析 WebSocket URL（例如 ws://127.0.0.1:9222/devtools/page/xxx）
fn parse_ws_url(ws_url: &str) -> Result<(String, u16, String), String> {
    let stripped = ws_url
        .strip_prefix("ws://")
        .ok_or_else(|| "仅支持 ws:// 协议".to_string())?;

    let (host_port, path) = match stripped.find('/') {
        Some(idx) => (&stripped[..idx], &stripped[idx..]),
        None => (stripped, "/"),
    };

    let (host, port) = match host_port.find(':') {
        Some(idx) => {
            let h = &host_port[..idx];
            let p: u16 = host_port[idx + 1..]
                .parse()
                .map_err(|e| format!("端口号无效: {}", e))?;
            (h.to_string(), p)
        }
        None => (host_port.to_string(), 80),
    };

    Ok((host, port, path.to_string()))
}

/// 执行简易 WebSocket 握手并发送 CDP 覆盖时区指令
pub fn send_cdp_timezone_override(
    ws_url: &str,
    timezone_id: &str,
    timeout: Duration,
) -> Result<(), String> {
    let (host, port, path) = parse_ws_url(ws_url)?;
    let mut stream = TcpStream::connect((host.as_str(), port))
        .map_err(|e| format!("连接 CDP 端口失败: {}", e))?;

    stream.set_read_timeout(Some(timeout)).ok();
    stream.set_write_timeout(Some(timeout)).ok();

    // 1. 发送标准 WebSocket 升级握手请求
    let handshake_req = format!(
        "GET {} HTTP/1.1\r\n\
        Host: {}:{}\r\n\
        Upgrade: websocket\r\n\
        Connection: Upgrade\r\n\
        Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\n\
        Sec-WebSocket-Version: 13\r\n\
        \r\n",
        path, host, port
    );

    stream
        .write_all(handshake_req.as_bytes())
        .map_err(|e| format!("发送 WebSocket 握手失败: {}", e))?;

    // 2. 校验服务端握手响应
    let mut buf = [0u8; 1024];
    let n = stream
        .read(&mut buf)
        .map_err(|e| format!("读取握手响应失败: {}", e))?;

    let resp_header = String::from_utf8_lossy(&buf[..n]);
    if !resp_header.contains("101 Switching Protocols") {
        return Err(format!("WebSocket 握手被拒绝: {}", resp_header));
    }

    // 3. 构建 CDP 目标指令: Emulation.setTimezoneOverride
    let payload = serde_json::json!({
        "id": 1,
        "method": "Emulation.setTimezoneOverride",
        "params": {
            "timezoneId": timezone_id
        }
    })
    .to_string();

    // 4. 发送客户端带掩码的 WebSocket 文本帧 (Opcode 0x1)
    let frame = build_masked_ws_text_frame(payload.as_bytes());
    stream
        .write_all(&frame)
        .map_err(|e| format!("发送 CDP 时区帧失败: {}", e))?;

    Ok(())
}

/// 构造 RFC 6455 规范的客户端带掩码（Masked）文本帧
fn build_masked_ws_text_frame(payload: &[u8]) -> Vec<u8> {
    let mut frame = Vec::new();
    // FIN (0x80) | Text Opcode (0x01)
    frame.push(0x81);

    let len = payload.len();
    let mask_key: [u8; 4] = [0x12, 0x34, 0x56, 0x78];

    if len <= 125 {
        frame.push(0x80 | (len as u8));
    } else if len <= 65535 {
        frame.push(0x80 | 126);
        frame.extend_from_slice(&(len as u16).to_be_bytes());
    } else {
        frame.push(0x80 | 127);
        frame.extend_from_slice(&(len as u64).to_be_bytes());
    }

    // 加入 4 字节掩码密钥
    frame.extend_from_slice(&mask_key);

    // 载荷异或混淆后追加
    for (i, &b) in payload.iter().enumerate() {
        frame.push(b ^ mask_key[i % 4]);
    }

    frame
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ws_url() {
        let (host, port, path) =
            parse_ws_url("ws://127.0.0.1:9222/devtools/page/abc-123").unwrap();
        assert_eq!(host, "127.0.0.1");
        assert_eq!(port, 9222);
        assert_eq!(path, "/devtools/page/abc-123");
    }

    #[test]
    fn test_build_masked_frame() {
        let payload = b"hello";
        let frame = build_masked_ws_text_frame(payload);
        assert_eq!(frame[0], 0x81);
        assert_eq!(frame[1], 0x80 | 5);
        assert_eq!(frame.len(), 2 + 4 + 5);
    }
}
