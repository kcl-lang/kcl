use anyhow::{bail, Result};
use encoding::{all::encodings, EncoderTrap};

/// Encoding string value to bytes with specific encoding format.
pub fn encode_text(value: &str, encoding: Option<String>) -> Result<Vec<u8>> {
    if let Some(encoding) = encoding {
        let encoding = normalize_encoding_name(&encoding)?;
        let valid_encodings = encodings();
        for valid_encoding in valid_encodings {
            if valid_encoding.name() == encoding {
                return valid_encoding
                    .encode(value, EncoderTrap::Strict)
                    .map_err(|e| anyhow::anyhow!(e));
            }
        }
        bail!("unknown encoding {encoding}")
    } else {
        Ok(value.as_bytes().to_vec())
    }
}

fn normalize_encoding_name(encoding: &str) -> Result<String> {
    if let Some(i) = encoding.find(|c: char| c == ' ' || c.is_ascii_uppercase()) {
        let mut out = encoding.as_bytes().to_owned();
        for byte in &mut out[i..] {
            if *byte == b' ' {
                *byte = b'-';
            } else {
                byte.make_ascii_lowercase();
            }
        }
        String::from_utf8(out).map_err(|e| anyhow::anyhow!(e))
    } else {
        Ok(encoding.into())
    }
}
