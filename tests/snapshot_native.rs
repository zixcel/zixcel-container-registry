use serde_json::json;
use std::{
    io::Write,
    process::{Command, Stdio},
};
use zix_operation_contracts::Binding;
use zixcel_container_registry::{
    manifest,
    public_inventory::{PublicSnapshot, SCHEMA, State},
};
#[test]
fn installed_provider_reads_and_compares_public_snapshots_without_loading_custody() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("public.json");
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let snapshot = PublicSnapshot {
        schema: SCHEMA.into(),
        provider_version: env!("CARGO_PKG_VERSION").into(),
        namespace: "example".into(),
        observed_at_epoch_s: now,
        public_only: true,
        state: State::Complete,
        reason: None,
        repositories: Vec::new(),
    };
    std::fs::write(&file, serde_json::to_vec(&snapshot).unwrap()).unwrap();
    let exe = env!("CARGO_BIN_EXE_zix-docker");
    let binding = Binding::for_catalog(
        "zixcel-container-registry",
        env!("CARGO_PKG_VERSION"),
        &manifest::catalog().unwrap(),
        &zix_cli::files::digest(&std::fs::read(exe).unwrap()),
    );
    for (operation, args) in [
        (
            "docker.snapshot-show",
            json!({"namespace":"example","snapshot-file":file}),
        ),
        (
            "docker.snapshot-diff",
            json!({"namespace":"example","before-file":file,"after-file":file}),
        ),
    ] {
        let input = json!({"schema":"zix-provider/v2","id":"a".repeat(32),"operation":operation,"arguments":args,"limits":{"timeout_ms":40000,"maximum_response_bytes":131072},"binding":binding});
        let mut child = Command::new(exe)
            .arg("--zix-provider-v2")
            .env_clear()
            .env("ZIX_DOCKER_HUB_TOKEN", "SYNTHETIC_TOKEN_NOT_USED")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .take()
            .unwrap()
            .write_all(&serde_json::to_vec(&input).unwrap())
            .unwrap();
        let result = child.wait_with_output().unwrap();
        assert!(
            result.status.success(),
            "{}",
            String::from_utf8_lossy(&result.stderr)
        );
        let value: serde_json::Value = serde_json::from_slice(&result.stdout).unwrap();
        assert!(value.get("error").is_none(), "{value}");
        assert!(!String::from_utf8_lossy(&result.stdout).contains("SYNTHETIC_TOKEN"));
        assert_eq!(value["result"]["namespace"], "example");
    }
}
