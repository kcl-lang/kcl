use anyhow::{Result, bail};
use encoding_rs::Encoding;

/// Encoding string value to bytes with specific encoding format.
pub fn encode_text(value: &str, encoding: Option<String>) -> Result<Vec<u8>> {
    if let Some(encoding_name) = encoding {
        let encoding_name = normalize_encoding_name(&encoding_name)?;

        // Look up the encoding by label
        if let Some(encoding) = Encoding::for_label(encoding_name.as_bytes()) {
            // Encode the string
            let (cow, _, had_errors) = encoding.encode(value);

            if had_errors {
                bail!(
                    "encoding errors occurred while encoding to {}",
                    encoding_name
                );
            }

            Ok(cow.into_owned())
        } else {
            bail!("unknown encoding {}", encoding_name)
        }
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
