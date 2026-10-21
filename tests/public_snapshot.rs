mod support;
use serde_json::json;
use support::*;
use zixcel_container_registry::{
    Hub, Method,
    public_inventory::{Limits, State, compare, show},
};
const REPOS: &str = "/v2/namespaces/example/repositories?page_size=100&page=1";
fn tags(name: &str) -> String {
    format!("/v2/namespaces/example/repositories/{name}/tags?page_size=100&page=1")
}
fn complete() -> zixcel_container_registry::public_inventory::PublicSnapshot {
    let mut f = Fake::new();
    f.push(Method::Get, REPOS, 200, page(vec![repo("one")], None));
    f.push(Method::Get, &tags("one"), 200, page(vec![tag()], None));
    Hub::new(f)
        .public_snapshot("example", 1000, Limits::default())
        .unwrap()
}
#[test]
fn public_projection_is_explicit_bounded_and_has_no_custody_or_private_fields() {
    let s = complete();
    assert_eq!(s.state, State::Complete);
    assert_eq!(
        s.repositories[0].tags[0].oci.as_ref().unwrap().state,
        State::Unobserved
    );
    let bytes = serde_json::to_vec(&s).unwrap();
    let text = String::from_utf8(bytes.clone()).unwrap();
    for denied in [
        "unprojected",
        "credential",
        "collection",
        "is_private",
        "not exposed",
    ] {
        assert!(!text.contains(denied));
    }
    assert_eq!(show(&bytes, "example", 1001).unwrap(), s);
    assert!(show(&bytes, "other", 1001).is_err());
    assert!(show(&bytes, "example", 88000).is_err());
    assert!(show(&bytes, "example", 900).is_err());
}
#[test]
fn denial_timeout_and_partial_pages_never_become_a_complete_empty_inventory() {
    for status in [401, 403, 404, 429, 500] {
        let mut f = Fake::new();
        f.push(Method::Get, REPOS, status, json!({}));
        let s = Hub::new(f)
            .public_snapshot("example", 1000, Limits::default())
            .unwrap();
        assert_ne!(s.state, State::Complete);
        assert!(s.repositories.is_empty());
    }
    let mut f = Fake::new();
    f.error(Method::Get, REPOS);
    assert_eq!(
        Hub::new(f)
            .public_snapshot("example", 1000, Limits::default())
            .unwrap()
            .state,
        State::Unavailable
    );
    let mut f = Fake::new();
    let mut p = page(
        vec![repo("one")],
        Some("https://hub.docker.com/v2/namespaces/example/repositories?page=2&page_size=100"),
    );
    p["count"] = json!(2);
    f.push(Method::Get, REPOS, 200, p);
    f.error(
        Method::Get,
        "/v2/namespaces/example/repositories?page_size=100&page=2",
    );
    f.push(Method::Get, &tags("one"), 403, json!({}));
    let s = Hub::new(f)
        .public_snapshot("example", 1000, Limits::default())
        .unwrap();
    assert_eq!(s.state, State::Unavailable);
    assert_eq!(s.repositories.len(), 1);
    assert_eq!(s.repositories[0].state, State::Denied);
    let mut after = s.clone();
    after.observed_at_epoch_s += 1;
    after.repositories.clear();
    let diff = compare(&s, &after, 1002).unwrap();
    assert!(diff.removals_withheld);
    assert!(diff.removed.is_empty());
}

#[path = "public_snapshot/boundaries.rs"]
mod boundaries;
#[path = "public_snapshot/comparison.rs"]
mod comparison;
