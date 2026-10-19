mod support;
use serde_json::json;
use support::*;
use zixcel_container_registry::{Hub, Method, apply, plan};
#[test]
fn acknowledges_all_distinct_targets_before_observing_delayed_absence() {
    let mut f = Fake::new();
    f.snapshot("one");
    f.snapshot("two");
    f.snapshot("one");
    f.snapshot("two");
    f.push(
        Method::Delete,
        "/v2/repositories/example/one/",
        202,
        json!(null),
    );
    f.push(
        Method::Delete,
        "/v2/repositories/example/two/",
        204,
        json!(null),
    );
    f.push(
        Method::Get,
        "/v2/namespaces/example/repositories/one",
        200,
        repo("one"),
    );
    f.push(
        Method::Get,
        "/v2/namespaces/example/repositories/two",
        404,
        json!({}),
    );
    let mut h = Hub::new(f);
    let p = plan(&mut h, "example", &["one".into(), "two".into()], 100).unwrap();
    let d = directory();
    let r = apply(&mut h, &p, 101, &digest(&p), d.path()).unwrap();
    assert_eq!(r["state"], "verification_pending");
    assert_eq!(r["verified_absent"], false);
    assert_eq!(r["accepted"].as_array().unwrap().len(), 2);
    assert_eq!(r["accepted"][0]["http_status"], 202);
    let j: serde_json::Value = serde_json::from_slice(
        &std::fs::read(d.path().join(format!("{}.json", digest(&p)))).unwrap(),
    )
    .unwrap();
    assert_eq!(j["accepted"], r["accepted"]);
}
#[test]
fn readback_failure_does_not_erase_accepted_write_or_trigger_another_delete() {
    let mut f = Fake::new();
    f.snapshot("one");
    f.snapshot("one");
    f.push(
        Method::Delete,
        "/v2/repositories/example/one/",
        202,
        json!(null),
    );
    f.error(Method::Get, "/v2/namespaces/example/repositories/one");
    let mut h = Hub::new(f);
    let p = plan(&mut h, "example", &["one".into()], 100).unwrap();
    let d = directory();
    let r = apply(&mut h, &p, 101, &digest(&p), d.path()).unwrap();
    assert_eq!(r["state"], "verification_pending");
    assert_eq!(r["accepted"][0]["http_status"], 202);
}
#[test]
fn absence_observation_is_read_only_and_cannot_assert_deletion_without_acknowledgement() {
    let mut f = Fake::new();
    f.push(
        Method::Get,
        "/v2/namespaces/example/repositories/one",
        404,
        json!({}),
    );
    f.push(
        Method::Get,
        "/v2/namespaces/example/repositories/two",
        200,
        repo("two"),
    );
    let r = Hub::new(f)
        .observe_absence("example", &["one".into(), "two".into()])
        .unwrap();
    assert!(!r.all_observed_absent);
    assert_eq!(r.observed_absent, vec!["one"]);
    assert!(r.absence_is_not_deletion_proof);
}
