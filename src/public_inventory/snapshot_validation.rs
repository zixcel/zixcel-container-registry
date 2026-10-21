use super::{PublicSnapshot, REGISTRY, SCHEMA, State, platform, validation};
use crate::{Error, Result};
use std::collections::BTreeSet;
pub(super) fn validate(value: &PublicSnapshot) -> Result<()> {
    crate::identifier(&value.namespace)?;
    let parts = value.provider_version.split('.').collect::<Vec<_>>();
    if value.schema != SCHEMA
        || parts.len() != 3
        || parts[0] != "0"
        || parts[1] != "2"
        || parts[2].is_empty()
        || !parts[2].bytes().all(|b| b.is_ascii_digit())
        || !value.public_only
        || value.repositories.len() > 8
        || value.state == State::Unobserved
    {
        return Err(Error("invalid_snapshot"));
    }
    reason(&value.reason)?;
    let mut names = BTreeSet::new();
    for repository in &value.repositories {
        crate::identifier(&repository.name)?;
        validation::timestamp(&repository.last_updated)?;
        reason(&repository.reason)?;
        if !names.insert(&repository.name)
            || repository.tags.len() > 8
            || repository.state == State::Unobserved
        {
            return Err(Error("invalid_snapshot"));
        }
        if value.state == State::Complete && repository.state != State::Complete {
            return Err(Error("inconsistent_snapshot"));
        }
        let mut tags = BTreeSet::new();
        for tag in &repository.tags {
            validation::tag(&tag.name)?;
            validation::timestamp(&tag.last_updated)?;
            if !tags.insert(&tag.name) {
                return Err(Error("duplicate_tag"));
            }
            if let Some(d) = &tag.digest {
                validation::digest(d)?;
            }
            if let Some(proof) = &tag.oci {
                validation::digest(&proof.digest)?;
                reason(&proof.reason)?;
                if proof.registry != REGISTRY
                    || tag.digest.as_ref() != Some(&proof.digest)
                    || proof.platforms.len() > 16
                {
                    return Err(Error("invalid_oci_observation"));
                }
                for platform in &proof.platforms {
                    platform::validate_platform(platform)?;
                }
                if proof.state == State::Complete {
                    if proof.reason.is_some()
                        || !matches!(
                            proof.media_type.as_deref(),
                            Some(
                                "application/vnd.oci.image.index.v1+json"
                                    | "application/vnd.oci.image.manifest.v1+json"
                                    | "application/vnd.docker.distribution.manifest.list.v2+json"
                                    | "application/vnd.docker.distribution.manifest.v2+json"
                            )
                        )
                    {
                        return Err(Error("invalid_oci_observation"));
                    }
                } else if proof.media_type.is_some() || !proof.platforms.is_empty() {
                    return Err(Error("invalid_oci_observation"));
                }
            }
        }
    }
    if serde_json::to_vec(value)
        .map_err(|_| Error("invalid_snapshot"))?
        .len()
        > 98_304
    {
        return Err(Error("snapshot_limit"));
    }
    Ok(())
}
fn reason(value: &Option<String>) -> Result<()> {
    if value.as_ref().is_some_and(|v| {
        v.is_empty() || v.len() > 64 || !v.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
    }) {
        return Err(Error("invalid_snapshot_reason"));
    }
    Ok(())
}
