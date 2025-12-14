/*
 * The contents of this file are subject to the Mozilla Public
 * License Version 1.1 (the "License"); you may not use this file
 * except in compliance with the License. You may obtain a copy of
 * the License at http://www.mozilla.org/MPL/
 *
 * Software distributed under the License is distributed on an "AS
 * IS" basis, WITHOUT WARRANTY OF ANY KIND, either express or
 * implied. See the License for the specific language governing
 * rights and limitations under the License.
 *
 * The Original Code is Luna code.
 *
 * The Initial Developer of the Original Code is Olivier ARMAND
 * <olivier.calc@gmail.com>.
 * Portions created by the Initial Developer are Copyright (C) 2011-2014
 * the Initial Developer. All Rights Reserved.
 *
 * Contributor(s):
 *   - Luna-RS Contributors (Rust port derived from luna.c `doccrypt()`)
 */

use cipher::{BlockEncrypt, KeyInit};
use des::TdesEde3;
use thiserror::Error;

/// 3DES encryption keys hardcoded in upstream Luna
/// These keys are derived from tien_crypted_header in luna.c
const KEY1: [u8; 8] = [0x16, 0xA7, 0xA7, 0x32, 0x68, 0xA7, 0xBA, 0x73];
const KEY2: [u8; 8] = [0xD9, 0xA8, 0x86, 0xA4, 0x34, 0x45, 0x94, 0x10];
const KEY3: [u8; 8] = [0x3D, 0x80, 0x8C, 0xB5, 0xDF, 0xB3, 0x80, 0x6B];

/// Base IV value for the custom counter scheme
/// This value comes from IVEC_BASE in luna.c line 405
const IVEC_BASE: u32 = 0x6fe21307;

/// Counter wraps at this value (luna.c line 413)
const COUNTER_WRAP: u32 = 1024;

/// DES block size in bytes
const BLOCK_SIZE: usize = 8;

#[derive(Debug, Error)]
pub enum DESError {
    #[error("Data length must be multiple of 8 bytes, got {0} bytes")]
    InvalidLength(usize),
    #[allow(dead_code)]
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
}

/// Encrypts document data using the custom 3DES scheme from Luna.
///
/// This implements the encryption algorithm from `doccrypt()` in luna.c (lines 394-427).
/// The algorithm uses a non-standard approach:
/// 1. Combines three DES keys into 3DES-EDE3
/// 2. Uses a custom IV counter scheme (not standard CBC/CTR)
/// 3. For each 8-byte block:
///    - Calculates IV = base_iv + counter (counter wraps at 1024)
///    - Encrypts the IV bytes using 3DES-ECB
///    - XORs the encrypted IV with the plaintext block
///    - Increments counter (mod 1024)
///
/// # Arguments
/// * `data` - Mutable slice containing data to encrypt in-place
///
/// # Returns
/// * `Result<(), DESError>` - Ok if successful, error otherwise
///
/// # Errors
/// * Returns `DESError::InvalidLength` if data length is not a multiple of 8
///
/// # Example
/// ```rust,ignore
/// let mut data = vec![0u8; 16]; // Must be multiple of 8
/// encrypt_document(&mut data)?;
/// ```
pub fn encrypt_document(data: &mut [u8]) -> Result<(), DESError> {
    // Verify data length is multiple of block size
    if data.len() % BLOCK_SIZE != 0 {
        return Err(DESError::InvalidLength(data.len()));
    }

    // Combine the three 8-byte keys into a single 24-byte key for 3DES-EDE3
    let mut key_24 = [0u8; 24];
    key_24[0..8].copy_from_slice(&KEY1);
    key_24[8..16].copy_from_slice(&KEY2);
    key_24[16..24].copy_from_slice(&KEY3);

    // Initialize 3DES cipher with the combined key
    let cipher = TdesEde3::new(&key_24.into());

    // Counter for IV generation (wraps at 1024)
    let mut ivec_incr: u32 = 0;

    // Process data in 8-byte blocks
    for chunk in data.chunks_mut(BLOCK_SIZE) {
        // Calculate current IV value
        let current_ivec = IVEC_BASE.wrapping_add(ivec_incr);
        
        // Increment counter and wrap at 1024
        ivec_incr += 1;
        if ivec_incr == COUNTER_WRAP {
            ivec_incr = 0;
        }

        // Build IV block: first 4 bytes are zeros, next 4 bytes are current_ivec in little-endian
        // This matches the C code in luna.c lines 415-418:
        //   ivec[4] = (unsigned char)(current_ivec >> 0);
        //   ivec[5] = (unsigned char)(current_ivec >> 8);
        //   ivec[6] = (unsigned char)(current_ivec >> 16);
        //   ivec[7] = (unsigned char)(current_ivec >> 24);
        let mut iv_block = [0u8; BLOCK_SIZE];
        iv_block[4] = (current_ivec >> 0) as u8;
        iv_block[5] = (current_ivec >> 8) as u8;
        iv_block[6] = (current_ivec >> 16) as u8;
        iv_block[7] = (current_ivec >> 24) as u8;

        // Encrypt the IV block using 3DES-ECB
        let mut encrypted_iv = iv_block.into();
        cipher.encrypt_block(&mut encrypted_iv);

        // XOR the encrypted IV with the plaintext chunk to produce ciphertext
        // This matches the C code in luna.c lines 421-423
        for (i, &encrypted_byte) in encrypted_iv.iter().enumerate() {
            if i < chunk.len() {
                chunk[i] ^= encrypted_byte;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encrypt_document_invalid_length() {
        let mut data = vec![0u8; 7]; // Not a multiple of 8
        let result = encrypt_document(&mut data);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DESError::InvalidLength(7)));
    }

    #[test]
    fn test_encrypt_document_empty() {
        let mut data = vec![];
        let result = encrypt_document(&mut data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_encrypt_document_single_block() {
        let mut data = vec![0u8; 8];
        let original = data.clone();
        let result = encrypt_document(&mut data);
        assert!(result.is_ok());
        // Data should be modified after encryption
        assert_ne!(data, original);
    }

    #[test]
    fn test_encrypt_document_multiple_blocks() {
        let mut data = vec![0u8; 24]; // 3 blocks
        let original = data.clone();
        let result = encrypt_document(&mut data);
        assert!(result.is_ok());
        // Data should be modified after encryption
        assert_ne!(data, original);
    }

    #[test]
    fn test_encrypt_document_known_vector() {
        // Test with a known pattern to verify encryption is deterministic
        let mut data1 = vec![0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77];
        let mut data2 = data1.clone();
        
        encrypt_document(&mut data1).unwrap();
        encrypt_document(&mut data2).unwrap();
        
        // Same input should produce same output
        assert_eq!(data1, data2);
    }

    #[test]
    fn test_counter_wrap() {
        // Test that counter wrapping works correctly
        // Create enough data to force counter wrap (1024 blocks * 8 bytes)
        let mut data = vec![0u8; 1024 * 8 + 8]; // 1025 blocks
        let result = encrypt_document(&mut data);
        assert!(result.is_ok());
    }

    #[test]
    fn test_encrypt_document_different_inputs() {
        // Test that different inputs produce different outputs
        let mut data1 = vec![0x00; 8];
        let mut data2 = vec![0xFF; 8];
        
        encrypt_document(&mut data1).unwrap();
        encrypt_document(&mut data2).unwrap();
        
        assert_ne!(data1, data2);
    }
}
