use aes_gcm::{
    Aes128Gcm, Nonce,
    aead::{Aead, KeyInit},
};
use base64::Engine;
use minecraft_protocol::prelude::Uuid;
use sha2::{Digest, Sha256};
use std::{fs, path::Path};
use thiserror::Error;

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
const EDUCATION_UUID_MSB: u64 = 0x0000_0001_0000_0001;

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

#[derive(Debug, Error)]
pub enum FloodgateError {
    #[error("Failed to read Floodgate key '{path}': {source}")]
    KeyRead {
        path: String,
        source: std::io::Error,
    },
    #[error("Floodgate key must contain exactly 16 raw AES-128 bytes, got {0} bytes")]
    InvalidKeyLength(usize),
    #[error("{0} must be at most 16 bytes")]
    InvalidPrefix(&'static str),
    #[error("{0} must not contain NUL bytes")]
    InvalidPrefixNul(&'static str),
    #[error("Unsupported Floodgate data version: {0}")]
    UnsupportedVersion(i32),
    #[error("Multiple Floodgate payloads were provided")]
    MultiplePayloads,
    #[error("Education Floodgate data was received but education support is disabled")]
    EducationDisabled,
    #[error("Invalid Floodgate header")]
    InvalidHeader,
    #[error("Floodgate payload is too large")]
    PayloadTooLarge,
    #[error("Invalid Floodgate payload")]
    InvalidPayload,
    #[error("Invalid Floodgate IV encoding")]
    InvalidIvEncoding,
    #[error("Invalid Floodgate ciphertext encoding")]
    InvalidCiphertextEncoding,
    #[error("Invalid Floodgate IV length")]
    InvalidIvLength,
    #[error("Invalid Floodgate ciphertext length")]
    InvalidCiphertextLength,
    #[error("Invalid Floodgate AES key")]
    InvalidAesKey,
    #[error("Floodgate authentication failed")]
    AuthenticationFailed,
    #[error("Floodgate data is not valid UTF-8")]
    InvalidUtf8(#[from] std::string::FromUtf8Error),
    #[error("Invalid Floodgate data field count: {0}")]
    InvalidFieldCount(usize),
    #[error("Invalid Floodgate proxy flag")]
    InvalidProxyFlag,
    #[error("Invalid Education Floodgate flag")]
    InvalidEducationFlag,
    #[error("Floodgate xuid is not a valid 64-bit integer")]
    InvalidXuid,
    #[error("EduFloodgate xuid is not a valid Entra OID: {0}")]
    InvalidEducationOid(#[from] uuid::Error),
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
    pub fn from_config(config: &EnabledFloodgateConfig) -> Result<Self, FloodgateError> {
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
    ) -> Result<(String, Option<FloodgateData>), FloodgateError> {
        let mut clean_parts = Vec::new();
        let mut floodgate_data = None;

        for part in hostname.split('\0') {
            let Some(version) = floodgate_version(part.as_bytes()) else {
                clean_parts.push(part);
                continue;
            };

            if version != i32::from(VERSION) {
                return Err(FloodgateError::UnsupportedVersion(version));
            }
            if floodgate_data.is_some() {
                return Err(FloodgateError::MultiplePayloads);
            }

            let decrypted = decrypt(&self.key, part)?;
            let data = parse_data(&decrypted)?;

            if data.education && !self.education_enabled {
                return Err(FloodgateError::EducationDisabled);
            }

            floodgate_data = Some(data);
        }

        floodgate_data.map_or_else(
            || Ok((hostname.to_owned(), None)),
            |data| Ok((clean_parts.join("\0"), Some(data))),
        )
    }

    pub fn game_profile(&self, data: &FloodgateData) -> Result<(String, Uuid), FloodgateError> {
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
                .map_err(|_| FloodgateError::InvalidXuid)?;
            Uuid::from_u64_pair(0, xuid.cast_unsigned())
        };

        Ok((username, uuid))
    }
}

fn load_key(value: &str) -> Result<[u8; 16], FloodgateError> {
    let path = Path::new(value);
    let data = fs::read(path).map_err(|source| FloodgateError::KeyRead {
        path: path.display().to_string(),
        source,
    })?;

    if data.len() != 16 {
        return Err(FloodgateError::InvalidKeyLength(data.len()));
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

fn decrypt(key: &[u8; 16], value: &str) -> Result<String, FloodgateError> {
    let bytes = value.as_bytes();
    if !bytes.starts_with(HEADER) {
        return Err(FloodgateError::InvalidHeader);
    }

    let payload = &bytes[HEADER.len()..];
    if payload.len() > MAX_PAYLOAD_BYTES {
        return Err(FloodgateError::PayloadTooLarge);
    }

    let Some(separator) = payload.iter().position(|byte| *byte == b'!') else {
        return Err(FloodgateError::InvalidPayload);
    };

    let iv = base64::engine::general_purpose::STANDARD
        .decode(&payload[..separator])
        .map_err(|_| FloodgateError::InvalidIvEncoding)?;
    let ciphertext = base64::engine::general_purpose::STANDARD
        .decode(&payload[separator + 1..])
        .map_err(|_| FloodgateError::InvalidCiphertextEncoding)?;

    if iv.len() != IV_LENGTH {
        return Err(FloodgateError::InvalidIvLength);
    }
    if ciphertext.is_empty() || ciphertext.len() > MAX_CIPHERTEXT_BYTES {
        return Err(FloodgateError::InvalidCiphertextLength);
    }

    let cipher = Aes128Gcm::new_from_slice(key).map_err(|_| FloodgateError::InvalidAesKey)?;
    let nonce = Nonce::try_from(&iv[..]).map_err(|_| FloodgateError::InvalidIvLength)?;
    let plaintext = cipher
        .decrypt(&nonce, ciphertext.as_ref())
        .map_err(|_| FloodgateError::AuthenticationFailed)?;

    String::from_utf8(plaintext).map_err(FloodgateError::from)
}

fn parse_data(data: &str) -> Result<FloodgateData, FloodgateError> {
    let fields: Vec<&str> = data.split('\0').collect();

    if fields.len() != 12 && fields.len() != 15 {
        return Err(FloodgateError::InvalidFieldCount(fields.len()));
    }

    if fields[9] != "0" && fields[9] != "1" {
        return Err(FloodgateError::InvalidProxyFlag);
    }

    let education = if fields.len() == 15 {
        match fields[12] {
            "0" => false,
            "1" => true,
            _ => return Err(FloodgateError::InvalidEducationFlag),
        }
    } else {
        false
    };

    Ok(FloodgateData {
        username: fields[1].to_owned(),
        xuid: fields[2].to_owned(),
        education,
        tenant_id: if education {
            fields[13].to_owned()
        } else {
            String::new()
        },
    })
}

fn education_uuid(oid: &str) -> Result<Uuid, FloodgateError> {
    let parsed = uuid::Uuid::parse_str(oid)?;
    let value = parsed.as_u128();
    let msb = (value >> 64) as u64;
    let lsb = u64::try_from(value & u128::from(u64::MAX)).unwrap_or_default();

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

fn validate_prefix(prefix: &str, name: &'static str) -> Result<(), FloodgateError> {
    if prefix.len() > MAX_USERNAME_PREFIX_BYTES {
        return Err(FloodgateError::InvalidPrefix(name));
    }
    if prefix.contains('\0') {
        return Err(FloodgateError::InvalidPrefixNul(name));
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
        let nonce = Nonce::try_from(&iv[..]).expect("valid test nonce");
        let ciphertext = cipher.encrypt(&nonce, data.as_bytes()).unwrap();

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

    fn bedrock_data_with_education_fields() -> String {
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
            "0",
            "",
            "-1",
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
    fn accepts_non_education_15_field_payload_when_education_disabled() {
        let key = [7u8; 16];
        let encoded = encrypted_hostname(key, &bedrock_data_with_education_fields());
        let mut settings = settings(key);
        settings.education_enabled = false;

        let (_, parsed) = settings.parse_hostname(&encoded).unwrap();
        let data = parsed.unwrap();

        assert_eq!(data.username, "Player");
        assert_eq!(data.xuid, "123456789");
        assert!(!data.education);
        assert!(data.tenant_id.is_empty());

        let (username, uuid) = settings.game_profile(&data).unwrap();
        assert_eq!(username, ".Player");
        assert_eq!(uuid, Uuid::from_u64_pair(0, 123_456_789));
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
        assert_eq!(uuid, Uuid::from_u64_pair(0, 123_456_789));
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
