use hmac::digest::InvalidLength;
use hmac::{Hmac, Mac};
use minecraft_protocol::prelude::{BinaryReader, BinaryReaderError, VarInt};
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
    InvalidLength(#[from] InvalidLength),
    #[error("buffer too short to contain signature, received {0} bytes")]
    BufferTooShort(usize),
    #[error(transparent)]
    BinaryReader(#[from] BinaryReaderError),
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
    reader: &mut BinaryReader,
    secret_key: &[u8],
) -> Result<bool, VelocityKeyIntegrityError> {
    if reader.remaining() < 32 {
        return Err(VelocityKeyIntegrityError::BufferTooShort(
            reader.remaining(),
        ));
    }

    // Extract the signature (first 32 bytes) and the payload.
    let mut signature = vec![0u8; 32];
    reader.read_bytes(&mut signature)?;

    let mut payload = vec![0u8; reader.remaining()];
    reader.read_bytes(&mut payload)?;

    // Compute HMAC-SHA256 over the payload.
    let mut mac = HmacSha256::new_from_slice(secret_key)?;
    mac.update(&payload);
    let computed_signature = mac.finalize().into_bytes();

    // Use constant-time equality to compare signatures.
    if signature.ct_eq(&computed_signature).unwrap_u8() != 1 {
        return Ok(false);
    }

    // Read the version from the beginning of the payload.
    let mut payload_reader = BinaryReader::new(&payload);
    let version = payload_reader.read::<VarInt>()?.inner();
    if version != 1 {
        return Err(VelocityKeyIntegrityError::InvalidForwardingVersion(version));
    }

    Ok(true)
}
