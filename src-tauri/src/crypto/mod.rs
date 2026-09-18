//! Windows DPAPI 安全加密解密模块
//! 该模块利用 Windows 操作系统级的 CryptProtectData / CryptUnprotectData API，
//! 使用当前登录用户的安全凭证来加密用户的敏感数据（如 GitHub Token），
//! 确保在本地磁盘上仅保存加密后的密文，即使文件被盗取也无法被其他用户或设备解密。

use base64::prelude::*;

#[cfg(target_os = "windows")]
use windows_sys::Win32::Foundation::LocalFree;
#[cfg(target_os = "windows")]
use windows_sys::Win32::Security::Cryptography::{
    CryptProtectData, CryptUnprotectData, CRYPT_INTEGER_BLOB,
};

/// 加密明文字符串，返回 Base64 编码的密文字符串
pub fn encrypt_text(plain_text: &str) -> Result<String, String> {
    if plain_text.is_empty() {
        return Ok(String::new());
    }

    #[cfg(target_os = "windows")]
    {
        let plain_bytes = plain_text.as_bytes();
        let data_in = CRYPT_INTEGER_BLOB {
            cbData: plain_bytes.len() as u32,
            pbData: plain_bytes.as_ptr() as *mut u8,
        };
        let mut data_out = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };

        // 调用 Windows 原生 DPAPI 加密
        let success = unsafe {
            CryptProtectData(
                &data_in as *const _,
                std::ptr::null(),     // 无额外描述
                std::ptr::null_mut(), // 无额外熵
                std::ptr::null_mut(), // 保留
                std::ptr::null_mut(), // 无提示结构体
                0,                    // 默认标志
                &mut data_out as *mut _,
            )
        };

        if success == 0 || data_out.pbData.is_null() {
            return Err("Windows DPAPI 加密失败".to_string());
        }

        let encrypted_bytes = unsafe {
            let slice = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize);
            let vec = slice.to_vec();
            LocalFree(data_out.pbData as _);
            vec
        };

        Ok(BASE64_STANDARD.encode(encrypted_bytes))
    }

    #[cfg(not(target_os = "windows"))]
    {
        // 非 Windows 平台的模拟回退（用于交叉测试）
        let encoded = BASE64_STANDARD.encode(plain_text.as_bytes());
        Ok(format!("SIM_{}", encoded))
    }
}

/// 解密 Base64 编码的密文字符串，返回明文字符串
pub fn decrypt_text(cipher_base64: &str) -> Result<String, String> {
    if cipher_base64.is_empty() {
        return Ok(String::new());
    }

    #[cfg(target_os = "windows")]
    {
        let cipher_bytes = BASE64_STANDARD
            .decode(cipher_base64)
            .map_err(|e| format!("Base64 解码失败: {}", e))?;

        let data_in = CRYPT_INTEGER_BLOB {
            cbData: cipher_bytes.len() as u32,
            pbData: cipher_bytes.as_ptr() as *mut u8,
        };
        let mut data_out = CRYPT_INTEGER_BLOB {
            cbData: 0,
            pbData: std::ptr::null_mut(),
        };

        // 调用 Windows 原生 DPAPI 解密
        let success = unsafe {
            CryptUnprotectData(
                &data_in as *const _,
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                std::ptr::null_mut(),
                0,
                &mut data_out as *mut _,
            )
        };

        if success == 0 || data_out.pbData.is_null() {
            return Err("Windows DPAPI 解密失败: 凭据已被篡改或无权访问".to_string());
        }

        let decrypted_bytes = unsafe {
            let slice = std::slice::from_raw_parts(data_out.pbData, data_out.cbData as usize);
            let vec = slice.to_vec();
            LocalFree(data_out.pbData as _);
            vec
        };

        String::from_utf8(decrypted_bytes).map_err(|e| format!("UTF-8 解析失败: {}", e))
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(rest) = cipher_base64.strip_prefix("SIM_") {
            let bytes = BASE64_STANDARD
                .decode(rest)
                .map_err(|e| format!("Base64 解码失败: {}", e))?;
            String::from_utf8(bytes).map_err(|e| format!("UTF-8 解析失败: {}", e))
        } else {
            Err("非 Windows 平台密文格式不匹配".to_string())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encryption_and_decryption_cycle() {
        let original = "ghp_MockGithubSecretToken1234567890abcdef";
        let encrypted = encrypt_text(original).expect("加密应成功");
        assert_ne!(original, encrypted);
        assert!(!encrypted.is_empty());

        let decrypted = decrypt_text(&encrypted).expect("解密应成功");
        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_empty_string() {
        let res = encrypt_text("").unwrap();
        assert_eq!(res, "");
        let dec = decrypt_text("").unwrap();
        assert_eq!(dec, "");
    }
}
