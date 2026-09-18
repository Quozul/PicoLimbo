use aes_gcm::{
    Aes128Gcm,
    Nonce,
    aead::{Aead, KeyInit},
};
use base64::Engine;
use minecraft_protocol::prelude::Uuid;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};

use crate::configuration::floodgate::EnabledFloodgateConfig;

const IDENTIFIER: &[u8] = b"^Floodgate^";
const HEADER: &[u8] = b"^Floodgate^>";
const MAGIC: u8 = b'>';
const VERSION: u8 = 0;
const IV_LENGTH: usize = 12;
const MAX_PAYLOAD_BYTES: usize = 8192;
const MAX_CIPHERTEXT_BYTES: usize = 4096;
const MAX_USERNAME_PREFIX_BYTES: usize = 16;
const MAX_JAVA_USERNAME_BYTES: usize = 16;
const EDUCATION_UUID_MSB: u64 = 0x0000000100000001;

#[derive(Clone)]
pub struct FloodgateSettings {
    key: Option<[u8; 16]>,
    education_enabled: bool,
    username_prefix: String,
    education_prefix: String,
    replace_spaces: bool,
    education_uuid_legacy: bool,
}

#[derive(Clone)]
pub struct FloodgateData {
    pub username: String,
    pub xuid: String,
    pub ip: String,
    pub education: bool,
    pub tenant_id: String,
    pub ad_role: i32,
}

impl Default for FloodgateSettings {
    fn default() -> Self {
        Self {
            key: None,
            education_enabled: false,
            username_prefix: String::new(),
            education_prefix: String::new(),
            replace_spaces: true,
            education_uuid_legacy: false,
        }
    }
}

impl FloodgateSettings {
    pub fn from_config(config: &EnabledFloodgateConfig) -> Result<Self, String> {
        validate_prefix(&config.username_prefix, "Floodgate username prefix")?;
        validate_prefix(
            &config.education_username_prefix,
            "Education username prefix",
        )?;

        Ok(Self {
            key: Some(load_key(&config.key_file)?),
            education_enabled: config.education,
            username_prefix: config.username_prefix.clone(),
            education_prefix: config.education_username_prefix.clone(),
            replace_spaces: config.replace_spaces,
            education_uuid_legacy: config.education_uuid_legacy,
        })
    }

    pub fn parse_hostname(
        &self,
        hostname: &str,
    ) -> Result<(String, Option<FloodgateData>), String> {
        let Some(key) = self.key.as_ref() else {
            return Ok((hostname.to_string(), None));
        };

        let mut clean_parts = Vec::new();
        let mut floodgate_data = None;

        for part in hostname.split('\0') {
            if let Some(version) = floodgate_version(part.as_bytes()) {
                if version != VERSION as i32 {
                    return Err(format!("Unsupported Floodgate data version: {version}"));
                }

                if floodgate_data.is_some() {
                    return Err("Multiple Floodgate payloads were provided".to_string());
                }

                let decrypted = decrypt(key, part)?;
                let data = parse_data(&decrypted)?;

                if data.education && !self.education_enabled {
                    return Err(
                        "Education Floodgate data received but education support is disabled"
                            .to_string(),
                    );
                }

                floodgate_data = Some(data);
            } else {
                clean_parts.push(part);
            }
        }

        let Some(data) = floodgate_data else {
            return Ok((hostname.to_string(), None));
        };

        Ok((clean_parts.join("\0"), Some(data)))
    }

    pub fn game_profile(&self, data: &FloodgateData) -> Result<(String, Uuid), String> {
        let prefix = if data.education {
            &self.education_prefix
        } else {
            &self.username_prefix
        };

        let max_username_bytes = MAX_JAVA_USERNAME_BYTES.saturating_sub(prefix.len());
        let username = format!(
            "{}{}",
            prefix,
            truncate_utf8(&data.username, max_username_bytes)
        );
        let username = if self.replace_spaces {
            username.replace(' ', "_")
        } else {
            username
        };

        let uuid = if data.education {
            if self.education_uuid_legacy {
                legacy_education_uuid(&data.tenant_id, &data.username)
            } else {
                modern_education_uuid(&data.xuid)?
            }
        } else {
            let xuid = data
                .xuid
                .parse::<i64>()
                .map_err(|_| "Floodgate xuid is not a valid 64-bit integer".to_string())?;
            Uuid::from_u64_pair(0, xuid as u64)
        };

        Ok((username, uuid))
    }
}

fn load_key(value: &str) -> Result<[u8; 16], String> {
    let path = Path::new(value);
    let data = fs::read(path)
        .map_err(|error| format!("Failed to read Floodgate key '{}': {error}", path.display()))?;

    if data.len() != 16 {
        return Err(format!(
            "Floodgate AES key must be exactly 16 bytes, got {}",
            data.len()
        ));
    }

    let mut key = [0u8; 16];
    key.copy_from_slice(&data);
    Ok(key)
}

fn floodgate_version(value: &[u8]) -> Option<i32> {
    if value.len() <= IDENTIFIER.len() || !value.starts_with(IDENTIFIER) {
        return None;
    }

    Some(i32::from(value[IDENTIFIER.len()]) - i32::from(MAGIC))
}

fn decrypt(key: &[u8; 16], value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    if bytes.len() <= HEADER.len() || !bytes.starts_with(HEADER) {
        return Err("Invalid Floodgate header".to_string());
    }

    let payload = &bytes[HEADER.len()..];
    if payload.len() > MAX_PAYLOAD_BYTES {
        return Err("Floodgate payload is too large".to_string());
    }

    let Some(separator) = payload.iter().position(|byte| *byte == b'!') else {
        return Err("Invalid Floodgate payload".to_string());
    };

    let iv = base64::engine::general_purpose::STANDARD
        .decode(&payload[..separator])
        .map_err(|_| "Invalid Floodgate IV encoding".to_string())?;
    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(&payload[separator + 1..])
        .map_err(|_| "Invalid Floodgate ciphertext encoding".to_string())?;

    if iv.len() != IV_LENGTH {
        return Err("Invalid Floodgate IV length".to_string());
    }

    if ciphertext.is_empty() || ciphertext.len() > MAX_CIPHERTEXT_BYTES {
        return Err("Invalid Floodgate ciphertext length".to_string());
    }

    let cipher =
        Aes128Gcm::new_from_slice(key).map_err(|_| "Invalid Floodgate AES key".to_string())?;
    let nonce = Nonce::from_slice(&iv);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| "Floodgate authentication failed".to_string())?;

    String::from_utf8(plaintext).map_err(|_| "Floodgate data is not valid UTF-8".to_string())
}

fn parse_data(data: &str) -> Result<FloodgateData, String> {
    let fields: Vec<&str> = data.split('\0').collect();

    match fields.len() {
        12 => {}
        15 if fields[12] == "1" => {}
        length => {
            return Err(format!("Invalid Floodgate data field count: {length}"));
        }
    }

    if fields[9] != "0" && fields[9] != "1" {
        return Err("Invalid Floodgate proxy flag".to_string());
    }

    let education = fields.len() == 15;
    let tenant_id = if education {
        fields[13].to_string()
    } else {
        String::new()
    };
    let ad_role = if education {
        fields[14]
            .parse::<i32>()
            .map_err(|_| "Invalid Floodgate ad role".to_string())?
    } else {
        -1
    };

    Ok(FloodgateData {
        username: fields[1].to_string(),
        xuid: fields[2].to_string(),
        ip: fields[7].to_string(),
        education,
        tenant_id,
        ad_role,
    })
}

fn modern_education_uuid(oid: &str) -> Result<Uuid, String> {
    let parsed = uuid::Uuid::parse_str(oid)
        .map_err(|_| "EduFloodgate xuid is not a valid Entra OID".to_string())?;
    let value = parsed.as_u128();
    let msb = (value >> 64) as u64;
    let lsb = value as u64;

    let upper = ((msb >> 16) << 12) | (msb & 0xFFF);
    let lower = (lsb << 2) >> 60;

    Ok(Uuid::from_u64_pair(
        EDUCATION_UUID_MSB,
        (upper << 4) | lower,
    ))
}

fn legacy_education_uuid(tenant_id: &str, username: &str) -> Uuid {
    let mut digest = Sha256::new();
    digest.update(tenant_id.as_bytes());
    digest.update(b":");
    digest.update(username.as_bytes());
    let hash = digest.finalize();

    let mut lsb = 0u64;
    for byte in hash.iter().take(8) {
        lsb = (lsb << 8) | u64::from(*byte);
    }

    Uuid::from_u64_pair(EDUCATION_UUID_MSB, lsb)
}

fn validate_prefix(prefix: &str, name: &str) -> Result<(), String> {
    if prefix.len() > MAX_USERNAME_PREFIX_BYTES {
        return Err(format!("{name} must be at most 16 bytes"));
    }
    if prefix.contains('\0') {
        return Err(format!("{name} must not contain NUL bytes"));
    }
    Ok(())
}

fn truncate_utf8(value: &str, max: usize) -> String {
    if value.len() <= max {
        return value.to_string();
    }

    let mut end = max;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }

    value[..end].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use aes_gcm::aead::Aead;

    fn settings(key: [u8; 16]) -> FloodgateSettings {
        FloodgateSettings {
            key: Some(key),
            education_enabled: true,
            username_prefix: ".".to_string(),
            education_prefix: "+".to_string(),
            replace_spaces: true,
            education_uuid_legacy: false,
        }
    }

    fn payload(key: [u8; 16], data: &str) -> String {
        let cipher = Aes128Gcm::new_from_slice(&key).unwrap();
        let iv = [7u8; IV_LENGTH];
        let nonce = Nonce::from_slice(&iv);
        let ciphertext = cipher.encrypt(nonce, data.as_bytes()).unwrap();

        format!(
            "{}{}!{}",
            std::str::from_utf8(HEADER).unwrap(),
            base64::engine::general_purpose::STANDARD.encode(iv),
            base64::engine::general_purpose::STANDARD.encode(ciphertext)
        )
    }

    #[test]
    fn decrypts_standard_floodgate_payload() {
        let key = [1u8; 16];
        let data = "0\0Player\0-1\01\0en_US\02\01\0127.0.0.1\0\00\00\0123\0verify";
        let encoded = payload(key, data);
        let (hostname, parsed) = settings(key)
            .parse_hostname(&format!("lobby\0{encoded}\0example"))
            .unwrap();

        assert_eq!(hostname, "lobby\0example");
        let data = parsed.unwrap();
        assert_eq!(data.username, "Player");
        assert!(!data.education);
        assert_eq!(data.xuid, "-1");
    }

    #[test]
    fn rejects_multiple_payloads() {
        let key = [2u8; 16];
        let data = "0\0Player\00\01\0en_US\02\01\0127.0.0.1\0\00\00\0123\0verify";
        let encoded = payload(key, data);
        assert!(settings(key)
            .parse_hostname(&format!("{encoded}\0{encoded}"))
            .is_err());
    }

    #[test]
    fn parses_modern_education_uuid() {
        let data = parse_data(
            "0\0Student\000000000-0000-4000-8000-000000000001\01\0en_US\02\01\0127.0.0.1\0\00\00\0123\0verify\01\0tenant\00",
        )
        .unwrap();

        let (_, uuid) = settings([3u8; 16]).game_profile(&data).unwrap();
        assert_eq!(uuid.most_significant_bits(), Some(EDUCATION_UUID_MSB));
    }

    #[test]
    fn parses_legacy_education_uuid() {
        let data = parse_data(
            "0\0Student\00\01\0en_US\02\01\0127.0.0.1\0\00\00\0123\0verify\01\0tenant\00",
        )
        .unwrap();

        let mut settings = settings([4u8; 16]);
        settings.education_uuid_legacy = true;
        let (_, uuid) = settings.game_profile(&data).unwrap();

        let expected = legacy_education_uuid("tenant", "Student");
        assert_eq!(uuid, expected);
    }

    #[test]
    fn truncates_utf8_usernames() {
        assert_eq!(truncate_utf8("ééé", 3), "é");
        assert_eq!(truncate_utf8("abcdef", 3), "abc");
    }
}
