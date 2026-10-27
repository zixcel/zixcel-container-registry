use super::*;
#[test]
fn comparisons_require_schema_scope_versions_order_and_complete_membership() {
    let before = complete();
    let mut after = before.clone();
    after.observed_at_epoch_s += 1;
    after.repositories.clear();
    let diff = compare(&before, &after, 1001).unwrap();
    assert!(diff.complete);
    assert_eq!(diff.removed, vec!["example/one", "example/one:latest"]);
    after.state = State::Truncated;
    after.reason = Some("page_limit".into());
    assert!(compare(&before, &after, 1001).unwrap().removed.is_empty());
    for field in ["schema", "provider_version", "namespace"] {
        let mut value = serde_json::to_value(&before).unwrap();
        value[field] = json!("wrong");
        assert!(show(&serde_json::to_vec(&value).unwrap(), "example", 1001).is_err());
    }
    assert!(compare(&after, &before, 1002).is_err());
    after = before.clone();
    after.repositories[0].tags[0].digest = Some(format!("sha256:{}", "b".repeat(64)));
    assert!(compare(&before, &after, 1001).is_err()); // stale OCI proof cannot be substituted
    after.repositories[0].tags[0].oci = None;
    assert_eq!(
        compare(&before, &after, 1001).unwrap().changed,
        vec!["example/one:latest"]
    );
}

#[test]
fn producer_upgrade_retains_previous_snapshot_reads_and_rejects_unreviewed_versions() {
    let mut value = complete();
    for version in ["0.2.1", "0.3.0"] {
        value.provider_version = version.into();
        assert!(show(&serde_json::to_vec(&value).unwrap(), "example", 1001).is_ok());
    }
    value.provider_version = "0.4.0".into();
    assert!(show(&serde_json::to_vec(&value).unwrap(), "example", 1001).is_err());
}
