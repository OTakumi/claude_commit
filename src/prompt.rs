//! Prompt construction and validation for commit message generation
//!
//! This module handles building prompts from templates and git diffs,
//! and ensures they are within acceptable size limits.

use anyhow::Result;

/// Default maximum allowed prompt size in bytes (1MB)
pub const DEFAULT_MAX_PROMPT_SIZE: usize = 1_000_000;

/// Build a prompt by combining the prompt template and git diff
///
/// The final prompt structure is:
/// ```text
/// {prompt_template}
///
/// {git_diff}
/// ```
///
/// # Arguments
///
/// * `diff` - Git diff content
/// * `prompt_template` - Prompt template from configuration
/// * `max_size` - Maximum allowed combined size in bytes
///
/// # Returns
///
/// * `Result<String>` - Complete prompt to send to Claude
///
/// # Errors
///
/// * Combined prompt size exceeds `max_size`
///
/// # Example
///
/// ```
/// use claude_commit::prompt::build_prompt;
///
/// let prompt_template = "Generate a commit message:";
/// let diff = "+added line";
/// let prompt = build_prompt(diff, prompt_template, 1_000_000).unwrap();
/// assert_eq!(prompt, "Generate a commit message:\n\n+added line");
/// ```
pub fn build_prompt(diff: &str, prompt_template: &str, max_size: usize) -> Result<String> {
    // Validate size BEFORE allocating the combined string
    let combined_size = prompt_template.len() + 2 + diff.len(); // 2 = "\n\n"

    if combined_size > max_size {
        anyhow::bail!(
            "Prompt size ({} bytes) exceeds maximum allowed size ({} bytes). \
             Consider reducing the size of staged changes or splitting into multiple commits.",
            combined_size,
            max_size
        );
    }

    Ok(format!("{}\n\n{}", prompt_template, diff))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_prompt_basic() {
        // Arrange - setup test data
        let diff = "diff --git a/file.txt b/file.txt\n+new line";
        let prompt_template = "Generate a commit message:";

        // Act - execute the function
        let result = build_prompt(diff, prompt_template, DEFAULT_MAX_PROMPT_SIZE).unwrap();

        // Assert - verify the result
        assert_eq!(
            result,
            "Generate a commit message:\n\ndiff --git a/file.txt b/file.txt\n+new line"
        );
    }

    #[test]
    fn test_build_prompt_empty_diff() {
        // Arrange - empty diff
        let diff = "";
        let prompt_template = "Generate a commit message:";

        // Act
        let result = build_prompt(diff, prompt_template, DEFAULT_MAX_PROMPT_SIZE).unwrap();

        // Assert - should still include prompt with empty diff
        assert_eq!(result, "Generate a commit message:\n\n");
    }

    #[test]
    fn test_build_prompt_empty_prompt() {
        // Arrange - empty prompt
        let diff = "diff --git a/file.txt b/file.txt\n+new line";
        let prompt_template = "";

        // Act
        let result = build_prompt(diff, prompt_template, DEFAULT_MAX_PROMPT_SIZE).unwrap();

        // Assert - should have two newlines before diff
        assert_eq!(result, "\n\ndiff --git a/file.txt b/file.txt\n+new line");
    }

    #[test]
    fn test_build_prompt_both_empty() {
        // Arrange - both empty
        let diff = "";
        let prompt_template = "";

        // Act
        let result = build_prompt(diff, prompt_template, DEFAULT_MAX_PROMPT_SIZE).unwrap();

        // Assert - should be just two newlines
        assert_eq!(result, "\n\n");
    }

    #[test]
    fn test_build_prompt_special_characters() {
        // Arrange - special characters including newlines, Unicode, and emojis
        let diff =
            "diff --git a/日本語.txt b/日本語.txt\n+こんにちは 🎉\n+Special: \t\\n\"quotes\"";
        let prompt_template = "Prompt with 絵文字 🚀 and\nmultiple\nlines";

        // Act
        let result = build_prompt(diff, prompt_template, DEFAULT_MAX_PROMPT_SIZE).unwrap();

        // Assert - all special characters should be preserved
        assert!(result.contains("絵文字 🚀"));
        assert!(result.contains("こんにちは 🎉"));
        assert!(result.contains("multiple\nlines"));
        assert!(result.contains("Special: \t\\n\"quotes\""));
    }

    #[test]
    fn test_build_prompt_multiline_prompt() {
        // Arrange - multiline prompt
        let diff = "+added line";
        let prompt_template = "Line 1\nLine 2\nLine 3";

        // Act
        let result = build_prompt(diff, prompt_template, DEFAULT_MAX_PROMPT_SIZE).unwrap();

        // Assert - newlines in prompt should be preserved
        assert_eq!(result, "Line 1\nLine 2\nLine 3\n\n+added line");
    }

    #[test]
    fn test_build_prompt_very_long_input() {
        // Arrange - very long diff (simulate large file changes)
        let large_diff = "diff --git a/large.txt b/large.txt\n".to_string() + &"+".repeat(10000);
        let prompt_template = "Generate commit:";

        // Act
        let result = build_prompt(&large_diff, prompt_template, DEFAULT_MAX_PROMPT_SIZE).unwrap();

        // Assert - should handle large inputs without panic
        assert!(result.starts_with("Generate commit:\n\ndiff --git"));
        assert!(result.len() > 10000);
        assert!(result.contains(&"+".repeat(100))); // verify content is there
    }

    #[test]
    fn test_build_prompt_within_size_limit() {
        // Arrange - small prompt and diff
        let prompt_template = "Generate a commit message:";
        let diff = "+added line\n-removed line";

        // Act
        let result = build_prompt(diff, prompt_template, DEFAULT_MAX_PROMPT_SIZE);

        // Assert - should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_prompt_exactly_at_limit() {
        // Arrange - exactly 1MB total size
        let prompt_template = "Generate:";
        let diff_size = DEFAULT_MAX_PROMPT_SIZE - prompt_template.len() - 2; // 2 = "\n\n"
        let diff = "+".repeat(diff_size);

        // Act
        let result = build_prompt(&diff, prompt_template, DEFAULT_MAX_PROMPT_SIZE);

        // Assert - should succeed (exactly at limit)
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_prompt_just_over_limit() {
        // Arrange - 1 byte over 1MB
        let prompt_template = "Generate:";
        let diff_size = DEFAULT_MAX_PROMPT_SIZE - prompt_template.len() - 2 + 1; // 2 = "\n\n"
        let diff = "+".repeat(diff_size);

        // Act
        let result = build_prompt(&diff, prompt_template, DEFAULT_MAX_PROMPT_SIZE);

        // Assert - should fail
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("exceeds maximum allowed size"));
        assert!(error_msg.contains(&DEFAULT_MAX_PROMPT_SIZE.to_string()));
    }

    #[test]
    fn test_build_prompt_large_diff() {
        // Arrange - very large diff (10MB)
        let prompt_template = "Generate:";
        let diff = "+".repeat(10_000_000);

        // Act
        let result = build_prompt(&diff, prompt_template, DEFAULT_MAX_PROMPT_SIZE);

        // Assert - should fail with correct size in error
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // Total: 10,000,000 (diff) + 2 (separator) + 9 (prompt) = 10,000,011
        assert!(error_msg.contains("10000011")); // actual size
        assert!(error_msg.contains("1000000"));   // max size
    }

    #[test]
    fn test_build_prompt_unicode_characters() {
        // Arrange - Unicode characters (multi-byte)
        let prompt_template = "日本語プロンプト 🎉";  // Multi-byte characters
        let diff = "変更内容 🚀";

        // Act
        let result = build_prompt(diff, prompt_template, DEFAULT_MAX_PROMPT_SIZE);

        // Assert - should succeed and count bytes correctly
        assert!(result.is_ok());
        let prompt = result.unwrap();
        // Verify it counts bytes, not characters
        assert!(prompt.len() > prompt_template.chars().count() + diff.chars().count());
    }

    #[test]
    fn test_build_prompt_error_message_format() {
        // Arrange - exceeds limit
        let prompt_template = "X".repeat(600_000);
        let diff = "Y".repeat(500_000);

        // Act
        let result = build_prompt(&diff, &prompt_template, DEFAULT_MAX_PROMPT_SIZE);

        // Assert - verify error message contains helpful information
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("1100002 bytes")); // actual size
        assert!(error_msg.contains("1000000 bytes")); // max size
        assert!(error_msg.contains("Consider reducing"));
        assert!(error_msg.contains("splitting into multiple commits"));
    }

    #[test]
    fn test_build_prompt_custom_size_limit() {
        // Arrange - custom size limit (500 bytes)
        let prompt_template = "Generate:";
        let diff = "+".repeat(400);
        let custom_limit = 500;

        // Act
        let result = build_prompt(&diff, prompt_template, custom_limit);

        // Assert - should succeed (within custom limit)
        assert!(result.is_ok());
    }

    #[test]
    fn test_build_prompt_custom_size_limit_exceeded() {
        // Arrange - custom size limit (100 bytes)
        let prompt_template = "Generate:";
        let diff = "+".repeat(200);
        let custom_limit = 100;

        // Act
        let result = build_prompt(&diff, prompt_template, custom_limit);

        // Assert - should fail (exceeds custom limit)
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("exceeds maximum allowed size"));
        assert!(error_msg.contains(&custom_limit.to_string()));
    }
}
