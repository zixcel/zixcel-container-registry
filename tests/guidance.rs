use std::process::Command;
#[test]
fn every_operation_has_owned_guidance_and_secret_free_metadata() {
    let manifest = zixcel_container_registry::manifest::manifest().unwrap();
    let catalog = zixcel_container_registry::guidance().unwrap();
    assert_eq!(manifest.protocol, "zix-provider/v2");
    for c in &manifest.commands {
        assert!(
            catalog
                .entries
                .iter()
                .any(|g| g.operation == c.id && g.selectors.is_empty())
        );
    }
    let o = Command::new(env!("CARGO_BIN_EXE_zix-docker"))
        .arg("--guidance")
        .env_clear()
        .output()
        .unwrap();
    assert!(o.status.success());
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&o.stdout).unwrap(),
        serde_json::to_value(catalog).unwrap()
    );
}
