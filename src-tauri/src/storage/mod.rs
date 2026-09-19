//! 本地数据持久化存储与便携化配置模块
//! 优先在程序可执行文件所在目录下创建并管理设置配置文件 (config.json)、
//! 核心加密数据文件 (storage.json) 与文件索引缓存 (agent_index.json)。

pub mod types;

use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use chrono::Local;
use uuid::Uuid;

use crate::crypto::{decrypt_text, encrypt_text};
use self::types::{AppConfig, AppData, GitHubTokenItem, PromptItem, TokenDisplayView};

static STORAGE_LOCK: Mutex<()> = Mutex::new(());

/// 获取程序所在文件夹根目录（优先使用当前运行可执行文件所在目录）
pub fn get_app_dir() -> PathBuf {
    if let Ok(exe_path) = std::env::current_exe() {
        if let Some(parent) = exe_path.parent() {
            return parent.to_path_buf();
        }
    }
    std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."))
}

/// 获取本地数据持久化文件 (storage.json) 的路径
pub fn get_storage_path() -> PathBuf {
    get_app_dir().join("storage.json")
}

/// 获取系统设置配置文件 (config.json) 的路径
pub fn get_config_path() -> PathBuf {
    get_app_dir().join("config.json")
}

/// 获取 agent.md 本地索引缓存文件 (agent_index.json) 的路径
pub fn get_agent_index_path() -> PathBuf {
    get_app_dir().join("agent_index.json")
}

/// 初始化工作空间文件：在程序启动时自动在程序所在文件夹创建设置配置文件与数据缓存文件
pub fn init_app_workspace() {
    let _guard = STORAGE_LOCK.lock().unwrap();
    let app_dir = get_app_dir();
    let _ = fs::create_dir_all(&app_dir);

    // 1. 初始化 config.json
    let config_path = get_config_path();
    if !config_path.exists() {
        let default_config = AppConfig::default();
        if let Ok(json) = serde_json::to_string_pretty(&default_config) {
            let _ = fs::write(&config_path, json);
        }
    }

    // 2. 初始化 storage.json
    let storage_path = get_storage_path();
    if !storage_path.exists() {
        // 尝试从旧的 AppData 目录自动迁移历史数据
        let migrated = try_migrate_legacy_storage(&storage_path);
        if !migrated {
            let default_data = AppData::default();
            if let Ok(json) = serde_json::to_string_pretty(&default_data) {
                let _ = fs::write(&storage_path, json);
            }
        }
    }

    // 3. 初始化 agent_index.json
    let index_path = get_agent_index_path();
    if !index_path.exists() {
        let empty_index = serde_json::json!({
            "updated_at": "",
            "total_count": 0,
            "agents": []
        });
        if let Ok(json) = serde_json::to_string_pretty(&empty_index) {
            let _ = fs::write(&index_path, json);
        }
    }
}

/// 尝试从旧 AppData 目录迁移数据至程序所在目录
fn try_migrate_legacy_storage(target_path: &PathBuf) -> bool {
    let legacy_dir = dirs::data_local_dir()
        .or_else(dirs::config_dir)
        .map(|d| d.join("AIHelper").join("storage.json"));

    if let Some(old_path) = legacy_dir {
        if old_path.exists() && old_path != *target_path {
            if let Ok(content) = fs::read_to_string(&old_path) {
                if !content.trim().is_empty() {
                    let _ = fs::write(target_path, content);
                    return true;
                }
            }
        }
    }
    false
}

/// 读取系统设置配置
pub fn load_config() -> AppConfig {
    let _guard = STORAGE_LOCK.lock().unwrap();
    let path = get_config_path();
    if !path.exists() {
        return AppConfig::default();
    }

    match fs::read_to_string(&path) {
        Ok(content) => serde_json::from_str::<AppConfig>(&content).unwrap_or_default(),
        Err(_) => AppConfig::default(),
    }
}

/// 保存系统设置配置
pub fn save_config(config: &AppConfig) -> Result<(), String> {
    let _guard = STORAGE_LOCK.lock().unwrap();
    let path = get_config_path();
    let mut updated = config.clone();
    updated.updated_at = Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let json = serde_json::to_string_pretty(&updated)
        .map_err(|e| format!("序列化设置失败: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workspace_paths_and_config() {
        let app_dir = get_app_dir();
        assert!(app_dir.exists());

        init_app_workspace();

        let cfg_path = get_config_path();
        assert!(cfg_path.exists());

        let storage_path = get_storage_path();
        assert!(storage_path.exists());

        let index_path = get_agent_index_path();
        assert!(index_path.exists());

        let cfg = load_config();
        assert!(cfg.auto_minimize_to_orb);
    }
}
