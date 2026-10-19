use crate::{Error, Hub, Result, Snapshot, Transport};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DeletePlan {
    pub schema: String,
    pub namespace: String,
    pub repositories: Vec<Snapshot>,
    pub expires_at: u64,
    pub nonce: String,
}
impl DeletePlan {
    pub fn validate(&self, now: u64) -> Result<()> {
        crate::identifier(&self.namespace)?;
        let mut names = BTreeSet::new();
        if self.schema != "zix-docker-delete/v1"
            || self.expires_at <= now
            || self.expires_at > now.saturating_add(600)
            || self.repositories.is_empty()
            || self.repositories.len() > 10
            || self.nonce.len() != 32
            || !self.nonce.bytes().all(|b| b.is_ascii_hexdigit())
        {
            return Err(Error("invalid_plan"));
        }
        for s in &self.repositories {
            crate::identifier(&s.repository.name)?;
            if s.repository.namespace != self.namespace || !names.insert(&s.repository.name) {
                return Err(Error("invalid_plan"));
            }
        }
        Ok(())
    }
}
pub fn plan<T: Transport>(
    hub: &mut Hub<T>,
    namespace: &str,
    names: &[String],
    now: u64,
) -> Result<DeletePlan> {
    crate::validate_targets(namespace, names)?;
    let mut repositories = Vec::new();
    for name in names {
        repositories.push(hub.snapshot(namespace, name)?);
    }
    let mut nonce = [0u8; 16];
    getrandom::fill(&mut nonce).map_err(|_| Error("entropy_unavailable"))?;
    let result = DeletePlan {
        schema: "zix-docker-delete/v1".into(),
        namespace: namespace.into(),
        repositories,
        expires_at: now.saturating_add(600),
        nonce: nonce.iter().map(|b| format!("{b:02x}")).collect(),
    };
    result.validate(now)?;
    Ok(result)
}
