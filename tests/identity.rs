mod support;
use serde_json::json;
use support::*;
use zixcel_container_registry::{Hub, Method, identifier};
#[test]
fn auth_rate_limit_and_not_found_remain_distinct() {
    for (status, expected) in [
        (401, "auth_required"),
        (403, "permission_denied"),
        (429, "rate_limited"),
        (503, "provider_http_failed"),
    ] {
        let mut f = Fake::new();
        f.push(
            Method::Get,
            "/v2/namespaces/example/repositories/one",
            status,
            json!({}),
        );
        assert_eq!(
            Hub::new(f).repository("example", "one").unwrap_err().0,
            expected
        );
    }
    let mut f = Fake::new();
    f.push(
        Method::Get,
        "/v2/namespaces/example/repositories/one",
        404,
        json!({}),
    );
    assert!(Hub::new(f).repository("example", "one").unwrap().is_none());
}
#[test]
fn identifiers_reject_traversal_and_query_injection_before_network() {
    for name in [
        "../one",
        "One",
        "one?x=y",
        "https://host",
        "one/other",
        "one%2fother",
        "",
    ] {
        assert!(identifier(name).is_err());
        assert!(Hub::new(Fake::new()).repository("example", name).is_err());
    }
}
#[test]
fn tag_identity_and_digest_are_validated() {
    for malformed in [
        json!({"name":"../bad","last_updated":"2024","digest":null}),
        json!({"name":"latest","last_updated":"2024","digest":"not-a-digest"}),
    ] {
        let mut f = Fake::new();
        f.push(
            Method::Get,
            "/v2/namespaces/example/repositories/one",
            200,
            repo("one"),
        );
        f.push(
            Method::Get,
            "/v2/namespaces/example/repositories/one/tags?page_size=100&page=1",
            200,
            page(vec![malformed], None),
        );
        assert!(Hub::new(f).snapshot("example", "one").is_err());
    }
}
