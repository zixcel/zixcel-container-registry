use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    path::Path,
    process::{Command, Stdio},
};
#[test]
fn helper_verifies_delayed_absence_without_repeating_the_write_or_secret_prompt() {
    let d = tempfile::tempdir().unwrap();
    let fake = d.path().join("zix");
    let log = d.path().join("operations");
    fs::write(&fake,br#"#!/bin/sh
printf '%s\n' "$1 $2 $3 ${4-}" >> "$TEST_OPERATION_LOG"
case "$1 $2 $3 ${4-}" in
'describe docker --format json')
 printf '%s\n' '{"package":"zixcel-container-registry","commands":[{"id":"docker.verify-absent","effect":"read"}]}' ;;
'docker repositories delete plan')
 printf '%s\n' '{"result":{"sha256":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","plan":{"schema":"zix-local-operation/v1","plan":{"namespace":"example","repositories":[{"repository":{"name":"one"}}]}}}}' ;;
'docker repositories delete apply')
 printf '%s\n' '{"result":{"state":"verification_pending","accepted":[{"repository":"one","http_status":202}]}}' ;;
'docker repositories verify-absent --namespace')
 if [ -e "$TEST_OPERATION_LOG.count" ]; then
  printf '%s\n' '{"result":{"namespace":"example","all_observed_absent":true,"present":[],"observed_absent":["one"]}}'
 else
  touch "$TEST_OPERATION_LOG.count"
  printf '%s\n' '{"result":{"namespace":"example","all_observed_absent":false,"present":["one"],"observed_absent":[]}}'
 fi ;;
*) exit 7 ;;
esac
"#).unwrap();
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o700)).unwrap();
    let helper = Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/retire-local.sh");
    let command = format!(
        "bash '{}' example one",
        helper.to_str().unwrap().replace('\'', "'\\''")
    );
    let mut child = Command::new("script")
        .args(["--quiet", "--return", "--command", &command, "/dev/null"])
        .env("ZIX_BIN", &fake)
        .env("TEST_OPERATION_LOG", &log)
        .env("TMPDIR", d.path())
        .env_remove("ZIX_DOCKER_HUB_TOKEN")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(b"example-user\nSYNTHETIC_ONLY\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    let text = format!(
        "{}{}",
        String::from_utf8_lossy(&out.stdout),
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(out.status.success(), "{text}");
    assert!(text.contains("Deletion accepted and readback verified"));
    let calls = fs::read_to_string(log).unwrap();
    assert_eq!(
        calls.lines().filter(|l| l.contains("delete apply")).count(),
        1
    );
    assert_eq!(
        calls
            .lines()
            .filter(|l| l.contains("verify-absent"))
            .count(),
        2
    );
    assert_eq!(text.matches("Existing Docker Hub PAT").count(), 1);
}
