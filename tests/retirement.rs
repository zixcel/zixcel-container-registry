mod support;
use serde_json::json;
use support::*;
use zixcel_container_registry::{Hub, Method, apply, plan};
#[test]
fn approved_plan_deletes_once_and_reads_back_absence() {
    let mut f = Fake::new();
    f.snapshot("one");
    f.snapshot("one");
    f.push(
        Method::Delete,
        "/v2/repositories/example/one/",
        204,
        json!(null),
    );
    f.push(
        Method::Get,
        "/v2/namespaces/example/repositories/one",
        404,
        json!({}),
    );
    let mut h = Hub::new(f);
    let p = plan(&mut h, "example", &["one".into()], 100).unwrap();
    let d = directory();
    let r = apply(&mut h, &p, 101, &digest(&p), d.path()).unwrap();
    assert_eq!(r["state"], "complete");
    assert_eq!(r["verified_absent"], true);
    assert_eq!(std::fs::read_dir(d.path()).unwrap().count(), 1);
}
#[test]
fn changed_tag_blocks_all_writes() {
    let mut f = Fake::new();
    f.snapshot("one");
    f.push(
        Method::Get,
        "/v2/namespaces/example/repositories/one",
        200,
        repo("one"),
    );
    let mut t = tag();
    t["digest"] = json!(format!("sha256:{}", "b".repeat(64)));
    f.push(
        Method::Get,
        "/v2/namespaces/example/repositories/one/tags?page_size=100&page=1",
        200,
        page(vec![t], None),
    );
    let mut h = Hub::new(f);
    let p = plan(&mut h, "example", &["one".into()], 100).unwrap();
    let d = directory();
    assert_eq!(
        apply(&mut h, &p, 101, &digest(&p), d.path()).unwrap_err().0,
        "state_changed"
    );
    assert_eq!(std::fs::read_dir(d.path()).unwrap().count(), 0);
}
#[test]
fn changed_second_target_prevents_first_delete() {
    let mut f = Fake::new();
    f.snapshot("one");
    f.snapshot("two");
    f.snapshot("one");
    f.push(
        Method::Get,
        "/v2/namespaces/example/repositories/two",
        404,
        json!({}),
    );
    let mut h = Hub::new(f);
    let p = plan(&mut h, "example", &["one".into(), "two".into()], 100).unwrap();
    assert!(apply(&mut h, &p, 101, &digest(&p), directory().path()).is_err());
}
#[test]
fn unknown_delete_result_is_journaled_and_never_retried() {
    let mut f = Fake::new();
    f.snapshot("one");
    f.snapshot("one");
    f.error(Method::Delete, "/v2/repositories/example/one/");
    f.snapshot("one");
    let mut h = Hub::new(f);
    let p = plan(&mut h, "example", &["one".into()], 100).unwrap();
    let d = directory();
    assert_eq!(
        apply(&mut h, &p, 101, &digest(&p), d.path()).unwrap_err().0,
        "unknown_result"
    );
    assert_eq!(
        apply(&mut h, &p, 102, &digest(&p), d.path()).unwrap_err().0,
        "replay_forbidden"
    );
}
#[test]
fn expired_tampered_cross_scope_and_duplicate_plans_fail_before_network() {
    let mut f = Fake::new();
    f.snapshot("one");
    let mut h = Hub::new(f);
    let p = plan(&mut h, "example", &["one".into()], 100).unwrap();
    let hash = digest(&p);
    let d = directory();
    assert!(apply(&mut h, &p, 700, &hash, d.path()).is_err());
    assert!(apply(&mut h, &p, 101, &"a".repeat(64), d.path()).is_err());
    let mut q = p.clone();
    q.namespace = "other".into();
    assert!(apply(&mut h, &q, 101, &digest(&q), d.path()).is_err());
    q = p.clone();
    q.repositories.push(q.repositories[0].clone());
    assert!(apply(&mut h, &q, 101, &digest(&q), d.path()).is_err());
    assert!(plan(&mut h, "example", &["one".into(), "../two".into()], 100).is_err());
}
