use crate::configuration::require_boolean::{require_false, require_true};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum FloodgateConfig {
    Enabled(EnabledFloodgateConfig),
    Disabled(DisabledFloodgateConfig),
}

#[derive(Deserialize, Serialize)]
#[serde(default, deny_unknown_fields)]
#[allow(clippy::struct_excessive_bools)]
pub struct EnabledFloodgateConfig {
    #[serde(deserialize_with = "require_true")]
    pub enabled: bool,
    pub key_file: String,
    pub username_prefix: String,
    pub replace_spaces: bool,
    pub education: bool,
    pub education_username_prefix: String,
    pub education_uuid_legacy: bool,
}

impl Default for EnabledFloodgateConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            key_file: "key.pem".into(),
            username_prefix: ".".into(),
            replace_spaces: true,
            education: false,
            education_username_prefix: "+".into(),
            education_uuid_legacy: false,
        }
    }
}

#[derive(Deserialize, Serialize)]
pub struct DisabledFloodgateConfig {
    #[serde(deserialize_with = "require_false")]
    pub enabled: bool,
}

impl Default for FloodgateConfig {
    fn default() -> Self {
        Self::Disabled(DisabledFloodgateConfig { enabled: false })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enabled_config_fills_missing_values_from_defaults() {
        let config: FloodgateConfig = toml::from_str(
            r"
            enabled = true
            ",
        )
        .unwrap();

        let FloodgateConfig::Enabled(config) = config else {
            panic!("expected enabled configuration");
        };

        assert_eq!(config.key_file, "key.pem");
        assert_eq!(config.username_prefix, ".");
        assert!(config.replace_spaces);
        assert!(!config.education);
        assert_eq!(config.education_username_prefix, "+");
        assert!(!config.education_uuid_legacy);
    }

    #[test]
    fn disabled_config_accepts_legacy_fields() {
        let config: FloodgateConfig = toml::from_str(
            r#"
            enabled = false
            key_file = "old-key.pem"
            username_prefix = "."
            replace_spaces = true
            education = true
            education_username_prefix = "+"
            education_uuid_legacy = true
            "#,
        )
        .unwrap();

        assert!(matches!(config, FloodgateConfig::Disabled(_)));
        assert_eq!(
            toml::to_string_pretty(&config).unwrap().trim(),
            "enabled = false"
        );
    }
}
