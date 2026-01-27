//! Output structures for JSON formatting
//!
//! This module provides structures for serializing commit messages
//! into JSON format for programmatic consumption.

use serde::Serialize;

/// Commit message structure for JSON output
///
/// # Example
///
/// ```
/// use claude_commit::output::CommitMessage;
/// use serde_json;
///
/// let commit = CommitMessage {
///     message: "feat: add new feature".to_string(),
/// };
///
/// let json = serde_json::to_string(&commit).unwrap();
/// assert_eq!(json, r#"{"message":"feat: add new feature"}"#);
/// ```
#[derive(Serialize)]
pub struct CommitMessage {
    /// The generated commit message content
    pub message: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_commit_message_serialize_basic() {
        // Arrange - basic commit message
        let commit = CommitMessage {
            message: "feat: add new feature".to_string(),
        };

        // Act
        let result = serde_json::to_string(&commit);

        // Assert - should serialize to valid JSON
        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json, r#"{"message":"feat: add new feature"}"#);
    }

    #[test]
    fn test_commit_message_serialize_special_characters() {
        // Arrange - message with special characters
        let commit = CommitMessage {
            message: r#"fix: resolve "quote" issue and \backslash"#.to_string(),
        };

        // Act
        let result = serde_json::to_string(&commit);

        // Assert - special characters should be properly escaped
        assert!(result.is_ok());
        let json = result.unwrap();
        // JSON should escape quotes and backslashes
        assert!(json.contains(r#"\"quote\""#));
        assert!(json.contains(r#"\\"#));
    }

    #[test]
    fn test_commit_message_serialize_empty_message() {
        // Arrange - empty message
        let commit = CommitMessage {
            message: "".to_string(),
        };

        // Act
        let result = serde_json::to_string(&commit);

        // Assert - should serialize to empty string in JSON
        assert!(result.is_ok());
        let json = result.unwrap();
        assert_eq!(json, r#"{"message":""}"#);
    }

    #[test]
    fn test_commit_message_serialize_multiline_message() {
        // Arrange - multiline commit message
        let commit = CommitMessage {
            message: "feat: add feature\n\nThis is a longer description.\nWith multiple lines."
                .to_string(),
        };

        // Act
        let result = serde_json::to_string(&commit);

        // Assert - newlines should be escaped as \n
        assert!(result.is_ok());
        let json = result.unwrap();
        assert!(json.contains(r#"\n"#));
        assert!(json.contains("feat: add feature"));
        assert!(json.contains("longer description"));
    }

    #[test]
    fn test_commit_message_serialize_unicode_and_emoji() {
        // Arrange - message with Unicode and emoji
        let commit = CommitMessage {
            message: "feat: 日本語サポート追加 🎉🚀".to_string(),
        };

        // Act
        let result = serde_json::to_string(&commit);

        // Assert - Unicode and emoji should be preserved
        assert!(result.is_ok());
        let json = result.unwrap();
        // serde_json preserves Unicode by default
        assert!(json.contains("日本語サポート追加"));
        assert!(json.contains("🎉"));
        assert!(json.contains("🚀"));
    }

    #[test]
    fn test_commit_message_deserialize_and_verify_structure() {
        // Arrange - serialize a message first
        let original = CommitMessage {
            message: "test: verify roundtrip".to_string(),
        };
        let json = serde_json::to_string(&original).unwrap();

        // Act - parse back to verify structure
        let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();

        // Assert - should have correct structure
        assert!(parsed.is_object());
        assert_eq!(parsed["message"], "test: verify roundtrip");
    }
}
