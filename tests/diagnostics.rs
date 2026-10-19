mod support;
use serde_json::json;
use support::*;
use zixcel_container_registry::{Hub, Method, apply, plan};
#[test]
fn rejected_delete_retains_numeric_diagnostics_without_server_body_or_replay() {
    let mut f = Fake::new();
    f.snapshot("one");
    f.snapshot("one");
    f.push(
        Method::Delete,
        "/v2/repositories/example/one/",
        403,
        json!({"detail":"SYNTHETIC_SECRET"}),
    );
    let mut h = Hub::new(f);
    let p = plan(&mut h, "example", &["one".into()], 100).unwrap();
    let d = directory();
    assert_eq!(
        apply(&mut h, &p, 101, &digest(&p), d.path()).unwrap_err().0,
        "delete_rejected"
    );
    let bytes = std::fs::read(d.path().join(format!("{}.json", digest(&p)))).unwrap();
    let r: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(r["http_status"], 403);
    assert_eq!(r["pending"], "one");
    assert!(
        !String::from_utf8(bytes)
            .unwrap()
            .contains("SYNTHETIC_SECRET")
    );
}
