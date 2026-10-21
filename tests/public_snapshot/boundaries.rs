use super::*;
#[test]
fn unsafe_next_identity_duplicate_private_and_bad_dates_are_rejected_or_partial() {
    let mut f = Fake::new();
    let mut private = repo("private-name-must-not-leak");
    private["is_private"] = json!(true);
    f.push(Method::Get, REPOS, 200, page(vec![private], None));
    let s = Hub::new(f)
        .public_snapshot("example", 1000, Limits::default())
        .unwrap();
    assert_eq!(s.state, State::Partial);
    assert!(!serde_json::to_string(&s).unwrap().contains("private-name"));
    for next in [
        "https://attacker.invalid/secret",
        "https://hub.docker.com/v2/namespaces/example/repositories?page=2&page=2&page_size=100",
    ] {
        let mut f = Fake::new();
        let mut p = page(vec![repo("one")], Some(next));
        p["count"] = json!(2);
        f.push(Method::Get, REPOS, 200, p);
        assert_eq!(
            Hub::new(f)
                .public_snapshot("example", 1000, Limits::default())
                .unwrap()
                .state,
            State::Partial
        );
    }
    for date in [
        "2026-02-29T00:00:00Z",
        "2024-01-01T25:00:00Z",
        "2024-+1-01T00:00:00Z",
    ] {
        let mut f = Fake::new();
        let mut r = repo("one");
        r["last_updated"] = json!(date);
        f.push(Method::Get, REPOS, 200, page(vec![r], None));
        assert!(
            Hub::new(f)
                .public_snapshot("example", 1000, Limits::default())
                .is_err()
        );
    }
    let mut f = Fake::new();
    f.push(
        Method::Get,
        REPOS,
        200,
        page(vec![repo("one"), repo("two")], None),
    );
    f.push(Method::Get, &tags("one"), 200, page(vec![], None));
    let s = Hub::new(f)
        .public_snapshot(
            "example",
            1000,
            Limits {
                repositories: 1,
                ..Limits::default()
            },
        )
        .unwrap();
    assert_eq!(s.state, State::Truncated);
    assert_eq!(s.repositories.len(), 1);
}
