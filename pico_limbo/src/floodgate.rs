use aes_gcm::{
    Aes128Gcm, Nonce,
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
const MAX_USERNAME_BYTES: usize = 16;
const MAX_USERNAME_PREFIX_BYTES: usize = 16;
const EDUCATION_UUID_MSB: u64 = 0x0000000100000001;

#[derive(Clone)]
pub struct FloodgateSettings {
    key: [u8; 16],
    education_enabled: bool,
    username_prefix: String,
    education_prefix: String,
    replace_spaces: bool,
    education_uuid_legacy: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FloodgateData {
    pub username: String,
    pub xuid: String,
    pub education: bool,
    pub tenant_id: String,
}

impl Default for FloodgateSettings {
    fn default() -> Self {
        Self {
            key: [0; 16],
            education_enabled: false,
            username_prefix: ".".into(),
            education_prefix: "+".into(),
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
            key: load_key(&config.key_file)?,
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
        let mut clean_parts = Vec::new();
        let mut floodgate_data = None;

        for part in hostname.split('\0') {
            let Some(version) = floodgate_version(part.as_bytes()) else {
                clean_parts.push(part);
                continue;
            };

            if version != i32::from(VERSION) {
                return Err(format!("Unsupported Floodgate data version: {version}"));
            }
            if floodgate_data.is_some() {
                return Err("Multiple Floodgate payloads were provided".into());
            }

            let decrypted = decrypt(&self.key, part)?;
            let data = parse_data(&decrypted)?;

            if data.education && !self.education_enabled {
                return Err(
                    "Education Floodgate data was received but education support is disabled"
                        .into(),
                );
            }

            floodgate_data = Some(data);
        }

        match floodgate_data {
            Some(data) => Ok((clean_parts.join("\0"), Some(data))),
            None => Ok((hostname.to_owned(), None)),
        }
    }

    pub fn game_profile(&self, data: &FloodgateData) -> Result<(String, Uuid), String> {
        let prefix = if data.education {
            &self.education_prefix
        } else {
            &self.username_prefix
        };

        let prefix_bytes = prefix.len();
        let username_budget = MAX_USERNAME_BYTES.saturating_sub(prefix_bytes);
        let username = truncate_utf8(&data.username, username_budget);
        let username = format!("{prefix}{username}");
        let username = if self.replace_spaces {
            username.replace(' ', "_")
        } else {
            username
        };

        let uuid = if data.education {
            if self.education_uuid_legacy {
                legacy_education_uuid(&data.tenant_id, &data.username)
            } else {
                education_uuid(&data.xuid)?
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
            "Floodgate key must contain exactly 16 raw AES-128 bytes, got {} bytes",
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
    if !bytes.starts_with(HEADER) {
        return Err("Invalid Floodgate header".into());
    }

    let payload = &bytes[HEADER.len()..];
    if payload.len() > MAX_PAYLOAD_BYTES {
        return Err("Floodgate payload is too large".into());
    }

    let Some(separator) = payload.iter().position(|byte| *byte == b'!') else {
        return Err("Invalid Floodgate payload".into());
    };

    let iv = base64::engine::general_purpose::STANDARD
        .decode(&payload[..separator])
        .map_err(|_| "Invalid Floodgate IV encoding".to_string())?;
    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(&payload[separator + 1..])
        .map_err(|_| "Invalid Floodgate ciphertext encoding".to_string())?;

    if iv.len() != IV_LENGTH {
        return Err("Invalid Floodgate IV length".into());
    }
    if ciphertext.is_empty() || ciphertext.len() > MAX_CIPHERTEXT_BYTES {
        return Err("Invalid Floodgate ciphertext length".into());
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

    if fields.len() != 12 && fields.len() != 15 {
        return Err(format!(
            "Invalid Floodgate data field count: {}",
            fields.len()
        ));
    }

    if fields.len() == 15 && fields[12] != "1" {
        return Err("Invalid Education Floodgate flag".into());
    }

    if fields[9] != "0" && fields[9] != "1" {
        return Err("Invalid Floodgate proxy flag".into());
    }

    Ok(FloodgateData {
        username: fields[1].to_owned(),
        xuid: fields[2].to_owned(),
        education: fields.len() == 15,
        tenant_id: if fields.len() == 15 {
            fields[13].to_owned()
        } else {
            String::new()
        },
    })
}

fn education_uuid(oid: &str) -> Result<Uuid, String> {
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
        return value.to_owned();
    }

    let mut end = max;
    while end > 0 && !value.is_char_boundary(end) {
        end -= 1;
    }

    value[..end].to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(key: [u8; 16]) -> FloodgateSettings {
        FloodgateSettings {
            key,
            education_enabled: true,
            username_prefix: ".".into(),
            education_prefix: "+".into(),
            replace_spaces: true,
            education_uuid_legacy: false,
        }
    }

    fn encrypted_hostname(key: [u8; 16], data: &str) -> String {
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

    fn standard_data() -> String {
        [
            "0",
            "Player",
            "123456789",
            "1",
            "en_US",
            "0",
            "1",
            "127.0.0.1",
            "",
            "0",
            "123",
            "verify",
        ]
        .join("\0")
    }

    fn education_data() -> String {
        [
            "0",
            "Student",
            "00000000-0000-4000-8000-000000000001",
            "1",
            "en_US",
            "0",
            "1",
            "127.0.0.1",
            "",
            "1",
            "123",
            "verify",
            "1",
            "tenant",
            "0",
        ]
        .join("\0")
    }

    #[test]
    fn decrypts_standard_floodgate_payload() {
        let key = [1u8; 16];
        let encoded = encrypted_hostname(key, &standard_data());

        let (hostname, parsed) = settings(key)
            .parse_hostname(&format!("lobby\0{encoded}\0example"))
            .unwrap();

        assert_eq!(hostname, "lobby\0example");
        let data = parsed.unwrap();
        assert_eq!(
            data,
            FloodgateData {
                username: "Player".into(),
                xuid: "123456789".into(),
                education: false,
                tenant_id: String::new(),
            }
        );
    }

    #[test]
    fn rejects_multiple_payloads() {
        let key = [2u8; 16];
        let encoded = encrypted_hostname(key, &standard_data());
        assert!(
            settings(key)
                .parse_hostname(&format!("{encoded}\0{encoded}"))
                .is_err()
        );
    }

    #[test]
    fn rejects_education_when_disabled() {
        let key = [3u8; 16];
        let encoded = encrypted_hostname(key, &education_data());
        let mut settings = settings(key);
        settings.education_enabled = false;
        assert!(settings.parse_hostname(&encoded).is_err());
    }

    #[test]
    fn builds_standard_game_profile() {
        let settings = settings([4u8; 16]);
        let data = parse_data(&standard_data()).unwrap();
        let (username, uuid) = settings.game_profile(&data).unwrap();

        assert_eq!(username, ".Player");
        assert_eq!(uuid, Uuid::from_u64_pair(0, 123456789));
    }

    #[test]
    fn builds_modern_education_uuid() {
        let settings = settings([5u8; 16]);
        let data = parse_data(&education_data()).unwrap();
        let (_, uuid) = settings.game_profile(&data).unwrap();
        let expected = education_uuid("00000000-0000-4000-8000-000000000001").unwrap();
        assert_eq!(uuid, expected);
    }

    #[test]
    fn builds_legacy_education_uuid() {
        let mut settings = settings([6u8; 16]);
        settings.education_uuid_legacy = true;
        let data = parse_data(&education_data()).unwrap();
        let (_, uuid) = settings.game_profile(&data).unwrap();
        assert_eq!(uuid, legacy_education_uuid("tenant", "Student"));
    }

    #[test]
    fn truncates_utf8_without_splitting() {
        assert_eq!(truncate_utf8("ééé", 3), "é");
        assert_eq!(truncate_utf8("abcdef", 3), "abc");
    }
}
