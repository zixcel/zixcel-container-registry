use serde_json::json;
use zix_operation_contracts::Binding;
use zixcel_container_registry::{manifest, provider::Request};
#[test]
fn manifest_and_request_share_exact_write_policy() {
    let m = manifest::manifest().unwrap();
    m.validate().unwrap();
    assert_eq!(m.commands.len(), 9);
    let write = m
        .commands
        .iter()
        .filter(|c| c.effect == "operator-write")
        .collect::<Vec<_>>();
    assert_eq!(write.len(), 1);
    assert_eq!(write[0].id, "docker.delete-apply");
    let binding = Binding::for_catalog(
        "zixcel-container-registry",
        env!("CARGO_PKG_VERSION"),
        &manifest::catalog().unwrap(),
        &"a".repeat(64),
    );
    let mut value = json!({"schema":"zix-provider/v1","id":"a".repeat(32),"operation":"docker.list","arguments":{"namespace":"example"},"limits":{"timeout_ms":40000,"maximum_response_bytes":131072},"binding":binding});
    serde_json::from_value::<Request>(value.clone())
        .unwrap()
        .validate(&binding)
        .unwrap();
    value["arguments"]["unknown"] = json!("bad");
    assert!(
        serde_json::from_value::<Request>(value.clone())
            .unwrap()
            .validate(&binding)
            .is_err()
    );
    value["arguments"]
        .as_object_mut()
        .unwrap()
        .remove("unknown");
    value["binding"]["version"] = json!("9.0.0");
    assert!(
        serde_json::from_value::<Request>(value)
            .unwrap()
            .validate(&binding)
            .is_err()
    );
}
#[test]
fn executable_does_not_echo_invalid_credentials_or_raw_protocol_input() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let mut child = Command::new(env!("CARGO_BIN_EXE_zix-docker"))
        .arg("--zix-provider-v1")
        .env("ZIX_DOCKER_HUB_TOKEN", "SYNTHETIC_SECRET_DO_NOT_ECHO")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"SYNTHETIC_SECRET_DO_NOT_ECHO")
        .unwrap();
    let r = child.wait_with_output().unwrap();
    assert!(!r.status.success());
    assert!(!String::from_utf8_lossy(&r.stderr).contains("SYNTHETIC_SECRET"));
    assert!(r.stdout.is_empty());
}

#[test]
fn explicit_custody_rejects_environment_secret_before_connecting_or_echoing() {
    use std::{
        io::Write,
        process::{Command, Stdio},
    };
    let exe = env!("CARGO_BIN_EXE_zix-docker");
    let binding = Binding::for_catalog(
        "zixcel-container-registry",
        env!("CARGO_PKG_VERSION"),
        &manifest::catalog().unwrap(),
        &zix_cli::files::digest(&std::fs::read(exe).unwrap()),
    );
    let value = json!({"schema":"zix-provider/v1","id":"a".repeat(32),"operation":"docker.list",
        "arguments":{"namespace":"example","username":"example-user","collection":"default"},
        "limits":{"timeout_ms":40000,"maximum_response_bytes":131072},"binding":binding});
    let mut child = Command::new(exe)
        .arg("--zix-provider-v1")
        .env_clear()
        .env("ZIX_DOCKER_HUB_TOKEN", "SYNTHETIC_DO_NOT_ECHO")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&value).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    let result: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["error"], "ambiguous_credential_source");
    for bytes in [&output.stdout, &output.stderr] {
        assert!(!String::from_utf8_lossy(bytes).contains("SYNTHETIC_DO_NOT_ECHO"));
    }
}
