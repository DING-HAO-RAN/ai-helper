pub mod antigravity;
pub mod app;
pub mod cli;
pub mod commands;
pub mod crypto;
pub mod scanner;
pub mod storage;
pub mod timezone;

/// 创建静默无窗口的子进程命令，杜绝 Windows 下任何控制台黑框闪现
pub fn create_hidden_command(program: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

#[cfg(test)]
mod tests {
    use crate::storage::{save_token, get_token_secret, delete_token, upsert_prompt, delete_prompt};

    #[test]
    fn test_storage_and_crypto_flow() {
        // 测试 Prompt
        let p = upsert_prompt(None, "测试提示词".into(), "这是一个测试内容".into(), vec!["AI".into()]).unwrap();
        assert_eq!(p.title, "测试提示词");
        delete_prompt(&p.id).unwrap();

        // 测试 Token
        let view = save_token("test_alias".into(), "ghp_1234567890abcdefghijklmn".into(), "测试备注".into(), true).unwrap();
        assert_eq!(view.alias, "test_alias");
        assert!(view.masked_token.contains("****"));

        let secret = get_token_secret(Some("test_alias")).unwrap();
        assert_eq!(secret, "ghp_1234567890abcdefghijklmn");

        let default_secret = get_token_secret(None).unwrap();
        assert_eq!(default_secret, "ghp_1234567890abcdefghijklmn");

        delete_token(&view.id).unwrap();
    }
}
