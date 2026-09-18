//! AGENT.md 标记块安全注入模块
//! 使用专用防重防破坏注释标记块包裹 GitHub 访问凭据使用指引，
//! 支持自动识别已有标记块进行无缝替换，未标记文件自动在末尾追加。

use std::fs;
use std::path::Path;
use serde::{Deserialize, Serialize};

pub const MARKER_START: &str = "<!-- AI-HELPER:GITHUB-CREDENTIALS-START -->";
pub const MARKER_END: &str = "<!-- AI-HELPER:GITHUB-CREDENTIALS-END -->";

/// 标准注入内容模板
pub fn get_token_guide_block() -> String {
    format!(
        "{}\n## 🔑 GitHub Access Credentials (AI Helper Automated Guide)\n\
        When you need to access private GitHub repositories or perform Git/API operations, \
        you can dynamically retrieve the decrypted GitHub Token via:\n\
        - **PowerShell**:\n  ```powershell\n  $env:GITHUB_TOKEN = (ai-helper token get)\n  ```\n\
        - **Bash / Linux / Mac**:\n  ```bash\n  export GITHUB_TOKEN=$(ai-helper token get)\n  ```\n\
        - **Direct Output**:\n  `ai-helper token get`\n\
        {}\n",
        MARKER_START, MARKER_END
    )
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectSummary {
    pub total: usize,
    pub success: usize,
    pub failed: usize,
    pub details: Vec<InjectResultItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InjectResultItem {
    pub path: String,
    pub success: bool,
    pub message: String,
}

/// 检查文件内容是否已包含标记块
pub fn check_has_guide(content: &str) -> bool {
    content.contains(MARKER_START) && content.contains(MARKER_END)
}

/// 安全注入或更新单个文件的凭据指引
pub fn inject_file(path_str: &str) -> Result<bool, String> {
    let path = Path::new(path_str);
    if !path.exists() {
        return Err("目标文件不存在".to_string());
    }

    let content = fs::read_to_string(path)
        .map_err(|e| format!("读取文件失败: {}", e))?;

    let guide_block = get_token_guide_block();

    let new_content = if check_has_guide(&content) {
        // 替换已存在的标记块
        if let Some(start_idx) = content.find(MARKER_START) {
            if let Some(end_rel_idx) = content[start_idx..].find(MARKER_END) {
                let end_idx = start_idx + end_rel_idx + MARKER_END.len();
                let mut updated = String::new();
                updated.push_str(&content[..start_idx]);
                updated.push_str(guide_block.trim());
                if end_idx < content.len() {
                    updated.push_str(&content[end_idx..]);
                } else {
                    updated.push('\n');
                }
                updated
            } else {
                format!("{}\n\n{}", content.trim_end(), guide_block)
            }
        } else {
            format!("{}\n\n{}", content.trim_end(), guide_block)
        }
    } else {
        // 在末尾安全追加
        if content.trim().is_empty() {
            guide_block
        } else {
            format!("{}\n\n{}", content.trim_end(), guide_block)
        }
    };

    fs::write(path, new_content).map_err(|e| format!("写入更新内容失败: {}", e))?;
    Ok(true)
}

/// 批量对所有文件进行注入
pub fn inject_all_files(paths: &[String]) -> InjectSummary {
    let mut success_count = 0;
    let mut failed_count = 0;
    let mut details = Vec::new();

    for p in paths {
        match inject_file(p) {
            Ok(_) => {
                success_count += 1;
                details.push(InjectResultItem {
                    path: p.clone(),
                    success: true,
                    message: "成功写入令牌使用指南".to_string(),
                });
            }
            Err(e) => {
                failed_count += 1;
                details.push(InjectResultItem {
                    path: p.clone(),
                    success: false,
                    message: e,
                });
            }
        }
    }

    InjectSummary {
        total: paths.len(),
        success: success_count,
        failed: failed_count,
        details,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_inject_and_replace() {
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "# Project Agent Guidelines\nSome original rules.").unwrap();
        let path_str = tmp.path().to_str().unwrap();

        // 第一次注入
        let res = inject_file(path_str);
        assert!(res.is_ok());

        let read_back = fs::read_to_string(path_str).unwrap();
        assert!(check_has_guide(&read_back));
        assert!(read_back.contains("# Project Agent Guidelines"));
        assert!(read_back.contains(MARKER_START));

        // 第二次注入（应当无缝替换，而非重复追加）
        let res2 = inject_file(path_str);
        assert!(res2.is_ok());

        let read_back2 = fs::read_to_string(path_str).unwrap();
        let start_count = read_back2.matches(MARKER_START).count();
        assert_eq!(start_count, 1, "标记块不应重复追加");
    }
}
