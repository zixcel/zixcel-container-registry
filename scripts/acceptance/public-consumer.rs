use zixcel_container_registry::{
    Error, Hub, Method, Reply, Result, Transport,
    public_inventory::{Limits, State, compare, show},
};
struct Unavailable;
impl Transport for Unavailable {
    fn request(&mut self, _: Method, _: &str) -> Result<Reply> {
        Err(Error("transport_timeout"))
    }
    fn ensure_budget(&self, _: usize) -> Result<()> {
        Ok(())
    }
}
#[test]
fn independently_imported_public_contract_preserves_unknown_membership() {
    let before = Hub::new(Unavailable)
        .public_snapshot("example", 1000, Limits::default())
        .unwrap();
    assert_eq!(before.state, State::Unavailable);
    let mut after = before.clone();
    after.observed_at_epoch_s += 1;
    let diff = compare(&before, &after, 1002).unwrap();
    assert!(!diff.complete);
    assert!(diff.removed.is_empty());
    assert!(diff.removals_withheld);
    let bytes = serde_json::to_vec(&before).unwrap();
    assert_eq!(show(&bytes, "example", 1001).unwrap(), before);
    assert!(show(&bytes, "another", 1001).is_err());
}
