use super::{
    Limits, PublicRepository, PublicSnapshot, PublicTag, SCHEMA, State, pages, validation,
};
use crate::{Error, Hub, Result, Transport};
use std::collections::BTreeSet;
impl<T: Transport> Hub<T> {
    /// Anonymous/public metadata only; this does not prove image publisher trust.
    pub fn public_snapshot(
        &mut self,
        namespace: &str,
        now: u64,
        limits: Limits,
    ) -> Result<PublicSnapshot> {
        crate::identifier(namespace)?;
        limits.validate()?;
        let page = pages::observe(
            self,
            &format!("/v2/namespaces/{namespace}/repositories"),
            limits.pages,
            limits.repositories,
        );
        let mut snapshot = PublicSnapshot {
            schema: SCHEMA.into(),
            provider_version: env!("CARGO_PKG_VERSION").into(),
            namespace: namespace.into(),
            observed_at_epoch_s: now,
            public_only: true,
            state: page.state,
            reason: page.reason,
            repositories: Vec::new(),
        };
        let mut seen = BTreeSet::new();
        for row in page.rows {
            let repository = crate::client::decode_repository(
                &serde_json::to_vec(&row).map_err(|_| Error("invalid_response"))?,
            )?;
            if repository.namespace != namespace || !seen.insert(repository.name.clone()) {
                return Err(Error("identity_mismatch"));
            }
            if repository.is_private {
                snapshot.state = State::Partial;
                snapshot.reason = Some("public_only_filter".into());
                continue;
            }
            validation::timestamp(&repository.last_updated)?;
            let tags = pages::observe(
                self,
                &format!(
                    "/v2/namespaces/{namespace}/repositories/{}/tags",
                    repository.name
                ),
                limits.pages,
                limits.tags,
            );
            let mut item = PublicRepository {
                name: repository.name,
                last_updated: repository.last_updated,
                state: tags.state,
                reason: tags.reason,
                tags: Vec::new(),
            };
            let mut seen_tags = BTreeSet::new();
            for row in tags.rows {
                let name = row["name"].as_str().ok_or(Error("invalid_response"))?;
                validation::tag(name)?;
                if !seen_tags.insert(name.to_owned()) {
                    return Err(Error("duplicate_tag"));
                }
                let updated = row["last_updated"]
                    .as_str()
                    .ok_or(Error("invalid_response"))?;
                validation::timestamp(updated)?;
                let digest = match &row["digest"] {
                    serde_json::Value::Null => None,
                    serde_json::Value::String(d) => {
                        validation::digest(d)?;
                        Some(d.clone())
                    }
                    _ => return Err(Error("invalid_digest")),
                };
                let oci = digest.as_deref().map(|d| {
                    super::oci_observation::observe(
                        &mut self.transport,
                        namespace,
                        &item.name,
                        d,
                        limits.oci,
                    )
                });
                item.tags.push(PublicTag {
                    name: name.into(),
                    digest,
                    last_updated: updated.into(),
                    oci,
                });
            }
            item.tags.sort_by(|a, b| a.name.cmp(&b.name));
            if item.state != State::Complete
                || item.tags.iter().any(|t| {
                    t.oci
                        .as_ref()
                        .is_some_and(|m| limits.oci && m.state != State::Complete)
                })
            {
                if snapshot.state == State::Complete {
                    snapshot.state = State::Partial;
                    snapshot.reason = Some("incomplete_repository".into());
                }
            }
            snapshot.repositories.push(item);
        }
        snapshot.repositories.sort_by(|a, b| a.name.cmp(&b.name));
        super::snapshot_validation::validate(&snapshot)?;
        Ok(snapshot)
    }
}
