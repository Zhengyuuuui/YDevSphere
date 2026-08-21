//! Swift 真实路径验证（可选，`#[ignore]`，不污染默认 `cargo test`）。
//!
//! 运行：`cargo test --test swift_realpath -- --ignored`
//!
//! 验证含 `.xcodeproj` 的真实 Swift 工程被边界判定为真项目，且技术栈含
//! Swift（+Xcode）。扫描只读，不落库、不写 `~/.ydevsphere`。
//!
//! 路径硬编码自修复 prompt 中的真实工程；若本机不存在会跳过（打印提示）。

use ydevsphere_lib::core::models::ProjectKind;
use ydevsphere_lib::core::scanner;

/// 真实 Swift 路径（存在则验证；不存在则打印跳过提示）。
const REAL_PATHS: &[&str] = &[
    "/Users/zhengyusheng/Documents/manus iosapp develop/intento",
    "/Users/zhengyusheng/Documents/Lifesailing-ios/LifeSailing",
];

#[test]
#[ignore]
fn real_swift_paths_detected() {
    for path in REAL_PATHS {
        let dir = std::path::Path::new(path);
        if !dir.is_dir() {
            eprintln!("[skip] 路径不存在（本机无此工程）: {path}");
            continue;
        }

        // 直接扫描该目录作为工作区根：根含 .xcodeproj → 两阶段 → 根应为真项目。
        let out = scanner::scan_workspace(dir).unwrap_or_else(|e| {
            panic!("扫描 {path} 失败: {e}");
        });

        // 根目录本身应被识别为 Swift 真项目（Real）。
        let root_proj = out
            .projects
            .iter()
            .find(|p| p.path == dir.display().to_string())
            .unwrap_or_else(|| {
                panic!("{path} 根目录未判为项目；识别到: {:?}",
                    out.projects.iter().map(|p| &p.path).collect::<Vec<_>>())
            });

        assert_eq!(
            root_proj.kind,
            ProjectKind::Real,
            "{path} 应判为 Real 真项目"
        );
        assert_eq!(
            root_proj.language.as_deref(),
            Some("Swift"),
            "{path} language 应为 Swift"
        );

        let ids: Vec<&str> = root_proj
            .technologies
            .iter()
            .map(|t| t.id.as_str())
            .collect();
        assert!(
            ids.contains(&"swift"),
            "{path} 技术栈应含 swift，实际 {ids:?}"
        );
        assert!(
            ids.contains(&"xcode"),
            "{path} 含 .xcodeproj，技术栈应含 xcode，实际 {ids:?}"
        );
        eprintln!("[ok] {path} → kind={:?} techs={ids:?}", root_proj.kind);
    }
}
