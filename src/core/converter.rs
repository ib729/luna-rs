// MIT License - New code for Luna-RS
// See LICENSE.MIT for full license text

use std::path::Path;

use super::compression;
use super::des;
use super::xml::{self, ScriptType};
use super::tns_writer::{self, TnsFileEntry};

/// Errors that can occur during conversion
#[derive(Debug, thiserror::Error)]
pub enum ConversionError {
    #[error("XML error: {0}")]
    Xml(#[from] xml::XMLError),

    #[error("Compression error: {0}")]
    Compression(#[from] compression::CompressionError),

    #[error("DES encryption error: {0}")]
    Des(#[from] des::DESError),

    #[error("ZIP error: {0}")]
    Zip(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[allow(dead_code)]
    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Main converter that orchestrates the conversion process
pub struct Converter;

impl Converter {
    /// Create a new converter instance
    pub fn new() -> Self {
        Self
    }

    /// Convert a script file to .tns format
    ///
    /// # Arguments
    /// * `input_path` - Path to the input script file
    /// * `output_path` - Path where the .tns file will be written
    /// * `script_type` - Type of script (Lua or Python)
    /// * `_encrypt` - Whether to encrypt (currently always true for .tns files)
    #[allow(dead_code)]
    pub fn convert_to_tns(
        &self,
        input_path: &Path,
        output_path: &Path,
        script_type: ScriptType,
        _encrypt: bool,
    ) -> Result<(), ConversionError> {
        // Read the script content
        let script_content = std::fs::read_to_string(input_path)
            .map_err(|e| ConversionError::Io(e))?;
        
        // Convert based on script type
        match script_type {
            ScriptType::Lua => {
                self.convert_lua_to_tns(&script_content, output_path, "")
            }
            ScriptType::Python => {
                let filename = input_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| ConversionError::InvalidInput("Invalid filename".to_string()))?;
                self.convert_python_to_tns(&script_content, filename, output_path, "")
            }
        }
    }

    /// Convert a Lua script to .tns format
    ///
    /// # Arguments
    /// * `lua_script` - The Lua script content
    /// * `output_path` - Path where the .tns file will be written
    /// * `document_name` - Name for the document (empty for default)
    pub fn convert_lua_to_tns(
        &self,
        lua_script: &str,
        output_path: &Path,
        document_name: &str,
    ) -> Result<(), ConversionError> {
        // 1. Wrap Lua script in XML
        let script_xml = xml::wrap_lua_script(lua_script, document_name)?;
        
        // 2. Compress the XML
        let compressed = compression::compress_xml(&script_xml)?;
        
        // 3. Pad to 8-byte boundary for DES
        let mut padded = pad_to_8_bytes(compressed);
        
        // 4. Encrypt with DES
        des::encrypt_document(&mut padded)?;
        
        // 5. Add TI encrypted header
        let mut problem_data = Vec::new();
        problem_data.extend_from_slice(xml::get_ti_encrypted_header());
        problem_data.extend_from_slice(&padded);
        
        // 6. Get default Document.xml
        let document_xml = xml::create_default_document_xml();
        
        // 7. Create the .tns archive
        create_tns_archive(
            output_path,
            document_xml,
            &problem_data,
            "Problem1.xml",
        )?;
        
        Ok(())
    }

    /// Convert a Python script to .tns format
    ///
    /// # Arguments
    /// * `python_script` - The Python script content
    /// * `python_filename` - The filename of the Python script
    /// * `output_path` - Path where the .tns file will be written
    /// * `document_name` - Name for the document (empty for default)
    pub fn convert_python_to_tns(
        &self,
        python_script: &str,
        python_filename: &str,
        output_path: &Path,
        document_name: &str,
    ) -> Result<(), ConversionError> {
        // 1. Create Python XML wrapper
        let python_xml = xml::wrap_python_script(python_filename, document_name)?;
        
        // 2. Compress the XML
        let compressed = compression::compress_xml(&python_xml)?;
        
        // 3. Pad to 8-byte boundary for DES
        let mut padded = pad_to_8_bytes(compressed);
        
        // 4. Encrypt with DES
        des::encrypt_document(&mut padded)?;
        
        // 5. Add TI encrypted header
        let mut problem_data = Vec::new();
        problem_data.extend_from_slice(xml::get_ti_encrypted_header());
        problem_data.extend_from_slice(&padded);
        
        // 6. Get default Document.xml
        let document_xml = xml::create_default_document_xml();
        
        // 7. Create the .tns archive with both Problem1.xml and the Python file
        create_tns_archive_with_python(
            output_path,
            document_xml,
            &problem_data,
            python_filename,
            python_script.as_bytes(),
        )?;
        
        Ok(())
    }

    /// Convert plain text to .tns format
    ///
    /// Since TI-Nspire doesn't have a native "plain text note" format,
    /// this converts the text to a Lua script that displays the text on screen.
    /// The resulting .tns file will show the text when opened on the calculator.
    ///
    /// # Arguments
    /// * `text` - The plain text content
    /// * `output_path` - Path where the .tns file will be written
    /// * `document_name` - Name for the document (empty for default)
    pub fn convert_text_to_tns(
        &self,
        text: &str,
        output_path: &Path,
        document_name: &str,
    ) -> Result<(), ConversionError> {
        // Convert text to a Lua script that displays it
        let lua_script = xml::text_to_lua_script(text);
        
        // Use the existing Lua conversion pipeline
        self.convert_lua_to_tns(&lua_script, output_path, document_name)
    }

    /// Extract a script from .tns format
    #[allow(dead_code)]
    pub fn extract_from_tns(
        &self,
        _input_path: &Path,
        _output_path: &Path,
    ) -> Result<(), ConversionError> {
        // TODO: Implement extraction pipeline
        todo!("Implement .tns to script extraction")
    }
}

impl Default for Converter {
    fn default() -> Self {
        Self::new()
    }
}

/// Pad data to 8-byte boundary (required for DES encryption)
fn pad_to_8_bytes(mut data: Vec<u8>) -> Vec<u8> {
    let remainder = data.len() % 8;
    if remainder != 0 {
        let padding = 8 - remainder;
        data.extend(vec![0u8; padding]);
    }
    data
}

/// Create a .tns archive with Document.xml and Problem1.xml
///
/// Uses the custom TNS writer that generates proper TI-Nspire format with:
/// - TI-specific magic bytes (*TIMLP + version) for first file
/// - Standard ZIP signatures for subsequent files
/// - TIPD end signature instead of PK\x05\x06
fn create_tns_archive(
    output_path: &Path,
    document_xml: &[u8],
    problem_xml: &[u8],
    _problem_name: &str,
) -> Result<(), ConversionError> {
    let entries = vec![
        TnsFileEntry::new_ti_encrypted("Document.xml", document_xml.to_vec()),
        TnsFileEntry::new_ti_encrypted("Problem1.xml", problem_xml.to_vec()),
    ];

    tns_writer::write_tns_file(output_path, entries, false)
        .map_err(|e| ConversionError::Zip(format!("Failed to write TNS file: {}", e)))
}

/// Create a .tns archive with Document.xml, Problem1.xml, and a Python file
fn create_tns_archive_with_python(
    output_path: &Path,
    document_xml: &[u8],
    problem_xml: &[u8],
    python_filename: &str,
    python_content: &[u8],
) -> Result<(), ConversionError> {
    // Compress Python content using deflate
    let compressed_python = compression::compress_xml(python_content)?;
    let python_crc = crc32fast::hash(python_content);

    let entries = vec![
        TnsFileEntry::new_ti_encrypted("Document.xml", document_xml.to_vec()),
        TnsFileEntry::new_ti_encrypted("Problem1.xml", problem_xml.to_vec()),
        TnsFileEntry::new_deflated(
            python_filename,
            compressed_python,
            python_content.len() as u32,
            python_crc,
        ),
    ];

    tns_writer::write_tns_file(output_path, entries, false)
        .map_err(|e| ConversionError::Zip(format!("Failed to write TNS file: {}", e)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_pad_to_8_bytes() {
        let data = vec![1, 2, 3];
        let padded = pad_to_8_bytes(data);
        assert_eq!(padded.len(), 8);
        assert_eq!(padded, vec![1, 2, 3, 0, 0, 0, 0, 0]);

        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        let padded = pad_to_8_bytes(data);
        assert_eq!(padded.len(), 8);

        let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
        let padded = pad_to_8_bytes(data);
        assert_eq!(padded.len(), 16);
    }

    #[test]
    fn test_convert_lua_to_tns() {
        let converter = Converter::new();
        // Write to project root for comparison
        let output_path = std::path::Path::new("/Users/ivanbelousov/Documents/5 - Code /Projects/Luna/test_rust_output.tns");

        // Use exact same script as test_simple.lua (without trailing newline)
        let lua_script = "-- Simple test\nprint(\"Hello World!\")";
        let result = converter.convert_lua_to_tns(lua_script, &output_path, "");

        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        assert!(output_path.exists());

        // Verify TI header magic
        let bytes = fs::read(&output_path).unwrap();
        assert_eq!(&bytes[0..6], b"*TIMLP", "Should start with TI magic");
        assert_eq!(&bytes[6..10], b"0500", "Should have version 0500");

        // Check that TIPD marker exists
        let len = bytes.len();
        let mut found_tipd = false;
        for i in (0..len.saturating_sub(4)).rev() {
            if &bytes[i..i+4] == b"TIPD" {
                found_tipd = true;
                break;
            }
        }
        assert!(found_tipd, "Should have TIPD end marker");

        // Don't clean up - leave for comparison
        // let _ = fs::remove_file(output_path);
    }

    #[test]
    fn test_convert_python_to_tns() {
        let converter = Converter::new();
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_python.tns");

        let python_script = "print('Hello from Python!')";
        let result = converter.convert_python_to_tns(python_script, "test.py", &output_path, "");

        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        assert!(output_path.exists());

        // Verify TI header magic
        let bytes = fs::read(&output_path).unwrap();
        assert_eq!(&bytes[0..6], b"*TIMLP", "Should start with TI magic");

        // Clean up
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn test_tns_has_tipd_end_marker() {
        let converter = Converter::new();
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_tipd.tns");

        let lua_script = "print('Testing TIPD marker')";
        converter.convert_lua_to_tns(lua_script, &output_path, "").unwrap();

        let bytes = fs::read(&output_path).unwrap();

        // Find TIPD near the end of the file
        let len = bytes.len();
        let mut found_tipd = false;
        for i in (0..len.saturating_sub(4)).rev() {
            if &bytes[i..i+4] == b"TIPD" {
                found_tipd = true;
                break;
            }
        }
        assert!(found_tipd, "Should have TIPD end marker");

        // Clean up
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn test_convert_text_to_tns() {
        let converter = Converter::new();
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_text.tns");

        let plain_text = "Hello, World!\nThis is a plain text note.\nIt supports multiple lines.\n\nUse UP/DOWN arrows to scroll.\nPress ENTER to reset scroll position.\n\nLine 7\nLine 8\nLine 9\nLine 10\nLine 11\nLine 12\nLine 13\nLine 14\nLine 15\nLine 16\nLine 17\nLine 18\nLine 19\nLine 20";

        let result = converter.convert_text_to_tns(plain_text, &output_path, "");

        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        assert!(output_path.exists());

        // Clean up
        let _ = fs::remove_file(output_path);
    }

    #[test]
    fn test_convert_text_with_latex_to_tns() {
        let converter = Converter::new();
        let temp_dir = std::env::temp_dir();
        let output_path = temp_dir.join("test_latex.tns");

        // Text with LaTeX notation
        let math_text = r"Quadratic formula: x = (-b \pm \sqrt{b^2 - 4ac}) / 2a
Einstein: E = mc^2
Greek: \alpha, \beta, \gamma, \delta
Operators: a \times b = c, x \leq y, a \neq b
Symbols: \infty, \sum, \int, \partial, \nabla
Subscripts: H_2O, x_1, a_n
Superscripts: x^2 + y^2 = z^2";

        let result = converter.convert_text_to_tns(math_text, &output_path, "");

        if let Err(e) = &result {
            eprintln!("Error: {:?}", e);
        }
        assert!(result.is_ok());
        assert!(output_path.exists());

        // Clean up
        let _ = fs::remove_file(output_path);
    }
}
