use crowsi_credential_broker::LinuxCustodyStore;
use std::{
    io::Write,
    process::{Command, Output, Stdio},
    sync::Arc,
};
use zixcel_container_registry::custody;
fn call(op: &str, token: &[u8]) -> Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_zix-docker"))
        .args([op, "sample-org", "sample-user", "default"])
        .env_clear()
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            std::env::var("DBUS_SESSION_BUS_ADDRESS").unwrap(),
        )
        .env("XDG_RUNTIME_DIR", std::env::var("XDG_RUNTIME_DIR").unwrap())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(token).unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!String::from_utf8_lossy(&output.stdout).contains("synthetic-pat"));
    assert!(!String::from_utf8_lossy(&output.stderr).contains("synthetic-pat"));
    output
}
#[test]
#[ignore = "run scripts/acceptance/keyring.sh with a private synthetic OS provider"]
fn os_custody_persists_without_exporting_pat_and_fails_closed_after_lock_or_service_loss() {
    assert_eq!(
        std::env::var("ZIX_DOCKER_ISOLATED_KEYRING").as_deref(),
        Ok("1")
    );
    let phase = std::env::var("ZIX_DOCKER_KEYRING_PHASE").unwrap();
    if phase == "unavailable" {
        assert!(!call("credential-status", b"").status.success());
        return;
    }
    if phase == "seed" {
        assert!(call("credential-import", b"synthetic-pat").status.success());
        assert!(!call("credential-import", b"replacement").status.success());
    }
    assert!(call("credential-status", b"").status.success());
    let store = Arc::new(LinuxCustodyStore::connect("default").unwrap());
    assert_eq!(
        &*custody::load(store.clone(), "sample-org", "sample-user", "docker.list").unwrap(),
        "synthetic-pat"
    );
    assert!(custody::load(store.clone(), "other-org", "sample-user", "docker.list").is_err());
    if phase == "restart" {
        assert!(call("credential-lock", b"").status.success());
        assert!(custody::load(store, "sample-org", "sample-user", "docker.list").is_err());
        assert!(!call("credential-import", b"replacement").status.success());
    }
}
