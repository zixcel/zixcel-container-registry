use super::{ManifestObservation, REGISTRY, State, oci::verify_manifest};
use crate::{Error, Transport};
pub(super) fn observe<T: Transport>(
    transport: &mut T,
    namespace: &str,
    repository: &str,
    digest: &str,
    enabled: bool,
) -> ManifestObservation {
    let result = if enabled {
        transport
            .manifest(namespace, repository, digest)
            .and_then(|r| verify_manifest(digest, r))
    } else {
        Err(Error("oci_not_requested"))
    };
    result.unwrap_or_else(|e| ManifestObservation {
        state: if e.0 == "oci_denied" {
            State::Denied
        } else if enabled {
            State::Unavailable
        } else {
            State::Unobserved
        },
        registry: REGISTRY.into(),
        digest: digest.into(),
        media_type: None,
        platforms: Vec::new(),
        reason: Some(
            match e.0 {
                "oci_denied"
                | "oci_not_requested"
                | "oci_digest_mismatch"
                | "oci_media_type_mismatch"
                | "invalid_oci_manifest"
                | "invalid_oci_index"
                | "invalid_oci_descriptor"
                | "invalid_oci_platform"
                | "oci_descriptor_limit"
                | "unsupported_oci_media_type" => e.0,
                _ => "oci_unavailable",
            }
            .into(),
        ),
    })
}
