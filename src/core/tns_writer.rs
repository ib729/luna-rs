// MIT License - New code for Luna-RS
// See LICENSE.MIT for full license text

//! TI-Nspire TNS file writer
//!
//! This module creates TNS files with the proper TI-specific format.
//! TNS files are modified ZIP archives with custom magic bytes and end markers.

use std::io::{self, Write, Cursor};
use std::path::Path;

/// TI-Nspire specific magic bytes for first file entry
/// Bytes: 2A 54 49 4D 4C 50 = "*TIMLP"
const TI_HEADER_MAGIC: &[u8] = b"*TIMLP";

/// TI-Nspire version string (e.g., "0500" for version 5.0)
const TI_VERSION_DEFAULT: &[u8] = b"0500";

/// TI-Nspire version for documents with bitmaps
const TI_VERSION_BITMAP: &[u8] = b"0700";

/// Standard ZIP local file header signature (used for 2nd+ files)
const STD_LOCAL_HEADER_SIG: &[u8] = &[0x50, 0x4B, 0x03, 0x04];

/// ZIP central directory header signature
const CENTRAL_DIR_SIG: &[u8] = &[0x50, 0x4B, 0x01, 0x02];

/// TI-Nspire end of central directory signature
/// Standard ZIP uses 0x06054b50 ("PK\x05\x06")
/// TI uses 0x44504954 ("TIPD")
const TI_END_SIG: &[u8] = b"TIPD";

/// TI encrypted compression method
const TI_ENCRYPTED_METHOD: u16 = 0x0D;

/// Standard deflate compression method
const DEFLATE_METHOD: u16 = 0x08;

/// Version needed to extract
const VERSION_NEEDED: u16 = 20;

/// Version made by (MS-DOS)
const VERSION_MADE_BY: u16 = 20;

/// File entry for the TNS archive
pub struct TnsFileEntry {
    pub filename: String,
    pub data: Vec<u8>,
    pub method: u16,
    /// Original uncompressed size (for deflated files)
    pub uncompressed_size: Option<u32>,
    /// CRC32 of original uncompressed data (for deflated files)
    pub crc32: Option<u32>,
}

impl TnsFileEntry {
    pub fn new_ti_encrypted(filename: &str, data: Vec<u8>) -> Self {
        Self {
            filename: filename.to_string(),
            data,
            method: TI_ENCRYPTED_METHOD,
            uncompressed_size: None,
            crc32: None,
        }
    }

    /// Create a deflated entry with pre-compressed data
    pub fn new_deflated(filename: &str, compressed_data: Vec<u8>, original_size: u32, crc: u32) -> Self {
        Self {
            filename: filename.to_string(),
            data: compressed_data,
            method: DEFLATE_METHOD,
            uncompressed_size: Some(original_size),
            crc32: Some(crc),
        }
    }
}

/// Information about a written file entry (for central directory)
struct WrittenEntry {
    filename: String,
    method: u16,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    local_header_offset: u32,
}

/// Write a TNS file with the given entries
///
/// The first file gets the TI-specific header (*TIMLP + version),
/// subsequent files get standard PK signatures.
/// The end of central directory uses TIPD instead of PK\x05\x06.
pub fn write_tns_file(
    output_path: &Path,
    entries: Vec<TnsFileEntry>,
    has_bitmap: bool,
) -> io::Result<()> {
    let mut buffer = Cursor::new(Vec::new());
    let mut written_entries: Vec<WrittenEntry> = Vec::new();

    let version = if has_bitmap { TI_VERSION_BITMAP } else { TI_VERSION_DEFAULT };

    for (i, entry) in entries.iter().enumerate() {
        let local_header_offset = buffer.position() as u32;

        // Use provided CRC or compute from data
        // For TI encrypted files, CRC is of the encrypted data
        // For deflated files, CRC should be of original uncompressed data (provided by caller)
        let crc = entry.crc32.unwrap_or_else(|| crc32fast::hash(&entry.data));
        let compressed_size = entry.data.len() as u32;
        // For TI encrypted files, compressed = uncompressed (data is already processed)
        // For deflated files, use the provided uncompressed size
        let uncompressed_size = entry.uncompressed_size.unwrap_or(compressed_size);

        // Write local file header
        if i == 0 {
            // First entry: TI-specific magic
            write_ti_local_header(&mut buffer, &entry.filename, entry.method, crc, compressed_size, uncompressed_size, version)?;
        } else {
            // Subsequent entries: standard ZIP signature
            write_std_local_header(&mut buffer, &entry.filename, entry.method, crc, compressed_size, uncompressed_size)?;
        }

        // Write file data
        buffer.write_all(&entry.data)?;

        written_entries.push(WrittenEntry {
            filename: entry.filename.clone(),
            method: entry.method,
            crc32: crc,
            compressed_size,
            uncompressed_size,
            local_header_offset,
        });
    }

    // Record start of central directory
    let central_dir_offset = buffer.position() as u32;

    // Write central directory entries
    for entry in &written_entries {
        write_central_dir_entry(&mut buffer, entry)?;
    }

    // Calculate central directory size
    let central_dir_size = buffer.position() as u32 - central_dir_offset;

    // Write end of central directory (TI-specific)
    write_ti_end_of_central_dir(&mut buffer, written_entries.len() as u16, central_dir_size, central_dir_offset)?;

    // Write to file
    std::fs::write(output_path, buffer.into_inner())?;

    Ok(())
}

/// Write TI-specific local file header (for first file)
///
/// Format:
/// - 6 bytes: "*TIMLP"
/// - 4 bytes: version string (e.g., "0500")
/// - 2 bytes: version needed to extract
/// - 2 bytes: general purpose flags
/// - 2 bytes: compression method
/// - 4 bytes: DOS date/time
/// - 4 bytes: CRC-32
/// - 4 bytes: compressed size
/// - 4 bytes: uncompressed size
/// - 2 bytes: filename length
/// - 2 bytes: extra field length
/// - n bytes: filename
fn write_ti_local_header<W: Write>(
    writer: &mut W,
    filename: &str,
    method: u16,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
    version: &[u8],
) -> io::Result<()> {
    // TI magic + version (10 bytes total)
    writer.write_all(TI_HEADER_MAGIC)?;
    writer.write_all(version)?;

    // Version needed to extract (2 bytes)
    writer.write_all(&VERSION_NEEDED.to_le_bytes())?;

    // General purpose flags (2 bytes)
    writer.write_all(&0u16.to_le_bytes())?;

    // Compression method (2 bytes)
    writer.write_all(&method.to_le_bytes())?;

    // DOS date/time (4 bytes) - use fixed value like C version
    writer.write_all(&0x00200000u32.to_le_bytes())?;

    // CRC-32 (4 bytes)
    writer.write_all(&crc32.to_le_bytes())?;

    // Compressed size (4 bytes)
    writer.write_all(&compressed_size.to_le_bytes())?;

    // Uncompressed size (4 bytes)
    writer.write_all(&uncompressed_size.to_le_bytes())?;

    // Filename length (2 bytes)
    let filename_len = filename.len() as u16;
    writer.write_all(&filename_len.to_le_bytes())?;

    // Extra field length (2 bytes)
    writer.write_all(&0u16.to_le_bytes())?;

    // Filename
    writer.write_all(filename.as_bytes())?;

    Ok(())
}

/// Write standard ZIP local file header (for subsequent files)
fn write_std_local_header<W: Write>(
    writer: &mut W,
    filename: &str,
    method: u16,
    crc32: u32,
    compressed_size: u32,
    uncompressed_size: u32,
) -> io::Result<()> {
    // Standard PK signature (4 bytes)
    writer.write_all(STD_LOCAL_HEADER_SIG)?;

    // Version needed to extract (2 bytes)
    writer.write_all(&VERSION_NEEDED.to_le_bytes())?;

    // General purpose flags (2 bytes)
    writer.write_all(&0u16.to_le_bytes())?;

    // Compression method (2 bytes)
    writer.write_all(&method.to_le_bytes())?;

    // DOS date/time (4 bytes)
    writer.write_all(&0x00200000u32.to_le_bytes())?;

    // CRC-32 (4 bytes)
    writer.write_all(&crc32.to_le_bytes())?;

    // Compressed size (4 bytes)
    writer.write_all(&compressed_size.to_le_bytes())?;

    // Uncompressed size (4 bytes)
    writer.write_all(&uncompressed_size.to_le_bytes())?;

    // Filename length (2 bytes)
    let filename_len = filename.len() as u16;
    writer.write_all(&filename_len.to_le_bytes())?;

    // Extra field length (2 bytes)
    writer.write_all(&0u16.to_le_bytes())?;

    // Filename
    writer.write_all(filename.as_bytes())?;

    Ok(())
}

/// Write central directory file header
fn write_central_dir_entry<W: Write>(
    writer: &mut W,
    entry: &WrittenEntry,
) -> io::Result<()> {
    // Central directory signature (4 bytes)
    writer.write_all(CENTRAL_DIR_SIG)?;

    // Version made by (2 bytes)
    writer.write_all(&VERSION_MADE_BY.to_le_bytes())?;

    // Version needed to extract (2 bytes)
    writer.write_all(&VERSION_NEEDED.to_le_bytes())?;

    // General purpose flags (2 bytes)
    writer.write_all(&0u16.to_le_bytes())?;

    // Compression method (2 bytes)
    writer.write_all(&entry.method.to_le_bytes())?;

    // DOS date/time (4 bytes)
    writer.write_all(&0x00200000u32.to_le_bytes())?;

    // CRC-32 (4 bytes)
    writer.write_all(&entry.crc32.to_le_bytes())?;

    // Compressed size (4 bytes)
    writer.write_all(&entry.compressed_size.to_le_bytes())?;

    // Uncompressed size (4 bytes)
    writer.write_all(&entry.uncompressed_size.to_le_bytes())?;

    // Filename length (2 bytes)
    let filename_len = entry.filename.len() as u16;
    writer.write_all(&filename_len.to_le_bytes())?;

    // Extra field length (2 bytes)
    writer.write_all(&0u16.to_le_bytes())?;

    // File comment length (2 bytes)
    writer.write_all(&0u16.to_le_bytes())?;

    // Disk number start (2 bytes)
    writer.write_all(&0u16.to_le_bytes())?;

    // Internal file attributes (2 bytes)
    writer.write_all(&0u16.to_le_bytes())?;

    // External file attributes (4 bytes)
    writer.write_all(&0u32.to_le_bytes())?;

    // Relative offset of local header (4 bytes)
    writer.write_all(&entry.local_header_offset.to_le_bytes())?;

    // Filename
    writer.write_all(entry.filename.as_bytes())?;

    Ok(())
}

/// Write TI-specific end of central directory record
///
/// Uses "TIPD" signature instead of standard "PK\x05\x06"
fn write_ti_end_of_central_dir<W: Write>(
    writer: &mut W,
    num_entries: u16,
    central_dir_size: u32,
    central_dir_offset: u32,
) -> io::Result<()> {
    // TI end signature (4 bytes) - "TIPD" instead of PK\x05\x06
    writer.write_all(TI_END_SIG)?;

    // Number of this disk (2 bytes)
    writer.write_all(&0u16.to_le_bytes())?;

    // Disk where central directory starts (2 bytes)
    writer.write_all(&0u16.to_le_bytes())?;

    // Number of central directory records on this disk (2 bytes)
    writer.write_all(&num_entries.to_le_bytes())?;

    // Total number of central directory records (2 bytes)
    writer.write_all(&num_entries.to_le_bytes())?;

    // Size of central directory (4 bytes)
    writer.write_all(&central_dir_size.to_le_bytes())?;

    // Offset of start of central directory (4 bytes)
    writer.write_all(&central_dir_offset.to_le_bytes())?;

    // Comment length (2 bytes)
    writer.write_all(&0u16.to_le_bytes())?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ti_header_magic() {
        let mut buffer = Cursor::new(Vec::new());
        write_ti_local_header(
            &mut buffer,
            "test.xml",
            TI_ENCRYPTED_METHOD,
            0x12345678,
            100,
            100,
            TI_VERSION_DEFAULT,
        ).unwrap();

        let bytes = buffer.into_inner();

        // Check TI magic: "*TIMLP"
        assert_eq!(&bytes[0..6], b"*TIMLP");
        // Check version: "0500"
        assert_eq!(&bytes[6..10], b"0500");
    }

    #[test]
    fn test_std_header_magic() {
        let mut buffer = Cursor::new(Vec::new());
        write_std_local_header(
            &mut buffer,
            "test.xml",
            TI_ENCRYPTED_METHOD,
            0x12345678,
            100,
            100,
        ).unwrap();

        let bytes = buffer.into_inner();

        // Check standard ZIP signature: PK\x03\x04
        assert_eq!(&bytes[0..4], &[0x50, 0x4B, 0x03, 0x04]);
    }

    #[test]
    fn test_ti_end_signature() {
        let mut buffer = Cursor::new(Vec::new());
        write_ti_end_of_central_dir(&mut buffer, 2, 100, 500).unwrap();

        let bytes = buffer.into_inner();

        // Check TI end signature: "TIPD"
        assert_eq!(&bytes[0..4], b"TIPD");
    }
}
