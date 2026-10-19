use crate::{DeletePlan, Error, Hub, Method, Result, Transport};
use serde_json::{Value, json};
use std::path::Path;
pub fn apply<T: Transport>(
    hub: &mut Hub<T>,
    plan: &DeletePlan,
    now: u64,
    digest: &str,
    directory: &Path,
) -> Result<Value> {
    let started = std::time::Instant::now();
    plan.validate(now)?;
    let value = serde_json::to_value(plan).map_err(|_| Error("invalid_plan"))?;
    if zix_operation_contracts::digest(&value) != digest {
        return Err(Error("plan_binding_mismatch"));
    }
    // Re-observe the entire batch before any destructive request. This cannot provide a server-side atomic transaction.
    for expected in &plan.repositories {
        let observed = hub.snapshot(&plan.namespace, &expected.repository.name)?;
        if &observed != expected {
            return Err(Error("state_changed"));
        }
    }
    hub.transport.ensure_budget(plan.repositories.len() * 2)?;
    plan.validate(now.saturating_add(started.elapsed().as_secs()))?;
    let mut journal = crate::journal::Journal::begin(directory, digest)?;
    let mut accepted = Vec::new();
    for target in &plan.repositories {
        plan.validate(now.saturating_add(started.elapsed().as_secs()))?;
        let name = &target.repository.name;
        journal.update(&json!({"state":"pending_do_not_replay","namespace":plan.namespace,"pending":name,"accepted":accepted,"plan_sha256":digest}))?;
        let result = match hub.transport.request(
            Method::Delete,
            &format!("/v2/repositories/{}/{name}/", plan.namespace),
        ) {
            Ok(r) => r,
            Err(error) => {
                journal.update(&json!({"state":"unknown_do_not_replay","namespace":plan.namespace,"pending":name,"accepted":accepted,"error_kind":crate::transport_error::label(error),"plan_sha256":digest}))?;
                return Err(Error("unknown_result"));
            }
        };
        if ![200, 202, 204].contains(&result.status) {
            journal.update(&json!({"state":"rejected_do_not_replay","namespace":plan.namespace,"pending":name,"accepted":accepted,"http_status":result.status,"plan_sha256":digest}))?;
            return Err(Error("delete_rejected"));
        }
        accepted.push(json!({"repository":name,"http_status":result.status}));
        // Persist acknowledgement before observing absence or sending the next distinct deletion.
        journal.update(&json!({"state":"acknowledged_do_not_replay","namespace":plan.namespace,"accepted":accepted,"plan_sha256":digest}))?;
    }
    let names = plan
        .repositories
        .iter()
        .map(|s| s.repository.name.clone())
        .collect::<Vec<_>>();
    let observed = match hub.observe_absence(&plan.namespace, &names) {
        Ok(value) => Some(value),
        Err(error) => {
            journal.update(&json!({"state":"verification_pending","namespace":plan.namespace,"accepted":accepted,"read_error_kind":crate::transport_error::label(error),"plan_sha256":digest}))?;
            None
        }
    };
    let verified = observed.as_ref().is_some_and(|o| o.all_observed_absent);
    let receipt = json!({"state":if verified {"complete"}else{"verification_pending"},"namespace":plan.namespace,"accepted":accepted,"observed":observed,"plan_sha256":digest,"verified_absent":verified});
    journal.update(&receipt)?;
    Ok(receipt)
}
