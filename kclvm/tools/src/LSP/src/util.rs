use parking_lot::RwLockReadGuard;
use ra_ap_vfs::{FileId, Vfs};
use serde::{de::DeserializeOwned, Serialize};

/// Deserializes a `T` from a json value.
pub fn from_json<T: DeserializeOwned>(
    what: &'static str,
    json: serde_json::Value,
) -> anyhow::Result<T> {
    T::deserialize(&json)
        .map_err(|e| anyhow::anyhow!("could not deserialize {}: {}: {}", what, e, json))
}

/// Converts the `T` to a json value
pub fn to_json<T: Serialize>(value: T) -> anyhow::Result<serde_json::Value> {
    serde_json::to_value(value).map_err(|e| anyhow::anyhow!("could not serialize to json: {}", e))
}

pub fn get_file_name(vfs: RwLockReadGuard<Vfs>, file_id: FileId) -> anyhow::Result<String> {
    if let Some(path) = vfs.file_path(file_id).as_path() {
        Ok(path
            .as_ref()
            .to_str()
            .ok_or(anyhow::anyhow!("Failed to get file file"))?
            .to_string())
    } else {
        Err(anyhow::anyhow!(
            "{} isn't on the file system.",
            vfs.file_path(file_id)
        ))
    }
}
