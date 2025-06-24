use hmac::digest::InvalidLength;
use hmac::{Hmac, Mac};
use minecraft_protocol::prelude::{DecodePacketField, VarInt, VarIntParseError};
use sha2::Sha256;
use subtle::ConstantTimeEq;
use thiserror::Error;

// Type alias for HMAC-SHA256.
type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Error)]
pub enum VelocityKeyIntegrityError {
    #[error("invalid forward version {0}")]
    InvalidForwardingVersion(i32),
    #[error(transparent)]
    InvalidVarInt(#[from] VarIntParseError),
    #[error(transparent)]
    InvalidLength(#[from] InvalidLength),
    #[error("buffer too short to contain signature, received {0} bytes")]
    BufferTooShort(usize),
}

/// Checks the integrity of the forwarded message using an HMAC signature.
///
/// The input `buf` is expected to have the first 32 bytes as the HMAC signature,
/// followed by the payload. The HMAC is computed over the entire payload. After verifying
/// the HMAC, the function reads a varint from the beginning of the payload and checks if it equals 1.
///
/// # Arguments
///
/// * `buf` - A byte slice containing the full message. The first 32 bytes are the HMAC signature.
/// * `secret_key` - A byte slice containing the secret key used for HMAC computation.
///
/// # Returns
///
/// * `Ok(true)` if the HMAC is valid and the version equals 1.
/// * `Ok(false)` if the computed signature does not match the provided signature.
/// * An error if the input buffer is malformed or the version is unsupported.
pub fn check_velocity_key_integrity(
    buf: &[u8],
    secret_key: &[u8],
    index: &mut usize,
) -> Result<bool, VelocityKeyIntegrityError> {
    if buf.len() < 32 {
        return Err(VelocityKeyIntegrityError::BufferTooShort(buf.len()));
    }

    // Extract the signature (first 32 bytes) and the payload.
    let signature = &buf[..32];
    *index += 32;
    let payload = &buf[32..];

    // Compute HMAC-SHA256 over the payload.
    let mut mac = HmacSha256::new_from_slice(secret_key)?;
    mac.update(payload);
    let computed_signature = mac.finalize().into_bytes();

    // Use constant-time equality to compare signatures.
    if signature.ct_eq(&computed_signature).unwrap_u8() != 1 {
        return Ok(false);
    }

    // Read the version from the beginning of the payload.
    let version = VarInt::decode(buf, index)?.value();
    if version != 1 {
        return Err(VelocityKeyIntegrityError::InvalidForwardingVersion(version));
    }

    Ok(true)
}
