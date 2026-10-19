mod support;
use support::*;
use zixcel_container_registry::{Hub, apply, plan};
#[test]
fn remaining_budget_must_cover_all_mutations_and_readbacks_before_journal() {
    let mut f = Fake::new();
    f.snapshot("one");
    f.snapshot("one");
    f.budget_failure = true;
    let mut h = Hub::new(f);
    let p = plan(&mut h, "example", &["one".into()], 100).unwrap();
    let d = directory();
    assert_eq!(
        apply(&mut h, &p, 101, &digest(&p), d.path()).unwrap_err().0,
        "request_budget"
    );
    assert_eq!(std::fs::read_dir(d.path()).unwrap().count(), 0);
}
