use regex::Regex;

use std::env::VarError;
use std::fmt::Write;
use std::{borrow::Cow, sync::LazyLock};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum EnvPlaceholderError {
    #[error("Missing environment variable: {0}")]
    MissingEnvVar(#[from] VarError),

    #[error("Format error: {0}")]
    Format(#[from] std::fmt::Error),
}

/// Expands environment placeholders in the given text.
///
/// Replaces occurrences of `${ENV_VAR}` with the corresponding value from the
/// process environment (via `std::env`). If a referenced variable is not set,
/// returns `ConfigError::MissingEnvVar`.
///
/// The sequence `\${` is treated as an escape and is converted to a literal `${`
/// without performing substitution.
pub fn expand_env_placeholders(input: &str) -> Result<Cow<'_, str>, EnvPlaceholderError> {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"(\\)?\$\{([A-Za-z_][A-Za-z0-9_]*)}").unwrap());

    if !RE.is_match(input) {
        return Ok(Cow::Borrowed(input));
    }

    let mut new_string = String::with_capacity(input.len());
    let mut last_match_end = 0;

    for caps in RE.captures_iter(input) {
        let match_whole = caps.get(0).unwrap();

        new_string.push_str(&input[last_match_end..match_whole.start()]);

        let is_escaped = caps.get(1).is_some();
        let name = &caps[2];

        if is_escaped {
            write!(new_string, "${{{name}}}")?;
        } else {
            let val = std::env::var(name)?;
            new_string.push_str(&val);
        }

        last_match_end = match_whole.end();
    }

    new_string.push_str(&input[last_match_end..]);

    Ok(Cow::Owned(new_string))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_replace_env_variables() {
        // Given
        unsafe {
            std::env::set_var("PATH", "sup3r-s3cr3t");
        }
        let config = r#"secret = "${PATH}""#.to_string();

        // When
        let result = expand_env_placeholders(&config).unwrap();

        // Then
        assert_eq!(result, r#"secret = "sup3r-s3cr3t""#);
    }

    #[test]
    fn should_replace_env_variables_and_keep_remaining() {
        // Given
        unsafe {
            std::env::set_var("PATH", "sup3r-s3cr3t");
        }
        let config = r#"[forwarding]
secret = "${PATH}"
method = "MODERN""#
            .to_string();

        // When
        let result = expand_env_placeholders(&config).unwrap();

        // Then
        assert_eq!(
            result,
            r#"[forwarding]
secret = "sup3r-s3cr3t"
method = "MODERN""#
        );
    }

    #[test]
    fn should_not_replace_escaped_env_variables() {
        // Given
        unsafe {
            std::env::set_var("PATH", "sup3r-s3cr3t");
        }
        let config = r#"secret = "$\{PATH}""#.to_string();

        // When
        let result = expand_env_placeholders(&config).unwrap();

        // Then
        assert_eq!(result, r#"secret = "$\{PATH}""#);
    }

    #[test]
    fn should_not_replace_partial_env_variables() {
        // Given
        let config = r#"secret = "${PATH""#.to_string();

        // When
        let result = expand_env_placeholders(&config).unwrap();

        // Then
        assert_eq!(result, r#"secret = "${PATH""#);
    }

    #[test]
    fn should_return_config_as_is_when_no_placeholder() {
        // Given
        let config = r#"secret = "another-secret""#.to_string();

        // When
        let result = expand_env_placeholders(&config).unwrap();

        // Then
        assert_eq!(result, r#"secret = "another-secret""#);
    }

    #[test]
    fn empty_config() {
        // Given
        let config = String::new();

        // When
        let result = expand_env_placeholders(&config).unwrap();

        // Then
        assert_eq!(result, "");
    }
}
