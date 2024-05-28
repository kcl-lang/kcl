use crate::client::fs::directory_is_not_empty;
use anyhow::Result;
use oci_distribution::manifest::IMAGE_LAYER_MEDIA_TYPE;
use oci_distribution::secrets::RegistryAuth;
use oci_distribution::{Client, Reference};
use std::path::{Path, PathBuf};

const OCI_SCHEME_PREFIX: &str = "oci://";

#[inline]
pub(crate) fn strip_oci_scheme_prefix(image: &str) -> &str {
    match image.strip_prefix(OCI_SCHEME_PREFIX) {
        Some(image_without_oci_prefix) => image_without_oci_prefix,
        None => image,
    }
}

pub(crate) fn oci_reg_repo_join(reg: &str, repo: &str) -> String {
    let reg = match reg.strip_suffix('/') {
        Some(reg) => reg,
        None => reg,
    };
    let repo = match repo.strip_prefix('/') {
        Some(repo) => repo,
        None => repo,
    };
    format!("{reg}/{repo}")
}

pub(crate) async fn pull_oci_and_extract_layer(
    client: &Client,
    name: &str,
    image: &str,
    tag: &Option<String>,
    save_dir: &Path,
) -> Result<PathBuf> {
    let image = strip_oci_scheme_prefix(image);
    let auth = RegistryAuth::Anonymous;
    let (img_data, path) = match &tag {
        Some(tag) => {
            let path = save_dir.join(format!("{name}_{tag}"));
            if directory_is_not_empty(&path) {
                return Ok(path);
            }
            let img_ref = Reference::try_from(format!("{image}:{tag}"))?;
            (
                client
                    .pull(&img_ref, &auth, vec![IMAGE_LAYER_MEDIA_TYPE])
                    .await?,
                path,
            )
        }
        None => {
            let img_ref = Reference::try_from(image)?;
            let resp = client.list_tags(&img_ref, &auth, None, None).await?;
            let tags = resp.tags;
            let mut semver_tags: Vec<String> =
                tags.into_iter().filter(|tag| tag != "latest").collect();
            semver_tags.sort_by(|a, b| b.cmp(a));
            let (tag, img_tag_ref) = if let Some(newest_tag) = semver_tags.first() {
                (
                    newest_tag.as_str(),
                    Reference::try_from(format!("{image}:{newest_tag}"))?,
                )
            } else {
                ("latest", img_ref)
            };
            let path = save_dir.join(format!("{name}_{tag}"));
            if directory_is_not_empty(&path) {
                return Ok(path);
            }
            (
                client
                    .pull(&img_tag_ref, &auth, vec![IMAGE_LAYER_MEDIA_TYPE])
                    .await?,
                path,
            )
        }
    };
    for layer in &img_data.layers {
        let buf = layer.data.as_slice();
        tar::Archive::new(buf).unpack(&path)?;
    }
    Ok(path)
}
