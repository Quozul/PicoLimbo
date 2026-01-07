use regex::{Captures, Regex};

use crate::configuration::config::ConfigError;
use std::{borrow::Cow, sync::LazyLock};

/// Expands environment placeholders in the given text.
///
/// Replaces occurrences of `${ENV_VAR}` with the corresponding value from the
/// process environment (via `std::env`). If a referenced variable is not set,
/// returns `ConfigError::MissingEnvVar`.
///
/// The sequence `\${` is treated as an escape and is converted to a literal `${`
/// without performing substitution.
pub fn expand_env_placeholders<'a>(input: &'a str) -> Result<Cow<'a, str>, ConfigError> {
    static RE: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r#"(\\)?\$\{([A-Za-z_][A-Za-z0-9_]*)\}"#).unwrap());

    for caps in RE.captures_iter(input) {
        let is_escaped = caps.get(1).is_some();
        let name = &caps[2];
        if is_escaped {
            continue;
        }
        if std::env::var(name).is_err() {
            return Err(ConfigError::MissingEnvVar(name.to_string()));
        }
    }

    let out = RE.replace_all(input, |caps: &Captures| {
        let is_escaped = caps.get(1).is_some();
        let name = &caps[2];
        if is_escaped {
            format!("${{{}}}", name)
        } else {
            std::env::var(name).unwrap()
        }
    });
    Ok(out)
}
