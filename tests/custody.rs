use crowsi_credential_broker::MemoryStore;
use std::sync::Arc;
use zixcel_container_registry::custody;
#[test]
fn stored_pat_is_bound_to_namespace_username_and_provider() {
    let store = Arc::new(MemoryStore::default());
    custody::enroll(&*store, "sample-org", "sample-user", "synthetic-pat").unwrap();
    assert_eq!(
        &*custody::load(store.clone(), "sample-org", "sample-user", "docker.list").unwrap(),
        "synthetic-pat"
    );
    assert!(custody::load(store.clone(), "other-org", "sample-user", "docker.list").is_err());
    assert!(custody::load(store.clone(), "sample-org", "other-user", "docker.list").is_err());
    assert!(custody::enroll(&*store, "sample-org", "sample-user", "replacement").is_err());
}
#[test]
fn only_explicit_persistent_collections_and_valid_pat_are_accepted() {
    for alias in ["session", "", "../default", "default;other"] {
        assert!(custody::collection(alias).is_err());
    }
    custody::collection("default").unwrap();
    let store = MemoryStore::default();
    for pat in ["", "bad\nsecret", "a b"] {
        assert!(custody::enroll(&store, "sample-org", "sample-user", pat).is_err());
    }
    assert!(custody::reference("Bad/Scope", "sample-user").is_err());
    assert_ne!(
        custody::reference("sample-org", "sample-user").unwrap(),
        custody::reference("sample-user", "sample-org").unwrap()
    );
}
