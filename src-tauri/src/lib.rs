pub mod cli;
pub mod crypto;
pub mod storage;

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
