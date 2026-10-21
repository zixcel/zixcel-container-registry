use super::platform::validate_platform;
use super::{ManifestObservation, Platform, REGISTRY, State, validation};
use crate::{Error, Result};
use serde::Deserialize;
use std::collections::BTreeSet;
pub struct ManifestReply {
    pub content_type: String,
    pub digest_header: String,
    pub body: Vec<u8>,
}
#[derive(Deserialize)]
struct Document {
    #[serde(rename = "schemaVersion")]
    schema_version: u32,
    #[serde(rename = "mediaType")]
    media_type: String,
    manifests: Option<Vec<Descriptor>>,
    config: Option<Descriptor>,
    layers: Option<Vec<Descriptor>>,
}
#[derive(Deserialize)]
struct Descriptor {
    #[serde(rename = "mediaType")]
    media_type: String,
    digest: String,
    size: u64,
    platform: Option<RawPlatform>,
}
#[derive(Deserialize)]
struct RawPlatform {
    os: String,
    architecture: String,
    variant: Option<String>,
}
pub fn verify_manifest(digest: &str, reply: ManifestReply) -> Result<ManifestObservation> {
    validation::digest(digest)?;
    if reply.body.len() > 1_048_576
        || reply.digest_header != digest
        || format!("sha256:{}", zix_cli::files::digest(&reply.body)) != digest
    {
        return Err(Error("oci_digest_mismatch"));
    }
    let body: Document =
        serde_json::from_slice(&reply.body).map_err(|_| Error("invalid_oci_manifest"))?;
    if body.schema_version != 2 || body.media_type != reply.content_type {
        return Err(Error("oci_media_type_mismatch"));
    }
    let mut platforms = Vec::new();
    match body.media_type.as_str() {
        "application/vnd.oci.image.index.v1+json"
        | "application/vnd.docker.distribution.manifest.list.v2+json" => {
            let manifests = body.manifests.ok_or(Error("invalid_oci_index"))?;
            if manifests.is_empty()
                || manifests.len() > 16
                || body.config.is_some()
                || body.layers.is_some()
            {
                return Err(Error("invalid_oci_index"));
            }
            let mut digests = BTreeSet::new();
            for descriptor in manifests {
                validate_descriptor(&descriptor)?;
                if !matches!(
                    descriptor.media_type.as_str(),
                    "application/vnd.oci.image.manifest.v1+json"
                        | "application/vnd.docker.distribution.manifest.v2+json"
                ) || !digests.insert(descriptor.digest.clone())
                {
                    return Err(Error("invalid_oci_descriptor"));
                }
                if let Some(platform) = descriptor.platform {
                    let public = Platform {
                        os: platform.os,
                        architecture: platform.architecture,
                        variant: platform.variant,
                    };
                    validate_platform(&public)?;
                    platforms.push(public);
                }
            }
        }
        "application/vnd.oci.image.manifest.v1+json"
        | "application/vnd.docker.distribution.manifest.v2+json" => {
            if body.manifests.is_some() {
                return Err(Error("invalid_oci_manifest"));
            }
            validate_descriptor(&body.config.ok_or(Error("invalid_oci_manifest"))?)?;
            let layers = body.layers.ok_or(Error("invalid_oci_manifest"))?;
            if layers.len() > 128 {
                return Err(Error("oci_descriptor_limit"));
            }
            for layer in layers {
                validate_descriptor(&layer)?;
            }
        }
        _ => return Err(Error("unsupported_oci_media_type")),
    }
    Ok(ManifestObservation {
        state: State::Complete,
        registry: REGISTRY.into(),
        digest: digest.into(),
        media_type: Some(body.media_type),
        platforms,
        reason: None,
    })
}
fn validate_descriptor(value: &Descriptor) -> Result<()> {
    validation::digest(&value.digest)?;
    if value.size == 0
        || value.size > 1_099_511_627_776
        || value.media_type.len() > 128
        || !value.media_type.starts_with("application/")
        || value.media_type.chars().any(char::is_control)
    {
        return Err(Error("invalid_oci_descriptor"));
    }
    Ok(())
}
