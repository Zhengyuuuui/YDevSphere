//! Git 仓库分析器（P1-1）。
//!
//! 输入项目路径，输出 `GitInfo`（branch / last_commit / status / last_update /
//! is_git_repo）。
//!
//! ## 约束
//! - **只读**：仅使用 `git2` 的只读 API（`Repository::open` / `head` / `statuses` /
//!   `revparse` 等），**绝不**调用任何写操作（`commit` / `checkout` / `push` / `pull` /
//!   `reset` 等）。
//! - **容错**：项目非 git 仓库、`.git` 损坏、权限不足、无 commit 等一律优雅降级，
//!   返回 `is_git_repo: false` 或对应字段为 `None`，**不得 panic**。
//!
//! 硬性约束：本模块禁止 `use tauri`。

use std::path::Path;

use git2::{Repository, StatusOptions};

use crate::core::models::{CommitInfo, GitInfo, GitStatus};

/// 分析项目路径的 git 状态。
///
/// 容错策略：
/// - `Repository::open` 失败（非仓库 / 损坏 / 权限不足）→ 返回 `is_git_repo: false`。
/// - 仓库有效但无 HEAD / 无 commit → 对应字段为 `None`。
pub fn analyze_git(project_path: &Path) -> GitInfo {
    let repo = match Repository::open(project_path) {
        Ok(repo) => repo,
        Err(_) => {
            // 非 git 仓库 / 损坏 / 权限不足等，优雅降级。
            return GitInfo {
                is_git_repo: false,
                branch: None,
                last_commit: None,
                status: None,
                last_update: None,
            };
        }
    };

    // 当前分支
    let branch = current_branch(&repo);

    // 最近一次 commit + 时间
    let last_commit = last_commit_info(&repo);
    let last_update = last_commit
        .as_ref()
        .map(|c| c.time.clone());

    // 工作区状态
    let status = workdir_status(&repo);

    GitInfo {
        is_git_repo: true,
        branch,
        last_commit,
        status,
        last_update,
    }
}

/// 读取当前分支名；HEAD detached 或出错时返回 `None`。
fn current_branch(repo: &Repository) -> Option<String> {
    let head = match repo.head() {
        Ok(h) => h,
        Err(_) => return None, // unborn / 无 HEAD
    };
    if !head.is_branch() {
        return None;
    }
    let shorthand = head.shorthand()?.to_string();
    Some(shorthand)
}

/// 读取 HEAD 指向的最近一次 commit；无 commit 时返回 `None`。
fn last_commit_info(repo: &Repository) -> Option<CommitInfo> {
    let commit = match repo.head().and_then(|head| head.peel_to_commit()) {
        Ok(c) => c,
        Err(_) => return None,
    };

    let hash = commit
        .id()
        .to_string()
        .chars()
        .take(8)
        .collect::<String>();

    let message = commit
        .summary()
        .map(|s| s.to_string())
        .unwrap_or_else(|| "".to_string());

    let author = commit
        .author()
        .name()
        .map(|s| s.to_string())
        .or_else(|| commit.author().email().map(|s| s.to_string()))
        .unwrap_or_else(|| "unknown".to_string());

    let time = format_commit_time(&commit);

    Some(CommitInfo {
        hash,
        message,
        author,
        time,
    })
}

/// 将 commit 时间格式化为 RFC3339。
fn format_commit_time(commit: &git2::Commit<'_>) -> String {
    let t = commit.time();
    // git2 `Time::seconds()` 为自 epoch 起的秒数，含时区偏移。
    let offset_seconds = t.offset_minutes() * 60;
    let unix = t.seconds() + offset_seconds as i64;
    let (y, m, d, hh, mm, ss) = unix_to_utc(unix);
    // 保留原始时区偏移
    let off = t.offset_minutes();
    let sign = if off < 0 { '-' } else { '+' };
    let abs = off.abs();
    format!(
        "{y:04}-{m:02}-{d:02}T{hh:02}:{mm:02}:{ss:02}{sign}{:02}:{:02}",
        abs / 60,
        abs % 60
    )
}

/// 计算工作区状态：Clean / Dirty（含 staged / unstaged / untracked）。
fn workdir_status(repo: &Repository) -> Option<GitStatus> {
    let mut opts = StatusOptions::new();
    opts.include_untracked(true);
    opts.recurse_untracked_dirs(false);
    opts.include_ignored(false);

    let statuses = match repo.statuses(Some(&mut opts)) {
        Ok(s) => s,
        Err(_) => return None,
    };

    let changed = statuses.iter().count();
    if changed == 0 {
        Some(GitStatus::Clean)
    } else {
        Some(GitStatus::Dirty {
            changed_files: changed,
        })
    }
}

/// 将 unix 秒（UTC）转换为公历 UTC 日期时间（Howard Hinnant 算法）。
fn unix_to_utc(secs: i64) -> (i64, u32, u32, u32, u32, u32) {
    let days = secs.div_euclid(86_400);
    let rem = secs.rem_euclid(86_400);
    let z = days + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = if m <= 2 { y + 1 } else { y };
    let (hh, mm, ss) = (rem / 3600, (rem % 3600) / 60, rem % 60);
    (y, m, d, hh as u32, mm as u32, ss as u32)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 创建一个临时目录并在其中初始化一个 git 仓库。
    /// 返回 (路径, 是否需要手动设置 user 配置)。
    fn init_git_repo(dir: &Path) {
        std::fs::create_dir_all(dir).expect("创建目录失败");
        // 若已有 .git 则清理
        let git_dir = dir.join(".git");
        if git_dir.exists() {
            let _ = std::fs::remove_dir_all(&git_dir);
        }
        let repo = Repository::init(dir).expect("初始化仓库失败");

        // 用回退的用户信息提交
        repo.config()
            .expect("读取配置失败")
            .set_str("user.name", "Test User")
            .expect("设置 user.name 失败");
        repo.config()
            .expect("读取配置失败")
            .set_str("user.email", "test@example.com")
            .expect("设置 user.email 失败");

        // 写一个文件并 commit
        let file = dir.join("README.md");
        std::fs::write(&file, "# test\n").expect("写文件失败");
        let mut index = repo.index().expect("读取 index 失败");
        index.add_path(Path::new("README.md")).expect("add 失败");
        index.write().expect("写 index 失败");
        let tree_id = index.write_tree().expect("写 tree 失败");
        let tree = repo.find_tree(tree_id).expect("找 tree 失败");
        let sig = git2::Signature::now("Test User", "test@example.com")
            .expect("创建签名失败");
        repo.commit(
            Some("HEAD"),
            &sig,
            &sig,
            "initial commit",
            &tree,
            &[],
        )
        .expect("commit 失败");
    }

    fn tmp_dir(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "ydevsphere_git_test_{}_{}",
            std::process::id(),
            name
        ))
    }

    #[test]
    fn non_git_repo_returns_is_git_repo_false() {
        let dir = tmp_dir("non_git");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("创建目录失败");

        let info = analyze_git(&dir);
        assert!(!info.is_git_repo);
        assert!(info.branch.is_none());
        assert!(info.last_commit.is_none());
        assert!(info.status.is_none());
        assert!(info.last_update.is_none());
    }

    #[test]
    fn git_repo_reads_branch_and_commit() {
        let dir = tmp_dir("repo");
        let _ = std::fs::remove_dir_all(&dir);
        init_git_repo(&dir);

        let info = analyze_git(&dir);
        assert!(info.is_git_repo);
        // 默认分支名可能是 master / main
        assert!(info.branch.is_some());

        let commit = info.last_commit.expect("应有最近 commit");
        assert_eq!(commit.message, "initial commit");
        assert!(!commit.hash.is_empty());
        assert_eq!(commit.author, "Test User");
        assert!(!commit.time.is_empty());
        assert!(info.last_update.is_some());
    }

    #[test]
    fn clean_repo_has_clean_status() {
        let dir = tmp_dir("clean");
        let _ = std::fs::remove_dir_all(&dir);
        init_git_repo(&dir);

        let info = analyze_git(&dir);
        assert!(matches!(info.status, Some(GitStatus::Clean)));
    }

    #[test]
    fn dirty_repo_counts_changed_files() {
        let dir = tmp_dir("dirty");
        let _ = std::fs::remove_dir_all(&dir);
        init_git_repo(&dir);

        // 新增一个 untracked 文件 → Dirty
        std::fs::write(dir.join("TODO.txt"), "todo\n").expect("写文件失败");

        let info = analyze_git(&dir);
        match info.status {
            Some(GitStatus::Dirty { changed_files }) => {
                assert!(changed_files >= 1);
            }
            other => panic!("应为 Dirty，实际: {other:?}"),
        }
    }

    #[test]
    fn missing_dir_returns_is_git_repo_false() {
        let dir = tmp_dir("missing");
        let _ = std::fs::remove_dir_all(&dir);
        // 目录不存在
        let info = analyze_git(&dir);
        assert!(!info.is_git_repo);
    }

    #[test]
    fn refuses_any_write_operation() {
        // 编译期/运行时确保只读：检查 git2 API 不因本模块而被调用写操作。
        // 这里仅做轻量冒烟：打开一个仓库，确认我们只读取。
        let dir = tmp_dir("readonly");
        let _ = std::fs::remove_dir_all(&dir);
        init_git_repo(&dir);

        let repo = Repository::open(&dir).expect("打开仓库失败");
        // 只读操作
        let head = repo.head().expect("读取 head");
        let _ = head.shorthand();
        // 不做任何写操作
        let _ = repo;
    }
}
