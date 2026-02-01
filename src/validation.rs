//! Input validation for commit message generation
//!
//! This module provides validation functions to ensure input sizes
//! are within acceptable limits before processing.

use anyhow::Result;

/// Maximum allowed prompt size in bytes (1MB)
pub const MAX_PROMPT_SIZE: usize = 1_000_000;

/// Size of the separator between prompt template and diff ("\n\n")
const SEPARATOR_SIZE: usize = 2;

/// Validate that the combined prompt size is within limits
///
/// This function checks the total size BEFORE string allocation to prevent
/// excessive memory usage from large git diffs.
///
/// # Arguments
///
/// * `prompt_template` - The prompt template from configuration
/// * `diff` - The git diff content
///
/// # Returns
///
/// * `Result<()>` - Ok if size is within limits, Err otherwise
///
/// # Errors
///
/// * Total size exceeds `MAX_PROMPT_SIZE` (1MB)
///
/// # Example
///
/// ```
/// use claude_commit::validation::validate_prompt_size;
///
/// let prompt = "Generate a commit message:";
/// let diff = "+added line\n-removed line";
///
/// assert!(validate_prompt_size(prompt, diff).is_ok());
/// ```
pub fn validate_prompt_size(prompt_template: &str, diff: &str) -> Result<()> {
    let total_size = prompt_template.len() + SEPARATOR_SIZE + diff.len();

    if total_size > MAX_PROMPT_SIZE {
        anyhow::bail!(
            "Prompt size ({} bytes) exceeds maximum allowed size ({} bytes). \
             Consider reducing the size of staged changes or splitting into multiple commits.",
            total_size,
            MAX_PROMPT_SIZE
        );
    }

    Ok(())
}

/// Calculate the total prompt size without allocating the combined string
///
/// # Arguments
///
/// * `prompt_template` - The prompt template from configuration
/// * `diff` - The git diff content
///
/// # Returns
///
/// * `usize` - Total size in bytes including separator
///
/// # Example
///
/// ```
/// use claude_commit::validation::calculate_prompt_size;
///
/// let prompt = "Generate a commit message:";
/// let diff = "+added line";
///
/// let size = calculate_prompt_size(prompt, diff);
/// assert_eq!(size, prompt.len() + 2 + diff.len());
/// ```
pub fn calculate_prompt_size(prompt_template: &str, diff: &str) -> usize {
    prompt_template.len() + SEPARATOR_SIZE + diff.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_prompt_size_within_limit() {
        // Arrange - small prompt and diff
        let prompt = "Generate a commit message:";
        let diff = "+added line\n-removed line";

        // Act
        let result = validate_prompt_size(prompt, diff);

        // Assert - should succeed
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_prompt_size_exactly_at_limit() {
        // Arrange - exactly 1MB total size
        let prompt = "Generate:";
        let diff_size = MAX_PROMPT_SIZE - prompt.len() - SEPARATOR_SIZE;
        let diff = "+".repeat(diff_size);

        // Act
        let result = validate_prompt_size(prompt, &diff);

        // Assert - should succeed (exactly at limit)
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_prompt_size_just_over_limit() {
        // Arrange - 1 byte over 1MB
        let prompt = "Generate:";
        let diff_size = MAX_PROMPT_SIZE - prompt.len() - SEPARATOR_SIZE + 1;
        let diff = "+".repeat(diff_size);

        // Act
        let result = validate_prompt_size(prompt, &diff);

        // Assert - should fail
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("exceeds maximum allowed size"));
        assert!(error_msg.contains(&MAX_PROMPT_SIZE.to_string()));
    }

    #[test]
    fn test_validate_prompt_size_large_diff() {
        // Arrange - very large diff (10MB)
        let prompt = "Generate:";
        let diff = "+".repeat(10_000_000);

        // Act
        let result = validate_prompt_size(prompt, &diff);

        // Assert - should fail with correct size in error
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        // Total: 10,000,000 (diff) + 2 (separator) + 9 (prompt) = 10,000,011
        assert!(error_msg.contains("10000011")); // actual size
        assert!(error_msg.contains("1000000"));   // max size
    }

    #[test]
    fn test_validate_prompt_size_empty_inputs() {
        // Arrange - both empty
        let prompt = "";
        let diff = "";

        // Act
        let result = validate_prompt_size(prompt, diff);

        // Assert - should succeed (2 bytes for separator)
        assert!(result.is_ok());
    }

    #[test]
    fn test_validate_prompt_size_unicode_characters() {
        // Arrange - Unicode characters (multi-byte)
        let prompt = "日本語プロンプト 🎉";  // Multi-byte characters
        let diff = "変更内容 🚀";

        // Act
        let result = validate_prompt_size(prompt, diff);

        // Assert - should succeed and count bytes correctly
        assert!(result.is_ok());
        let total = calculate_prompt_size(prompt, diff);
        // Verify it counts bytes, not characters
        assert!(total > prompt.chars().count() + diff.chars().count());
    }

    #[test]
    fn test_calculate_prompt_size_basic() {
        // Arrange
        let prompt = "Generate:";
        let diff = "+line";

        // Act
        let size = calculate_prompt_size(prompt, diff);

        // Assert - should be sum of both plus separator
        assert_eq!(size, prompt.len() + SEPARATOR_SIZE + diff.len());
        assert_eq!(size, 9 + 2 + 5);
        assert_eq!(size, 16);
    }

    #[test]
    fn test_calculate_prompt_size_empty() {
        // Arrange
        let prompt = "";
        let diff = "";

        // Act
        let size = calculate_prompt_size(prompt, diff);

        // Assert - should be just separator size
        assert_eq!(size, SEPARATOR_SIZE);
        assert_eq!(size, 2);
    }

    #[test]
    fn test_calculate_prompt_size_large_input() {
        // Arrange - large inputs
        let prompt = "A".repeat(500_000);
        let diff = "B".repeat(499_998);

        // Act
        let size = calculate_prompt_size(&prompt, &diff);

        // Assert - should be exactly at limit
        assert_eq!(size, MAX_PROMPT_SIZE);
        assert_eq!(size, 1_000_000);
    }

    #[test]
    fn test_validate_prompt_size_error_message_format() {
        // Arrange - exceeds limit
        let prompt = "X".repeat(600_000);
        let diff = "Y".repeat(500_000);

        // Act
        let result = validate_prompt_size(&prompt, &diff);

        // Assert - verify error message contains helpful information
        assert!(result.is_err());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("1100002 bytes")); // actual size
        assert!(error_msg.contains("1000000 bytes")); // max size
        assert!(error_msg.contains("Consider reducing"));
        assert!(error_msg.contains("splitting into multiple commits"));
    }
}
