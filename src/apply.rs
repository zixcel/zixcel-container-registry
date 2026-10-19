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
    let mut journal = crate::journal::Journal::begin(directory, digest)?;
    let mut completed = Vec::new();
    for target in &plan.repositories {
        let name = &target.repository.name;
        // Vendor-maintained hub-tool route, explicitly pinned in docs/api-contract.md.
        let result = hub
            .transport
            .request(
                Method::Delete,
                &format!("/v2/repositories/{}/{name}/", plan.namespace),
            )
            .map_err(|_| Error("unknown_result"))?;
        if ![200, 202, 204].contains(&result.status) {
            return Err(Error("unknown_result"));
        }
        if hub
            .repository(&plan.namespace, name)
            .map_err(|_| Error("unknown_result"))?
            .is_some()
        {
            return Err(Error("unknown_result"));
        }
        completed.push(name);
        journal.update(&json!({"state":"partial_do_not_replay","namespace":plan.namespace,"deleted":completed,"plan_sha256":digest}))?;
    }
    let receipt = json!({"state":"complete","namespace":plan.namespace,"deleted":completed,"plan_sha256":digest,"verified_absent":true});
    journal.update(&receipt)?;
    Ok(receipt)
}
