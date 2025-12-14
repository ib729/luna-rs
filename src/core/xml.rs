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

use thiserror::Error;

use super::math_render::latex_to_unicode;

/// Errors that can occur during XML processing
#[derive(Debug, Error)]
pub enum XMLError {
    #[error("Invalid script content: {0}")]
    InvalidContent(String),
    #[allow(dead_code)]
    #[error("XML generation failed: {0}")]
    GenerationFailed(String),
    #[error("UTF-8 encoding error: {0}")]
    EncodingError(String),
}

/// Type of script being processed
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptType {
    Lua,
    Python,
}

/// Parsed script data
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct ScriptData {
    pub script_type: ScriptType,
    pub content: String,
}

/// Wrap a Lua script in the required XML format
///
/// The TI-Nspire calculator expects Lua scripts to be wrapped in a specific
/// XML structure with CDATA sections.
pub fn wrap_lua_script(script: &str, _document_name: &str) -> Result<Vec<u8>, XMLError> {
    // From luna.c lines 289-307: The Lua header with compressed XML structure
    const LUA_HEADER: &[u8] = b"\x54\x49\x58\x43\x30\x31\x30\x30\x2D\x31\x2E\x30\x3F\x3E\x3C\x70\x72\
\x6F\x62\x20\x78\x6D\x6C\x6E\x73\x3D\x22\x75\x72\x6E\x3A\x54\x49\x2E\
\x50\xA8\x5F\x5B\x1F\x0A\x22\x20\x76\x65\x72\x3D\x22\x31\x2E\x30\x22\
\x20\x70\x62\x6E\x61\x6D\x65\x3D\x22\x22\x3E\x3C\x73\x79\x6D\x3E\x0E\
\x01\x3C\x63\x61\x72\x64\x20\x63\x6C\x61\x79\x3D\x22\x30\x22\x20\x68\
\x31\x3D\x22\xF1\x00\x00\xFF\x22\x20\x68\x32\x3D\x22\xF1\x00\x00\xFF\
\x22\x20\x77\x31\x3D\x22\xF1\x00\x00\xFF\x22\x20\x77\x32\x3D\x22\xF1\
\x00\x00\xFF\x22\x3E\x3C\x69\x73\x44\x75\x6D\x6D\x79\x43\x61\x72\x64\
\x3E\x30\x0E\x03\x3C\x66\x6C\x61\x67\x3E\x30\x0E\x04\x3C\x77\x64\x67\
\x74\x20\x78\x6D\x6C\x6E\x73\x3A\x73\x63\x3D\x22\x75\x72\x6E\x3A\x54\
\x49\x2E\x53\xAC\x84\xF2\x2A\x41\x70\x70\x22\x20\x74\x79\x70\x65\x3D\
\x22\x54\x49\x2E\x53\xAC\x84\xF2\x2A\x41\x70\x70\x22\x20\x76\x65\x72\
\x3D\x22\x31\x2E\x30\x22\x3E\x3C\x73\x63\x3A\x6D\x46\x6C\x61\x67\x73\
\x3E\x30\x0E\x06\x3C\x73\x63\x3A\x76\x61\x6C\x75\x65\x3E\x2D\x31\x0E\
\x07\x3C\x73\x63\x3A\x73\x63\x72\x69\x70\x74\x20\x76\x65\x72\x73\x69\
\x6F\x6E\x3D\x22\x35\x31\x32\x22\x20\x69\x64\x3D\x22\x30\x22\x3E\
<![CDATA[";

    const LUA_FOOTER: &[u8] = b"]]>\x0E\x08\x0E\x05\x0E\x02\x0E\x00";

    // Fix CDATA end sequences in the script (from luna.c lines 124-148)
    let fixed_script = fix_cdata_end_seq(script)?;

    // Calculate total size and allocate buffer
    let total_size = LUA_HEADER.len() + fixed_script.len() + LUA_FOOTER.len();
    let mut result = Vec::with_capacity(total_size);

    // Combine header + script + footer
    result.extend_from_slice(LUA_HEADER);
    result.extend_from_slice(fixed_script.as_bytes());
    result.extend_from_slice(LUA_FOOTER);

    Ok(result)
}

/// Wrap a Python script in the required XML format
///
/// Creates the XML wrapper that references the Python script file.
/// The actual .py file is added separately to the TNS archive.
pub fn wrap_python_script(python_filename: &str, _document_name: &str) -> Result<Vec<u8>, XMLError> {
    // From luna.c lines 565-573: Python header and footer
    const PY_HEADER: &[u8] = b"TIXC0100-1.0?><prob xmlns=\"urn:TI.Problem\" ver=\"1.0\" pbname=\"\">\
<sym>\x0E\x01<card clay=\"0\" h1=\"10000\" h2=\"10000\" w1=\"10000\" \
w2=\"10000\"><isDummyCard>0\x0E\x03<flag>0\x0E\x04<wdgt xmlns:py=\"urn:\
TI.PythonEditor\" type=\"TI.PythonEditor\" ver=\"1.0\"><py:data><py:name>";

    const PY_FOOTER: &[u8] = b"\x0E\x07<py:dirf>-10000000\x0E\x08\x0E\x06<py:mFlags>1024\x0E\x09\
<py:value>10\x0E\x0A\x0E\x05\x0E\x02\x0E\x00";

    // Validate filename length (from luna.c lines 576-579)
    if python_filename.len() > 240 {
        return Err(XMLError::InvalidContent(
            "Python script filenames limited to 240 characters".to_string(),
        ));
    }

    // Calculate total size and allocate buffer
    let total_size = PY_HEADER.len() + python_filename.len() + PY_FOOTER.len();
    let mut result = Vec::with_capacity(total_size);

    // Combine header + filename + footer
    result.extend_from_slice(PY_HEADER);
    result.extend_from_slice(python_filename.as_bytes());
    result.extend_from_slice(PY_FOOTER);

    Ok(result)
}

/// Convert plain text to a Lua script that displays the text on TI-Nspire
///
/// Since TI-Nspire doesn't have a native "plain text note" format,
/// we convert the text to a Lua script that renders it on screen.
/// This approach works on all TI-Nspire calculators.
///
/// LaTeX-style math notation is automatically converted to Unicode:
/// - Greek letters: \alpha, \beta, \Gamma, etc.
/// - Operators: \times, \div, \pm, \leq, \geq, etc.
/// - Symbols: \infty, \sum, \int, \partial, etc.
/// - Superscripts: x^2 → x², x^{10} → x¹⁰
/// - Subscripts: x_1 → x₁, x_{10} → x₁₀
pub fn text_to_lua_script(text: &str) -> String {
    // Convert LaTeX notation to Unicode
    let text = latex_to_unicode(text);

    // Escape special characters for Lua long string literal
    // We use [=[ ... ]=] syntax to handle most cases
    // If the text contains ]=], we need to use more equals signs
    let delimiter = find_safe_delimiter(&text);

    format!(
        r#"-- Text Note (generated by Luna-RS)
local text = [{delim}[{text}]{delim}]

local FONT_SIZE = 11
local LINE_HEIGHT = 15
local MARGIN_X = 4
local MARGIN_TOP = 20
local scroll = 0
local max_scroll = 0
local wrapped_lines = {{}}

-- Wrap text to fit screen width
function wrap_text(gc, txt, max_width)
    wrapped_lines = {{}}
    for line in (txt .. "\n"):gmatch("([^\r\n]*)\r?\n") do
        if line == "" then
            table.insert(wrapped_lines, "")
        else
            local current = ""
            for word in line:gmatch("%S+") do
                local test = current == "" and word or (current .. " " .. word)
                if gc:getStringWidth(test) > max_width then
                    if current ~= "" then
                        table.insert(wrapped_lines, current)
                    end
                    -- Handle very long words
                    if gc:getStringWidth(word) > max_width then
                        local chars = ""
                        for c in word:gmatch(".") do
                            if gc:getStringWidth(chars .. c) > max_width then
                                table.insert(wrapped_lines, chars)
                                chars = c
                            else
                                chars = chars .. c
                            end
                        end
                        current = chars
                    else
                        current = word
                    end
                else
                    current = test
                end
            end
            if current ~= "" then
                table.insert(wrapped_lines, current)
            end
        end
    end
end

function on.paint(gc)
    gc:setFont("sansserif", "r", FONT_SIZE)
    local w, h = platform.window:width(), platform.window:height()

    if #wrapped_lines == 0 then
        wrap_text(gc, text, w - MARGIN_X * 2)
    end

    local y = MARGIN_TOP - scroll
    for _, line in ipairs(wrapped_lines) do
        if y + LINE_HEIGHT > 0 and y < h then
            gc:drawString(line, MARGIN_X, y)
        end
        y = y + LINE_HEIGHT
    end

    max_scroll = math.max(0, #wrapped_lines * LINE_HEIGHT - h + MARGIN_TOP + 10)
end

function on.arrowKey(key)
    if key == "up" then
        scroll = math.max(0, scroll - LINE_HEIGHT)
    elseif key == "down" then
        scroll = math.min(max_scroll, scroll + LINE_HEIGHT)
    end
    platform.window:invalidate()
end

function on.enterKey()
    scroll = 0
    platform.window:invalidate()
end

function on.resize()
    wrapped_lines = {{}}
    platform.window:invalidate()
end

platform.window:invalidate()
"#,
        delim = delimiter,
        text = text
    )
}

/// Find a safe delimiter for Lua long string that doesn't appear in the text
fn find_safe_delimiter(text: &str) -> String {
    // Start with no equals signs: [[ ]]
    // If text contains ]], try [=[ ]=]
    // If text contains ]=], try [==[ ]==]
    // And so on...
    
    let mut equals = String::new();
    loop {
        let end_pattern = format!("]{equals}]");
        if !text.contains(&end_pattern) {
            return equals;
        }
        equals.push('=');
        
        // Safety limit - 10 equals signs should be more than enough
        if equals.len() > 10 {
            return equals;
        }
    }
}

/// Wrap plain text as a Lua script in the required XML format
///
/// This converts plain text to a Lua script that displays the text,
/// then wraps it in the TI-Nspire XML format.
#[allow(dead_code)]
pub fn wrap_plain_text(text: &str, document_name: &str) -> Result<Vec<u8>, XMLError> {
    // Convert text to Lua script
    let lua_script = text_to_lua_script(text);

    // Use the existing Lua wrapper
    wrap_lua_script(&lua_script, document_name)
}

/// Create the default Document.xml structure
///
/// This is the pre-encrypted Document.xml that TI-Nspire files require.
pub fn create_default_document_xml() -> &'static [u8] {
    // From luna.c lines 500-516: Pre-encrypted default document
    const DEFAULT_DOCUMENT_XML: &[u8] = b"\x0F\xCE\xD8\xD2\x81\x06\x86\x5B\x4A\x4A\xC5\xCE\xA9\x16\xF2\xD5\x1D\xA8\x2F\x6E\
\x00\x22\xF2\xF0\xC1\xA6\x06\x77\x4D\x7E\xA6\xC0\x3A\xF0\x5C\x74\xBA\xAA\x44\x60\
\xCD\x58\xE6\x70\xD7\x40\xF6\x9C\x17\xDC\xF0\x94\x77\xBF\xCA\xDE\xF7\x02\x09\xC9\
\x62\xB1\x5D\xEF\x22\xFA\x51\x37\xA0\x81\x91\x48\xE1\x83\x4D\xAD\x08\x31\x2D\xD0\
\xD3\xE3\x2D\x60\xAB\x13\xC2\x98\x2B\xED\x39\x5B\x09\x24\x39\x92\x2F\x0C\x7A\x4C\
\x95\x74\x91\x3B\x0C\xF4\x60\xCC\x73\x27\xCB\x07\x7E\x7F\xA9\x17\x87\xE2\xAC\xA2\
\x3B\xCC\xA0\xC4\xE3\x8E\x89\xF0\xC0\x51\x9F\xC2\xBE\xCE\x28\x45\xC3\xD4\x11\x90\
\xA6\xEC\x53\xA0\xFB\x5B\x46\x6B\x41\xAD\xE9\x53\xBB\x97\xDB\xB1\xD2\x68\xE2\xF6\
\x36\x0F\x26\x36\x75\x9B\xE9\x1F\x48\xAD\xE9\x29\x67\x00\x58\x19\xC3\xC0\x12\x76\
\xA0\x4A\x73\xF3\xB1\xD3\x09\x18\xD6\x06\xDD\x97\x24\x53\x3E\x22\xA4\xFB\x82\x50\
\x7B\x7C\x12\x88\x4E\x7D\x41\x80\xFE\x72\x92\x29\x87\xE8\x5C\x56\x72\xFF\x29\x16\
\x8C\x42\x5B\x8B\x9B\xA7\xD2\x08\x6D\xD3\x98\xFF\x91\xA9\x9E\xF3\x93\xA8\x2E\x1C\
\xB2\xA9\x6B\x6A\xDF\xF6\xCE\x2D\x15\x17\xCE\x6E\xC0\x4F\x9A\x9C\x0E\xDF\x19\x8D\
\x2D\xFA\x69\x9F\x11\xD2\x20\x12\xE0\x79\x14\x04\x4E\x62\x8F\x0A\x2A\x18\x72\x5A\
\x8B\x80\xB3\x3C\x9B\xD5\x67\x59\x4B\x51\x4D\xE0\xC3\x38\x28\xC3\xDC\xCD\x39\x22\
\x12\x8C\x40\x55";

    DEFAULT_DOCUMENT_XML
}

/// Get the TI encrypted header for XML files
///
/// This header precedes the deflated and encrypted XML content.
pub fn get_ti_encrypted_header() -> &'static [u8] {
    // From luna.c lines 521-523
    const TI_ENCRYPTED_HEADER: &[u8] = b"\x0F\xCE\xD8\xD2\x81\x06\x86\x5B\x99\xDD\xA2\x3D\xD9\xE9\x4B\xD4\x31\xBB\x50\xB6\
\x4D\xB3\x29\x24\x70\x60\x49\x38\x1C\x30\xF8\x99\x00\x4B\x92\x64\xE4\x58\xE6\xBC";

    TI_ENCRYPTED_HEADER
}

/// Fix CDATA end sequences in Lua scripts
///
/// Replaces occurrences of `]]>` with `]]><![CDATA[` to split CDATA sections
/// and prevent premature ending of the CDATA block.
/// Based on luna.c lines 124-148.
fn fix_cdata_end_seq(script: &str) -> Result<String, XMLError> {
    const CDATA_RESTART: &str = "]]><![CDATA[";
    
    let script_bytes = script.as_bytes();
    let mut result = Vec::with_capacity(script.len());
    let mut i = 0;
    
    while i < script_bytes.len() {
        // Look for "]]>" sequence
        if i + 2 < script_bytes.len()
            && script_bytes[i] == b']'
            && script_bytes[i + 1] == b']'
            && script_bytes[i + 2] == b'>'
        {
            // Copy "]]" 
            result.push(b']');
            result.push(b']');
            i += 2;
            
            // Insert the restart sequence "]]><![CDATA["
            result.extend_from_slice(CDATA_RESTART.as_bytes());
        } else {
            result.push(script_bytes[i]);
            i += 1;
        }
    }
    
    String::from_utf8(result).map_err(|e| XMLError::EncodingError(e.to_string()))
}

/// Convert UTF-8 string to TI-specific encoding
///
/// Based on luna.c `escape_unicode()` function (lines 83-122).
/// Converts UTF-8 characters to the encoding format expected by TI calculators.
#[allow(dead_code)]
pub fn escape_unicode(input: &str) -> Result<Vec<u8>, XMLError> {
    let input_bytes = input.as_bytes();
    let mut result = Vec::with_capacity(input.len() * 2); // Estimate
    
    // Skip UTF-8 BOM if present (from luna.c lines 93-94)
    let start = if input_bytes.len() >= 3
        && input_bytes[0] == 0xEF
        && input_bytes[1] == 0xBB
        && input_bytes[2] == 0xBF
    {
        3
    } else {
        0
    };
    
    let mut i = start;
    while i < input_bytes.len() {
        let (unicode_char, next_i) = utf8_to_unicode(input_bytes, i)?;
        
        // Convert to TI encoding (from luna.c lines 98-112)
        if unicode_char < 0x80 {
            result.push(unicode_char as u8);
        } else if unicode_char < 0x800 {
            result.push((unicode_char >> 8) as u8);
            result.push(unicode_char as u8);
        } else if unicode_char < 0x10000 {
            result.push(0x80);
            result.push((unicode_char >> 8) as u8);
            result.push(unicode_char as u8);
        } else {
            result.push(0x08);
            result.push((unicode_char >> 16) as u8);
            result.push((unicode_char >> 8) as u8);
            result.push(unicode_char as u8);
        }
        
        i = next_i;
    }
    
    Ok(result)
}

/// Read a UTF-8 character from input bytes
///
/// Based on luna.c `utf82unicode()` function (lines 45-80).
/// Returns (unicode_value, next_index).
#[allow(dead_code)]
fn utf8_to_unicode(bytes: &[u8], index: usize) -> Result<(u32, usize), XMLError> {
    if index >= bytes.len() {
        return Ok((0, index));
    }
    
    let b = bytes[index];
    
    // Single byte (ASCII)
    if (b & 0b1000_0000) == 0 {
        return Ok((b as u32, index + 1));
    }
    
    // Two byte sequence
    if (b & 0b1110_0000) == 0b1100_0000 {
        let mut c = ((b & 0b0001_1111) as u32) << 6;
        if index + 1 < bytes.len() {
            c |= (bytes[index + 1] & 0b0011_1111) as u32;
        }
        return Ok((c, (index + 2).min(bytes.len())));
    }
    
    // Three byte sequence
    if (b & 0b1111_0000) == 0b1110_0000 {
        let mut c = ((b & 0b0000_1111) as u32) << 12;
        if index + 1 < bytes.len() {
            c |= ((bytes[index + 1] & 0b0011_1111) as u32) << 6;
        }
        if index + 2 < bytes.len() {
            c |= (bytes[index + 2] & 0b0011_1111) as u32;
        }
        return Ok((c, (index + 3).min(bytes.len())));
    }
    
    // Four byte sequence
    if (b & 0b1111_1000) == 0b1111_0000 {
        let mut c = ((b & 0b0000_0111) as u32) << 18;
        if index + 1 < bytes.len() {
            c |= ((bytes[index + 1] & 0b0011_1111) as u32) << 12;
        }
        if index + 2 < bytes.len() {
            c |= ((bytes[index + 2] & 0b0011_1111) as u32) << 6;
        }
        if index + 3 < bytes.len() {
            c |= (bytes[index + 3] & 0b0011_1111) as u32;
        }
        return Ok((c, (index + 4).min(bytes.len())));
    }
    
    // Invalid UTF-8 sequence
    Ok((0, index + 1))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wrap_lua_script() {
        let script = "print('Hello, World!')";
        let result = wrap_lua_script(script, "test").unwrap();
        
        // Check that result contains the script
        let result_str = String::from_utf8_lossy(&result);
        assert!(result_str.contains(script));
        assert!(result_str.contains("<![CDATA["));
        assert!(result_str.contains("]]>"));
    }

    #[test]
    fn test_wrap_lua_with_cdata_end() {
        let script = "local x = [[some text]]>more text";
        let result = wrap_lua_script(script, "test").unwrap();
        let result_str = String::from_utf8_lossy(&result);
        
        // Should contain the restart sequence
        assert!(result_str.contains("]]><![CDATA["));
    }

    #[test]
    fn test_wrap_python_script() {
        let filename = "test.py";
        let result = wrap_python_script(filename, "test").unwrap();
        let result_str = String::from_utf8_lossy(&result);
        
        assert!(result_str.contains(filename));
        assert!(result_str.contains("TI.PythonEditor"));
    }

    #[test]
    fn test_python_filename_too_long() {
        let long_filename = "a".repeat(250);
        let result = wrap_python_script(&long_filename, "test");
        assert!(result.is_err());
    }

    #[test]
    fn test_fix_cdata_end_seq() {
        let input = "test]]>more";
        let result = fix_cdata_end_seq(input).unwrap();
        assert_eq!(result, "test]]]]><![CDATA[>more");
    }

    #[test]
    fn test_utf8_to_unicode_ascii() {
        let bytes = b"Hello";
        let (c, next) = utf8_to_unicode(bytes, 0).unwrap();
        assert_eq!(c, b'H' as u32);
        assert_eq!(next, 1);
    }

    #[test]
    fn test_escape_unicode_ascii() {
        let input = "Hello";
        let result = escape_unicode(input).unwrap();
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_escape_unicode_with_bom() {
        let input = "\u{FEFF}Hello";  // BOM + Hello
        let result = escape_unicode(input).unwrap();
        // BOM should be skipped
        assert_eq!(result, b"Hello");
    }

    #[test]
    fn test_text_to_lua_script() {
        let text = "Hello, TI-Nspire!\nThis is a plain text note.";
        let lua_script = text_to_lua_script(text);
        
        // Check that the Lua script contains the text
        assert!(lua_script.contains("Hello, TI-Nspire!"));
        assert!(lua_script.contains("This is a plain text note."));
        // Check for Lua structure
        assert!(lua_script.contains("function on.paint(gc)"));
        assert!(lua_script.contains("gc:drawString"));
        assert!(lua_script.contains("platform.window:invalidate()"));
    }

    #[test]
    fn test_text_to_lua_script_with_special_chars() {
        // Test text containing ]] which needs special handling
        let text = "Test with ]] brackets";
        let lua_script = text_to_lua_script(text);
        
        // Should use [=[ ]=] delimiter since text contains ]]
        assert!(lua_script.contains("[=["));
        assert!(lua_script.contains("]=]"));
    }

    #[test]
    fn test_find_safe_delimiter() {
        // No special chars - use [[ ]]
        assert_eq!(find_safe_delimiter("hello world"), "");
        
        // Contains ]] - conflicts with [[ ]], so use [=[ ]=] (one =)
        assert_eq!(find_safe_delimiter("test ]] more"), "=");
        
        // Contains ]=] - doesn't conflict with [[ ]], so use [[ ]]
        assert_eq!(find_safe_delimiter("test ]=] more"), "");
        
        // Contains ]==] - doesn't conflict with [[ ]] or [=[ ]=], so still empty
        assert_eq!(find_safe_delimiter("test ]==] more"), "");
        
        // Contains both ]] and ]=] - first conflicts with [[]], add = to get [=[]=]
        // But ]=] conflicts with [=[]=], so add another = to get [==[]==]
        assert_eq!(find_safe_delimiter("test ]] and ]=] here"), "==");
    }

    #[test]
    fn test_wrap_plain_text() {
        let text = "Hello, TI-Nspire!\nThis is a plain text note.";
        let result = wrap_plain_text(text, "MyNote").unwrap();
        
        // Check that result contains the Lua script wrapped in XML
        let result_str = String::from_utf8_lossy(&result);
        assert!(result_str.contains("Hello"));
        // Should contain CDATA since it's now a Lua script
        assert!(result_str.contains("<![CDATA["));
        // Should contain Lua code
        assert!(result_str.contains("function on.paint"));
    }

    #[test]
    fn test_wrap_plain_text_unicode() {
        let text = "Hello 世界! 🌍";
        let result = wrap_plain_text(text, "UnicodeNote").unwrap();
        assert!(!result.is_empty());
        
        // The Lua script should contain the unicode text
        let result_str = String::from_utf8_lossy(&result);
        assert!(result_str.contains("Hello"));
    }
}
