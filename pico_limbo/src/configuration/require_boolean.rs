use serde::Deserialize;

pub fn require_true<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let b = bool::deserialize(deserializer)?;
    if b {
        Ok(true)
    } else {
        Err(serde::de::Error::custom(
            "`enabled` must be true for this configuration",
        ))
    }
}

pub fn require_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let b = bool::deserialize(deserializer)?;
    if b {
        Err(serde::de::Error::custom(
            "`enabled` must be false for this configuration",
        ))
    } else {
        Ok(false)
    }
}
