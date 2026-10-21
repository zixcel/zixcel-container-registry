use super::{Limits, PublicRepository, PublicSnapshot, SCHEMA, State, pages, validation};
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
            let repository = match crate::client::decode_repository(
                &serde_json::to_vec(&row).map_err(|_| Error("invalid_response"))?,
            ) {
                Ok(r)
                    if r.namespace == namespace
                        && seen.insert(r.name.clone())
                        && validation::timestamp(&r.last_updated).is_ok() =>
                {
                    r
                }
                _ => {
                    snapshot.state = State::Partial;
                    snapshot.reason = Some("invalid_repository_metadata".into());
                    continue;
                }
            };
            if repository.is_private {
                snapshot.state = State::Partial;
                snapshot.reason = Some("public_only_filter".into());
                continue;
            }
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
                let mut tag = match super::metadata::tag(&row) {
                    Ok(tag) if seen_tags.insert(tag.name.clone()) => tag,
                    _ => {
                        item.state = State::Partial;
                        item.reason = Some("invalid_tag_metadata".into());
                        continue;
                    }
                };
                tag.oci = tag.digest.as_deref().map(|d| {
                    super::oci_observation::observe(
                        &mut self.transport,
                        namespace,
                        &item.name,
                        d,
                        limits.oci,
                    )
                });
                if limits.oci && tag.oci.as_ref().is_none_or(|o| o.state != State::Complete) {
                    item.state = State::Partial;
                    item.reason = Some("incomplete_oci_observation".into());
                }
                item.tags.push(tag);
            }
            item.tags.sort_by(|a, b| a.name.cmp(&b.name));
            if item.state != State::Complete && snapshot.state == State::Complete {
                snapshot.state = State::Partial;
                snapshot.reason = Some("incomplete_repository".into());
            }
            snapshot.repositories.push(item);
        }
        snapshot.repositories.sort_by(|a, b| a.name.cmp(&b.name));
        super::snapshot_validation::validate(&snapshot)?;
        Ok(snapshot)
    }
}
