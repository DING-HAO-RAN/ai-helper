//! AGENT.md 智能全盘扫描模块
//! 自动检测 Windows 本地磁盘驱动器，采用基于前缀与目录特征的快速剪枝算法，
//! 智能避开巨大无用目录（Windows系统目录、node_modules、.git 等），
//! 高速定位所有 AI 工具配置与项目工作区中的 agent.md / agents.md 文件。

pub mod injector;

use std::fs;
use std::path::{Path, PathBuf};
use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use walkdir::{DirEntry, WalkDir};

use self::injector::check_has_guide;

/// 扫描出的 AGENT.md 记录项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentFileInfo {
    pub path: String,
    pub filename: String,
    pub size_bytes: u64,
    pub modified_at: String,
    pub preview: String,
    pub has_token_guide: bool,
    pub tool_or_project: String,
}

/// 获取 Windows 上所有有效的硬盘驱动器根目录（例如 C:\, D:\ 等）
pub fn get_scan_roots() -> Vec<PathBuf> {
    let mut roots = Vec::new();

    #[cfg(target_os = "windows")]
    {
        use windows_sys::Win32::Storage::FileSystem::{GetDriveTypeW, GetLogicalDrives};

        const DRIVE_REMOVABLE: u32 = 2;
        const DRIVE_FIXED: u32 = 3;

        let drive_mask = unsafe { GetLogicalDrives() };
        for i in 0..26 {
            if (drive_mask & (1 << i)) != 0 {
                let letter = (b'A' + i as u8) as char;
                // 跳过 A 和 B（通常为软驱）
                if letter == 'A' || letter == 'B' {
                    continue;
                }
                let path_str = format!("{}:\\", letter);
                let wide_path: Vec<u16> = path_str.encode_utf16().chain(std::iter::once(0)).collect();
                let drive_type = unsafe { GetDriveTypeW(wide_path.as_ptr()) };
                // 仅扫描固态硬盘、机械硬盘和移动硬盘
                if drive_type == DRIVE_FIXED || drive_type == DRIVE_REMOVABLE {
                    let p = PathBuf::from(&path_str);
                    if p.exists() {
                        roots.push(p);
                    }
                }
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    {
        if let Some(home) = dirs::home_dir() {
            roots.push(home);
        }
        roots.push(PathBuf::from("/"));
    }

    // 若未检测到盘符，则回退至用户主目录
    if roots.is_empty() {
        if let Some(home) = dirs::home_dir() {
            roots.push(home);
        }
    }

    roots
}

/// 判断目录是否属于应当跳过的系统或庞大缓存目录
fn is_ignored_directory(entry: &DirEntry) -> bool {
    let file_name = entry.file_name().to_string_lossy().to_lowercase();
    let path_str = entry.path().to_string_lossy().to_lowercase();

    // 跳过根级系统目录与无用目录
    if file_name.starts_with('$') {
        return true;
    }

    let ignored_names = [
        "system volume information",
        "windows",
        "winnt",
        "recovery",
        "node_modules",
        ".git",
        ".svn",
        ".hg",
        "target",
        "__pycache__",
        ".cargo",
        ".rustup",
        "dist",
        "build",
        "package-cache",
        "winsxs",
    ];

    if ignored_names.contains(&file_name.as_str()) {
        return true;
    }

    // 跳过 Windows 系统深层缓存
    if path_str.contains("appdata\\local\\temp") || path_str.contains("appdata\\local\\microsoft") {
        return true;
    }

    false
}

/// 判断文件名是否符合 agent.md 规范
fn is_agent_filename(filename: &str) -> bool {
    let lower = filename.to_lowercase();
    lower == "agent.md" || lower == "agents.md"
}

/// 推断所属的 AI 工具或项目名称
fn infer_tool_or_project(path: &Path) -> String {
    let path_str = path.to_string_lossy().to_lowercase();

    if path_str.contains(".cursor") {
        return "Cursor AI".to_string();
    }
    if path_str.contains(".windsurf") {
        return "Windsurf".to_string();
    }
    if path_str.contains(".cline") {
        return "Cline".to_string();
    }
    if path_str.contains(".continue") {
        return "Continue.dev".to_string();
    }
    if path_str.contains(".github") {
        return "GitHub Copilot / Workflow".to_string();
    }
    if path_str.contains("dsh") || path_str.contains("deepseek") {
        return "DeepSeek Harness".to_string();
    }

    // 默认取上级目录名作为项目名称
    if let Some(parent) = path.parent() {
        if let Some(name) = parent.file_name() {
            return name.to_string_lossy().to_string();
        }
    }

    "未知项目".to_string()
}

/// 执行全盘扫描，收集所有 agent.md 信息
pub fn scan_all_agents() -> Vec<AgentFileInfo> {
    let roots = get_scan_roots();
    let mut results = Vec::new();

    for root in roots {
        let walker = WalkDir::new(root)
            .follow_links(false)
            .into_iter()
            .filter_entry(|e| {
                // 如果是目录，检查是否需要排除剪枝
                if e.file_type().is_dir() {
                    !is_ignored_directory(e)
                } else {
                    true
                }
            });

        for entry_res in walker {
            let entry = match entry_res {
                Ok(e) => e,
                Err(_) => continue, // 忽略权限不足等 IO 错误
            };

            if !entry.file_type().is_file() {
                continue;
            }

            let file_name = entry.file_name().to_string_lossy();
            if !is_agent_filename(&file_name) {
                continue;
            }

            let path = entry.path();
            let metadata = match entry.metadata() {
                Ok(m) => m,
                Err(_) => continue,
            };

            let modified_at = metadata
                .modified()
                .ok()
                .map(|time| {
                    let dt: DateTime<Local> = time.into();
                    dt.format("%Y-%m-%d %H:%M:%S").to_string()
                })
                .unwrap_or_else(|| "未知时间".to_string());

            // 读取前 200 字符作为预览
            let (preview, has_guide) = match fs::read_to_string(path) {
                Ok(content) => {
                    let guide_present = check_has_guide(&content);
                    let mut preview_str = content
                        .lines()
                        .take(3)
                        .collect::<Vec<&str>>()
                        .join(" ");
                    if preview_str.chars().count() > 150 {
                        preview_str = preview_str.chars().take(150).collect::<String>() + "...";
                    }
                    (preview_str, guide_present)
                }
                Err(_) => ("(无法读取文本内容)".to_string(), false),
            };

            let tool_or_project = infer_tool_or_project(path);

            results.push(AgentFileInfo {
                path: path.to_string_lossy().to_string(),
                filename: file_name.to_string(),
                size_bytes: metadata.len(),
                modified_at,
                preview,
                has_token_guide: has_guide,
                tool_or_project,
            });
        }
    }

    // 优先按更新时间降序排序
    results.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_agent_filename() {
        assert!(is_agent_filename("agent.md"));
        assert!(is_agent_filename("AGENT.MD"));
        assert!(is_agent_filename("Agents.md"));
        assert!(is_agent_filename("agents.MD"));
        assert!(!is_agent_filename("readme.md"));
        assert!(!is_agent_filename("agent.json"));
    }

    #[test]
    fn test_infer_tool() {
        let path = Path::new("C:\\Users\\user\\.cursor\\rules\\agent.md");
        assert_eq!(infer_tool_or_project(path), "Cursor AI");

        let proj_path = Path::new("D:\\Projects\\MyWebApp\\AGENT.md");
        assert_eq!(infer_tool_or_project(proj_path), "MyWebApp");
    }
}
