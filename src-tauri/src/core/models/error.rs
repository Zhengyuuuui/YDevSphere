//! 结构化命令错误（供 `commands/` 层返回给前端按 `code` 分支）。
//!
//! 本模块仅依赖 serde，不依赖 tauri，便于 Desktop / CLI / MCP 共享。
//!
//! 前端约定：`invoke` 的 reject 收到序列化后的 JSON 对象，按 `code` 字段
//! 分支处理，不依赖 `message` 的中文字符串（消除脆弱耦合）。

use serde::Serialize;

/// 扫描命令错误（serde 可序列化，供前端按 `code` 分支）。
#[derive(Debug, Clone, Serialize)]
pub struct ScanCommandError {
    /// 错误码：`"INVALID_DIRECTORY"` / `"IO_ERROR"` / `"DB_ERROR"` / `"INTERNAL_ERROR"`。
    pub code: String,
    /// 可读消息（中文，未来可国际化）。
    pub message: String,
}

impl ScanCommandError {
    /// 从错误码 + 消息构造。
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }

    /// 数据库错误统一映射为 `DB_ERROR`。
    pub fn db(message: impl Into<String>) -> Self {
        Self::new("DB_ERROR", message)
    }

    /// 内部错误统一映射为 `INTERNAL_ERROR`。
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", message)
    }
}

impl std::fmt::Display for ScanCommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// ScanCommandError 序列化为 JSON 对象，含 `code` + `message` 字段。
    #[test]
    fn serializes_to_code_and_message() {
        let err = ScanCommandError::new("INVALID_DIRECTORY", "不是有效目录: /bad");
        let json = serde_json::to_value(&err).expect("序列化应成功");

        assert_eq!(json["code"], "INVALID_DIRECTORY");
        assert_eq!(json["message"], "不是有效目录: /bad");
    }

    /// 辅助构造函数产生正确的 code。
    #[test]
    fn helper_constructors_set_correct_code() {
        assert_eq!(ScanCommandError::db("x").code, "DB_ERROR");
        assert_eq!(ScanCommandError::internal("x").code, "INTERNAL_ERROR");
    }
}
