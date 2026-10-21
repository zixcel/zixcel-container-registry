use super::{PublicSnapshot, State, snapshot_validation::validate};
use crate::{Error, Result};
use serde::Serialize;
use std::collections::BTreeMap;
#[derive(Serialize)]
pub struct SnapshotDiff {
    pub schema: &'static str,
    pub namespace: String,
    pub complete: bool,
    pub removals_withheld: bool,
    pub observed_only_after: Vec<String>,
    pub removed: Vec<String>,
    pub changed: Vec<String>,
}
pub fn show(bytes: &[u8], namespace: &str, now: u64) -> Result<PublicSnapshot> {
    if bytes.len() > 98_304 {
        return Err(Error("snapshot_limit"));
    }
    let snapshot: PublicSnapshot =
        serde_json::from_slice(bytes).map_err(|_| Error("invalid_snapshot"))?;
    validate(&snapshot)?;
    if snapshot.namespace != namespace {
        return Err(Error("snapshot_scope_mismatch"));
    }
    fresh(&snapshot, now)?;
    Ok(snapshot)
}
pub fn compare(before: &PublicSnapshot, after: &PublicSnapshot, now: u64) -> Result<SnapshotDiff> {
    validate(before)?;
    validate(after)?;
    fresh(before, now)?;
    fresh(after, now)?;
    if before.namespace != after.namespace || before.observed_at_epoch_s > after.observed_at_epoch_s
    {
        return Err(Error("snapshot_scope_or_order"));
    }
    let left = identities(before);
    let right = identities(after);
    let complete = before.state == State::Complete && after.state == State::Complete;
    Ok(SnapshotDiff {
        schema: "zix-docker-public-diff/v1",
        namespace: before.namespace.clone(),
        complete,
        removals_withheld: !complete,
        observed_only_after: right
            .keys()
            .filter(|k| !left.contains_key(*k))
            .cloned()
            .collect(),
        removed: if complete {
            left.keys()
                .filter(|k| !right.contains_key(*k))
                .cloned()
                .collect()
        } else {
            Vec::new()
        },
        changed: right
            .iter()
            .filter(|(k, v)| {
                v.is_some() && left.get(*k).is_some_and(|old| old.is_some() && old != *v)
            })
            .map(|(k, _)| k.clone())
            .collect(),
    })
}
fn identities(snapshot: &PublicSnapshot) -> BTreeMap<String, Option<String>> {
    let mut out = BTreeMap::new();
    for repository in &snapshot.repositories {
        let key = format!("{}/{}", snapshot.namespace, repository.name);
        out.insert(key.clone(), None);
        for tag in &repository.tags {
            out.insert(format!("{key}:{}", tag.name), tag.digest.clone());
        }
    }
    out
}
fn fresh(value: &PublicSnapshot, now: u64) -> Result<()> {
    if value.observed_at_epoch_s == 0
        || value.observed_at_epoch_s > now.saturating_add(60)
        || now.saturating_sub(value.observed_at_epoch_s) > 86_400
    {
        return Err(Error("snapshot_stale_or_future"));
    }
    Ok(())
}
