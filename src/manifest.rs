use crate::Result;
use serde_json::{Value, json};
use zix_cli::manifest::Manifest;
pub fn manifest() -> Result<Manifest> {
    let mut commands = Vec::new();
    for (id, words, extra, effect) in [
        ("list", vec!["repositories", "list"], vec![], "read"),
        (
            "get",
            vec!["repositories", "get"],
            vec!["repository"],
            "read",
        ),
        ("tags", vec!["tags", "list"], vec!["repository"], "read"),
        (
            "verify-absent",
            vec!["repositories", "verify-absent"],
            vec!["repositories"],
            "read",
        ),
        (
            "delete-plan",
            vec!["repositories", "delete", "plan"],
            vec!["repositories"],
            "read",
        ),
        (
            "delete-apply",
            vec!["repositories", "delete", "apply"],
            vec!["plan-file", "approval-sha256", "journal-directory"],
            "operator-write",
        ),
    ] {
        let mut args = vec![
            json!({"name":"namespace","required":true,"maximum":100}),
            json!({"name":"username","required":false,"maximum":100}),
        ];
        for name in extra {
            args.push(json!({"name":name,"required":true,"maximum":if name=="approval-sha256"{64}else{2048}}));
        }
        commands.push(json!({"id":format!("docker.{id}"),"words":words,"effect":effect,
            "description":if effect=="read"{"Bounded Docker Hub observation; planning does not delete."}else{"Operator-approved DELETE; verification_pending requires read-only verify-absent. No replay."},
            "required_permissions":[if effect=="read"{"hub:repository:read"}else{"hub:repository:delete"}],"arguments":args}));
    }
    let value = json!({"schema":"zix-extension/v2","package":env!("CARGO_PKG_NAME"),"version":env!("CARGO_PKG_VERSION"),"namespace":"docker",
        "platform":"linux-x86_64","protocol":"zix-provider/v1","credential_environment":["ZIX_DOCKER_HUB_TOKEN"],"runtime_environment":[],"timeout_ms":40000,"commands":commands});
    let manifest: Manifest =
        serde_json::from_value(value).map_err(|_| crate::Error("invalid_manifest"))?;
    manifest.validate()?;
    Ok(manifest)
}
pub fn catalog() -> Result<Value> {
    serde_json::to_value(manifest()?).map_err(|_| crate::Error("invalid_manifest"))
}
pub fn binding() -> Result<zix_operation_contracts::Binding> {
    let bytes = zix_cli::files::bounded(
        std::fs::File::open("/proc/self/exe")
            .map_err(|_| crate::Error("executable_unavailable"))?,
        67108864,
    )?;
    Ok(zix_operation_contracts::Binding::for_catalog(
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        &catalog()?,
        &zix_cli::files::digest(&bytes),
    ))
}
