mod support;
use serde_json::json;
use support::*;
use zixcel_container_registry::{Hub, Method};
#[test]
fn scoped_list_projects_only_allowed_fields() {
    let mut f = Fake::new();
    f.push(
        Method::Get,
        "/v2/namespaces/example/repositories?page_size=100&page=1",
        200,
        page(vec![repo("one")], None),
    );
    let rows = Hub::new(f).repositories("example").unwrap();
    let encoded = serde_json::to_string(&rows).unwrap();
    assert!(!encoded.contains("unprojected"));
    assert_eq!(rows[0].name, "one");
}
#[test]
fn wrong_namespace_and_duplicates_fail_closed() {
    for wrong in [true, false] {
        let mut row = repo("one");
        if wrong {
            row["namespace"] = json!("other");
        }
        let rows = if wrong {
            vec![row]
        } else {
            vec![row.clone(), row]
        };
        let mut f = Fake::new();
        f.push(
            Method::Get,
            "/v2/namespaces/example/repositories?page_size=100&page=1",
            200,
            page(rows, None),
        );
        assert!(Hub::new(f).repositories("example").is_err());
    }
}
#[test]
fn complete_pagination_is_required_and_foreign_next_is_never_followed() {
    for next in [
        None,
        Some("https://attacker.invalid/secret"),
        Some("https://hub.docker.com/v2/namespaces/other/repositories?page_size=100&page=2"),
    ] {
        let mut p = page(vec![repo("one")], next);
        p["count"] = json!(2);
        let mut f = Fake::new();
        f.push(
            Method::Get,
            "/v2/namespaces/example/repositories?page_size=100&page=1",
            200,
            p,
        );
        assert!(Hub::new(f).repositories("example").is_err());
    }
}
#[test]
fn two_pages_are_collected_in_scope() {
    let mut p = page(
        vec![repo("one")],
        Some("https://hub.docker.com/v2/namespaces/example/repositories?page_size=100&page=2"),
    );
    p["count"] = json!(2);
    let mut q = page(vec![repo("two")], None);
    q["count"] = json!(2);
    let mut f = Fake::new();
    f.push(
        Method::Get,
        "/v2/namespaces/example/repositories?page_size=100&page=1",
        200,
        p,
    );
    f.push(
        Method::Get,
        "/v2/namespaces/example/repositories?page_size=100&page=2",
        200,
        q,
    );
    assert_eq!(Hub::new(f).repositories("example").unwrap().len(), 2);
}
