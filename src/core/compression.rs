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
 *   - Luna-RS Contributors (Rust implementation derived from luna.c)
 */

use flate2::read::DeflateDecoder;
use flate2::write::DeflateEncoder;
use flate2::Compression;
use std::io::{Read, Write};
use thiserror::Error;

/// Errors that can occur during compression/decompression
#[derive(Debug, Error)]
pub enum CompressionError {
    #[error("Compression failed: {0}")]
    CompressionFailed(String),
    #[allow(dead_code)]
    #[error("Decompression failed: {0}")]
    DecompressionFailed(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Compress XML data using deflate compression
///
/// This function compresses XML data using the deflate algorithm (raw deflate without
/// zlib headers) to match the format expected by TI-Nspire calculators.
///
/// Based on luna.c `deflate_compressed_xml()` function (lines 474-497).
///
/// # Arguments
///
/// * `xml_data` - The XML data to compress
///
/// # Returns
///
/// A vector of compressed bytes, or an error if compression fails.
pub fn compress_xml(xml_data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    // Use deflate with -windowBits=-15 (no zlib header), matching luna.c line 484
    let mut encoder = DeflateEncoder::new(Vec::new(), Compression::default());
    
    encoder
        .write_all(xml_data)
        .map_err(|e| CompressionError::CompressionFailed(format!("Failed to write data: {}", e)))?;
    
    encoder
        .finish()
        .map_err(|e| CompressionError::CompressionFailed(format!("Failed to finish compression: {}", e)))
}

/// Decompress XML data using inflate decompression
///
/// This function decompresses data that was compressed with deflate (raw deflate
/// without zlib headers).
///
/// # Arguments
///
/// * `compressed_data` - The compressed data to decompress
///
/// # Returns
///
/// A vector of decompressed bytes, or an error if decompression fails.
#[allow(dead_code)]
pub fn decompress_xml(compressed_data: &[u8]) -> Result<Vec<u8>, CompressionError> {
    let mut decoder = DeflateDecoder::new(compressed_data);
    let mut decompressed = Vec::new();

    decoder
        .read_to_end(&mut decompressed)
        .map_err(|e| CompressionError::DecompressionFailed(format!("Failed to decompress: {}", e)))?;

    Ok(decompressed)
}

/// Compress XML data and return the compressed size along with the data
///
/// This is a helper function that returns both the compressed data and its size,
/// similar to the C implementation.
///
/// # Arguments
///
/// * `xml_data` - The XML data to compress
///
/// # Returns
///
/// A tuple of (compressed_data, size), or an error if compression fails.
#[allow(dead_code)]
pub fn compress_xml_with_size(xml_data: &[u8]) -> Result<(Vec<u8>, usize), CompressionError> {
    let compressed = compress_xml(xml_data)?;
    let size = compressed.len();
    Ok((compressed, size))
}

/// Estimate the maximum size needed for deflate compression buffer
///
/// Based on zlib documentation: compressed_size <= original_size + (original_size * 0.1) + 12
/// This matches the calculation in luna.c line 526.
///
/// # Arguments
///
/// * `original_size` - The size of the data to be compressed
///
/// # Returns
///
/// The estimated maximum size for the compressed data buffer.
#[allow(dead_code)]
pub fn estimate_compressed_size(original_size: usize) -> usize {
    original_size + (original_size / 10) + 12
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compress_decompress_roundtrip() {
        let original = b"Hello, World! This is a test of compression.";
        
        let compressed = compress_xml(original).unwrap();
        // Note: Small data might not compress smaller due to overhead
        assert!(!compressed.is_empty());
        
        let decompressed = decompress_xml(&compressed).unwrap();
        assert_eq!(decompressed, original);
    }

    #[test]
    fn test_compress_xml_data() {
        let xml = b"<test><data>Some XML content</data></test>";
        let compressed = compress_xml(xml).unwrap();
        
        // Compressed data should be non-empty
        assert!(!compressed.is_empty());
        
        // Should be able to decompress back
        let decompressed = decompress_xml(&compressed).unwrap();
        assert_eq!(decompressed, xml);
    }

    #[test]
    fn test_compress_empty_data() {
        let empty: &[u8] = b"";
        let compressed = compress_xml(empty).unwrap();
        let decompressed = decompress_xml(&compressed).unwrap();
        assert_eq!(decompressed, empty);
    }

    #[test]
    fn test_compress_xml_with_size() {
        let data = b"Test data for compression";
        let (compressed, size) = compress_xml_with_size(data).unwrap();
        
        assert_eq!(compressed.len(), size);
        assert!(!compressed.is_empty());
    }

    #[test]
    fn test_estimate_compressed_size() {
        // Test with various sizes
        assert_eq!(estimate_compressed_size(100), 100 + 10 + 12);
        assert_eq!(estimate_compressed_size(1000), 1000 + 100 + 12);
        assert_eq!(estimate_compressed_size(0), 12);
    }

    #[test]
    fn test_compress_large_repeated_data() {
        // Repetitive data should compress well
        let data = vec![b'A'; 1000];
        let compressed = compress_xml(&data).unwrap();
        
        // Should compress significantly
        assert!(compressed.len() < data.len() / 2);
        
        // Should decompress correctly
        let decompressed = decompress_xml(&compressed).unwrap();
        assert_eq!(decompressed, data);
    }

    #[test]
    fn test_decompress_invalid_data() {
        let invalid_data = b"This is not compressed data";
        let result = decompress_xml(invalid_data);
        
        // Should fail gracefully
        assert!(result.is_err());
    }

    #[test]
    fn test_compress_xml_header() {
        // Test with actual XML header like TI uses
        let xml = b"TIXC0100-1.0?><prob xmlns=\"urn:TI.Problem\"><test/></prob>";
        let compressed = compress_xml(xml).unwrap();
        let decompressed = decompress_xml(&compressed).unwrap();
        
        assert_eq!(decompressed, xml);
    }
}
