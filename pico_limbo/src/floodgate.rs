use aes_gcm::{
    Aes128Gcm,
    aead::{Aead, KeyInit},
};
use base64::Engine;
use minecraft_protocol::prelude::Uuid;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

const HEADER: &[u8] = b"^Floodgate^>";
const IV_LENGTH: usize = 12;
const EDUCATION_UUID_MSB: u64 = 0x0000000100000001;

#[derive(Clone)]
pub struct FloodgateConfig {
    pub enabled: bool,
    pub education_enabled: bool,
    pub key: Option<[u8; 16]>,
    pub username_prefix: String,
    pub education_prefix: String,
    pub replace_spaces: bool,
    pub education_uuid_legacy: bool,
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

impl Default for FloodgateConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            education_enabled: false,
            key: None,
            username_prefix: ".".to_string(),
            education_prefix: "+".to_string(),
            replace_spaces: true,
            education_uuid_legacy: false,
        }
    }
}

impl FloodgateConfig {
    pub fn from_settings(
        enabled: bool,
        education_enabled: bool,
        key_value: &str,
        username_prefix: String,
        education_prefix: String,
        replace_spaces: bool,
        education_uuid_legacy: bool,
    ) -> Result<Self, String> {
        let key = if enabled || education_enabled {
            Some(load_key(key_value)?)
        } else {
            None
        };

        Ok(Self {
            enabled,
            education_enabled,
            key,
            username_prefix,
            education_prefix,
            replace_spaces,
            education_uuid_legacy,
        })
    }

    pub fn parse_hostname(&self, hostname: &str) -> Result<(String, Option<FloodgateData>), String> {
        let Some(index) = hostname.find('\0') else {
            return Ok((hostname.to_string(), None));
        };

        if !(self.enabled || self.education_enabled) {
            return Ok((hostname.to_string(), None));
        }

        let base = hostname[..index].to_string();
        let encoded = &hostname[index + 1..];
        let key = self.key.as_ref().ok_or_else(|| "Floodgate key is not configured".to_string())?;
        let decrypted = decrypt(key, encoded)?;
        let data = parse_data(&decrypted)?;

        if data.education {
            if !self.education_enabled {
                return Err("EduFloodgate data received but edufloodgate is disabled".to_string());
            }
        } else if !self.enabled {
            return Err("Floodgate data received but floodgate is disabled".to_string());
        }

        Ok((base, Some(data)))
    }

    pub fn game_profile(&self, data: &FloodgateData) -> Result<(String, Uuid), String> {
        let username = if data.education {
            let prefix = &self.education_prefix;
            let max = 16usize.saturating_sub(prefix.len());
            format!("{}{}", prefix, truncate_utf8(&data.username, max))
        } else {
            let prefix = &self.username_prefix;
            let max = 16usize.saturating_sub(prefix.len());
            format!("{}{}", prefix, truncate_utf8(&data.username, max))
        };

        let username = if self.replace_spaces {
            username.replace(' ', "_")
        } else {
            username
        };

        let uuid = if data.education {
            if self.education_uuid_legacy {
                let mut digest = Sha256::new();
                digest.update(format!("{}:{}", data.tenant_id, data.username).as_bytes());
                let hash = digest.finalize();
                let mut lsb = 0u64;
                for byte in hash.iter().take(8) {
                    lsb = (lsb << 8) | u64::from(*byte);
                }
                Uuid::from_u64_pair(EDUCATION_UUID_MSB, lsb)
            } else {
                let parsed = uuid::Uuid::parse_str(&data.xuid)
                    .map_err(|_| "EduFloodgate xuid is not a valid Entra OID".to_string())?;
                let msb = parsed.as_u128() >> 64;
                let lsb = parsed.as_u128() as u64;
                let upper = ((msb >> 16) << 12) | (msb & 0xFFF);
                let lower = (lsb << 2) >> 60;
                Uuid::from_u64_pair(EDUCATION_UUID_MSB, (upper << 4) | lower)
            }
        } else {
            let xuid = data.xuid.parse::<u64>()
                .map_err(|_| "Floodgate xuid is not a valid unsigned 64-bit integer".to_string())?;
            Uuid::from_u64_pair(0, xuid)
        };

        Ok((username, uuid))
    }
}

fn load_key(value: &str) -> Result<[u8; 16], String> {
    let path = Path::new(value);
    let data = if path.is_file() {
        fs::read(path).map_err(|e| format!("Failed to read Floodgate key '{}': {e}", value))?
    } else if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(value) {
        decoded
    } else {
        value.as_bytes().to_vec()
    };

    if data.len() != 16 {
        return Err(format!("Floodgate AES key must be exactly 16 bytes, got {}", data.len()));
    }

    let mut key = [0u8; 16];
    key.copy_from_slice(&data);
    Ok(key)
}

fn decrypt(key: &[u8; 16], value: &str) -> Result<String, String> {
    let bytes = value.as_bytes();
    if bytes.len() <= HEADER.len() || !bytes.starts_with(HEADER) {
        return Err("Invalid Floodgate header".to_string());
    }

    let payload = &bytes[HEADER.len()..];
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

    let cipher = Aes128Gcm::new_from_slice(key)
        .map_err(|_| "Invalid Floodgate AES key".to_string())?;
    let nonce = aes_gcm::Nonce::from_slice(&iv);
    let plaintext = cipher
        .decrypt(nonce, ciphertext.as_ref())
        .map_err(|_| "Floodgate authentication failed".to_string())?;

    String::from_utf8(plaintext).map_err(|_| "Floodgate data is not valid UTF-8".to_string())
}

fn parse_data(data: &str) -> Result<FloodgateData, String> {
    let fields: Vec<&str> = data.split('\0').collect();

    if fields.len() != 12 && fields.len() != 15 {
        return Err(format!("Invalid Floodgate data field count: {}", fields.len()));
    }

    let education = fields.len() == 15 && fields[12] == "1";
    let tenant_id = if fields.len() == 15 { fields[13].to_string() } else { String::new() };
    let ad_role = if fields.len() == 15 {
        fields[14].parse::<i32>().map_err(|_| "Invalid Floodgate ad role".to_string())?
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
