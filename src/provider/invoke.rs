use super::Request;
use crate::{DeletePlan, Error, Https, Hub, Result};
use serde_json::{Value, json};
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
use zix_operation_contracts::{Binding, LocalPlan};
pub(super) fn invoke(request: &Request, binding: &Binding) -> Result<Value> {
    let get = |name: &str| {
        request
            .arguments
            .get(name)
            .map(String::as_str)
            .ok_or(Error("missing_argument"))
    };
    let namespace = get("namespace")?;
    crate::identifier(namespace)?;
    if let Some(name) = request.arguments.get("repository") {
        crate::identifier(name)?;
    }
    if let Some(names) = request.arguments.get("repositories") {
        for name in names.split(',') {
            crate::identifier(name)?;
        }
    }
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error("clock_unavailable"))?
        .as_secs();
    // Validate all local arguments and bindings before creating any authenticated connection.
    let mut approved = None;
    if request.operation == "docker.delete-apply" {
        let path = Path::new(get("plan-file")?);
        use std::os::unix::fs::MetadataExt;
        let meta = std::fs::symlink_metadata(path).map_err(|_| Error("unsafe_plan_file"))?;
        if meta.mode() & 0o077 != 0 {
            return Err(Error("unsafe_plan_file"));
        }
        let p: LocalPlan = serde_json::from_slice(&zix_cli::files::read(path, 1048576, false)?)
            .map_err(|_| Error("invalid_plan"))?;
        if !p.matches("docker.delete-apply", binding, get("approval-sha256")?) {
            return Err(Error("plan_binding_mismatch"));
        }
        let inner: DeletePlan =
            serde_json::from_value(p.plan.clone()).map_err(|_| Error("invalid_plan"))?;
        inner.validate(now)?;
        if inner.namespace != namespace {
            return Err(Error("identity_mismatch"));
        }
        approved = Some((inner, zix_operation_contracts::digest(&p.plan)));
    }
    let username = request.arguments.get("username").map(String::as_str);
    let token = zeroize::Zeroizing::new(std::env::var("ZIX_DOCKER_HUB_TOKEN").ok());
    if request.operation == "docker.delete-apply" && token.is_none() {
        return Err(Error("credential_missing"));
    }
    let mut hub = Hub::new(Https::new(username, token.as_deref())?);
    match request.operation.as_str() {
        "docker.list" => Ok(
            json!({"namespace":namespace,"complete":true,"repositories":hub.repositories(namespace)?}),
        ),
        "docker.get" => Ok(
            json!({"repository":hub.repository(namespace,get("repository")?)?,"absence_is_not_deletion_proof":true}),
        ),
        "docker.tags" => {
            Ok(json!({"snapshot":hub.snapshot(namespace,get("repository")?)?,"complete":true}))
        }
        "docker.delete-plan" => {
            let names = get("repositories")?
                .split(',')
                .map(str::to_owned)
                .collect::<Vec<_>>();
            let inner = crate::plan(&mut hub, namespace, &names, now)?;
            let p = LocalPlan::new(
                "docker.delete-apply".into(),
                binding.clone(),
                serde_json::to_value(inner).map_err(|_| Error("invalid_plan"))?,
            );
            Ok(json!({"sha256":p.digest(),"plan":p}))
        }
        "docker.delete-apply" => {
            let (plan, digest) = approved.ok_or(Error("invalid_plan"))?;
            crate::apply(
                &mut hub,
                &plan,
                now,
                &digest,
                Path::new(get("journal-directory")?),
            )
        }
        _ => Err(Error("unknown_operation")),
    }
}
