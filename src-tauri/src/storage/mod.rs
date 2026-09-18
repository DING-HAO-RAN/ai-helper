//! 本地数据持久化存储模块
//! 负责将提示词和加密后的 GitHub 凭据持久化存储在用户本地的配置文件中。

pub mod types;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use chrono::Local;
use uuid::Uuid;

use crate::crypto::{decrypt_text, encrypt_text};
use self::types::{AppData, GitHubTokenItem, PromptItem, TokenDisplayView};

static STORAGE_LOCK: Mutex<()> = Mutex::new(());

/// 获取本地数据持久化文件的路径
pub fn get_storage_path() -> PathBuf {
    let base_dir = dirs::data_local_dir()
        .or_else(dirs::config_dir)
        .unwrap_or_else(|| PathBuf::from("."));
    let app_dir = base_dir.join("AIHelper");
    if !app_dir.exists() {
        let _ = fs::create_dir_all(&app_dir);
    }
    app_dir.join("storage.json")
}

/// 从本地文件中读取数据，若不存在或损坏则返回默认数据，并在损坏时安全备份
pub fn load_data() -> AppData {
    let _guard = STORAGE_LOCK.lock().unwrap();
    let path = get_storage_path();
    if !path.exists() {
        return AppData::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => {
            if content.trim().is_empty() {
                return AppData::default();
            }
            match serde_json::from_str::<AppData>(&content) {
                Ok(data) => data,
                Err(err) => {
                    eprintln!("警告: 本地存储数据解析异常: {}，正在创建安全备份副本", err);
                    let backup_path = path.with_extension("json.corrupted.bak");
                    let _ = fs::copy(&path, &backup_path);
                    AppData::default()
                }
            }
        }
        Err(_) => AppData::default(),
    }
}

/// 保存数据到本地文件中
pub fn save_data(data: &AppData) -> Result<(), String> {
    let _guard = STORAGE_LOCK.lock().unwrap();
    let path = get_storage_path();
    let json = serde_json::to_string_pretty(data)
        .map_err(|e| format!("序列化数据失败: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("写入存储文件失败: {}", e))?;
    Ok(())
}

/// 获取指定别名（若未指定则为默认）的明文 GitHub Token
pub fn get_token_secret(alias: Option<&str>) -> Result<String, String> {
    let data = load_data();
    if data.tokens.is_empty() {
        return Err("本地未配置任何 GitHub 令牌，请在 AI Helper 中添加".to_string());
    }

    let token_item = match alias {
        Some(a) if !a.is_empty() => data.tokens.iter().find(|t| t.alias == a),
        _ => data
            .tokens
            .iter()
            .find(|t| t.is_default)
            .or_else(|| data.tokens.first()),
    };

    let item = token_item.ok_or_else(|| {
        format!(
            "未找到别名为 '{}' 的 GitHub 令牌",
            alias.unwrap_or("default")
        )
    })?;

    decrypt_text(&item.encrypted_token)
}

/// 获取用于前端展示的脱敏令牌列表
pub fn list_token_views() -> Vec<TokenDisplayView> {
    let data = load_data();
    data.tokens
        .into_iter()
        .map(|t| {
            let masked = match decrypt_text(&t.encrypted_token) {
                Ok(plain) => mask_secret(&plain),
                Err(_) => "****** (解密受限)".to_string(),
            };
            TokenDisplayView {
                id: t.id,
                alias: t.alias,
                masked_token: masked,
                note: t.note,
                is_default: t.is_default,
                created_at: t.created_at,
                updated_at: t.updated_at,
            }
        })
        .collect()
}

/// 脱敏遮罩
fn mask_secret(secret: &str) -> String {
    if secret.len() <= 8 {
        "********".to_string()
    } else {
        let prefix = &secret[..4];
        let suffix = &secret[secret.len() - 4..];
        format!("{}****{}", prefix, suffix)
    }
}

/// 新增或更新提示词
pub fn upsert_prompt(
    id: Option<String>,
    title: String,
    content: String,
    tags: Vec<String>,
) -> Result<PromptItem, String> {
    let mut data = load_data();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();

    let result_item = match id {
        Some(existing_id) => {
            if let Some(pos) = data.prompts.iter().position(|p| p.id == existing_id) {
                data.prompts[pos].title = title;
                data.prompts[pos].content = content;
                data.prompts[pos].tags = tags;
                data.prompts[pos].updated_at = now;
                data.prompts[pos].clone()
            } else {
                let item = PromptItem {
                    id: existing_id,
                    title,
                    content,
                    tags,
                    created_at: now.clone(),
                    updated_at: now,
                };
                data.prompts.push(item.clone());
                item
            }
        }
        None => {
            let item = PromptItem {
                id: Uuid::new_v4().to_string(),
                title,
                content,
                tags,
                created_at: now.clone(),
                updated_at: now,
            };
            data.prompts.push(item.clone());
            item
        }
    };

    save_data(&data)?;
    Ok(result_item)
}

/// 删除提示词
pub fn delete_prompt(id: &str) -> Result<(), String> {
    let mut data = load_data();
    let original_len = data.prompts.len();
    data.prompts.retain(|p| p.id != id);
    if data.prompts.len() != original_len {
        save_data(&data)?;
    }
    Ok(())
}

/// 新增或更新 GitHub 令牌（采用 Windows DPAPI 自动加密）
pub fn save_token(
    alias: String,
    plain_token: String,
    note: String,
    is_default: bool,
) -> Result<TokenDisplayView, String> {
    let mut data = load_data();
    let now = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let encrypted = encrypt_text(&plain_token)?;

    // 若设为默认，则先取消其他令牌的默认标记
    if is_default || data.tokens.is_empty() {
        for t in &mut data.tokens {
            t.is_default = false;
        }
    }

    let default_flag = is_default || data.tokens.is_empty();

    let (item_id, masked) = if let Some(pos) = data.tokens.iter().position(|t| t.alias == alias) {
        data.tokens[pos].encrypted_token = encrypted;
        data.tokens[pos].note = note.clone();
        data.tokens[pos].is_default = default_flag;
        data.tokens[pos].updated_at = now.clone();
        (data.tokens[pos].id.clone(), mask_secret(&plain_token))
    } else {
        let new_id = Uuid::new_v4().to_string();
        let new_item = GitHubTokenItem {
            id: new_id.clone(),
            alias: alias.clone(),
            encrypted_token: encrypted,
            note: note.clone(),
            is_default: default_flag,
            created_at: now.clone(),
            updated_at: now.clone(),
        };
        data.tokens.push(new_item);
        (new_id, mask_secret(&plain_token))
    };

    save_data(&data)?;

    Ok(TokenDisplayView {
        id: item_id,
        alias,
        masked_token: masked,
        note,
        is_default: default_flag,
        created_at: now.clone(),
        updated_at: now,
    })
}

/// 删除指定令牌
pub fn delete_token(id: &str) -> Result<(), String> {
    let mut data = load_data();
    data.tokens.retain(|t| t.id != id);
    // 若删除了默认项，且还剩有令牌，将第一个设为默认
    if !data.tokens.is_empty() && !data.tokens.iter().any(|t| t.is_default) {
        data.tokens[0].is_default = true;
    }
    save_data(&data)?;
    Ok(())
}

/// 设置默认令牌
pub fn set_default_token(id: &str) -> Result<(), String> {
    let mut data = load_data();
    let mut found = false;
    for t in &mut data.tokens {
        if t.id == id {
            t.is_default = true;
            found = true;
        } else {
            t.is_default = false;
        }
    }
    if !found {
        return Err("未找到指定的令牌".to_string());
    }
    save_data(&data)?;
    Ok(())
}
