use std::{
    fs,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
};
#[test]
fn old_installed_provider_is_rejected_before_username_or_secret_input() {
    let d = tempfile::tempdir().unwrap();
    let fake = d.path().join("zix");
    fs::write(&fake,b"#!/bin/sh\nprintf '%s\\n' '{\"package\":\"zixcel-container-registry\",\"version\":\"0.1.0\",\"commands\":[]}'\n").unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o700)).unwrap();
    let helper = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/retire-local.sh");
    let q = |p: &Path| format!("'{}'", p.to_str().unwrap().replace('\'', "'\\''"));
    let command = format!("bash {} example one", q(&helper));
    let out = Command::new("script")
        .args(["--quiet", "--return", "--command", &command, "/dev/null"])
        .env("ZIX_BIN", &fake)
        .env_remove("ZIX_DOCKER_HUB_TOKEN")
        .stdin(Stdio::null())
        .output()
        .unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(!out.status.success());
    assert!(text.contains("before supplying credentials"));
    assert!(!text.contains("login username:"));
    assert!(!text.contains("Existing Docker Hub PAT"));
}
