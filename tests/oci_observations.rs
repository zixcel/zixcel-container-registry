use serde_json::{Value, json};
use zixcel_container_registry::public_inventory::{ManifestReply, State, verify_manifest};
const INDEX: &str = "application/vnd.oci.image.index.v1+json";
fn document() -> Value {
    json!({"schemaVersion":2,"mediaType":INDEX,"manifests":[{"mediaType":"application/vnd.oci.image.manifest.v1+json","digest":format!("sha256:{}","b".repeat(64)),"size":128,"platform":{"os":"linux","architecture":"arm64","variant":"v8"}}]})
}
fn reply(value: &Value) -> (String, ManifestReply) {
    let body = serde_json::to_vec(value).unwrap();
    let digest = format!("sha256:{}", zix_cli::files::digest(&body));
    (
        digest.clone(),
        ManifestReply {
            body,
            content_type: INDEX.into(),
            digest_header: digest,
        },
    )
}
#[test]
fn actual_bytes_digest_media_and_platforms_are_verified_separately_from_hub_metadata() {
    let (digest, r) = reply(&document());
    let observed = verify_manifest(&digest, r).unwrap();
    assert_eq!(observed.state, State::Complete);
    assert_eq!(observed.registry, "https://registry-1.docker.io");
    assert_eq!(observed.platforms[0].architecture, "arm64");
    let (digest, mut r) = reply(&document());
    r.body.push(b' ');
    assert!(verify_manifest(&digest, r).is_err());
    let (digest, mut r) = reply(&document());
    r.digest_header = format!("sha256:{}", "c".repeat(64));
    assert!(verify_manifest(&digest, r).is_err());
    let (digest, mut r) = reply(&document());
    r.content_type = "application/json".into();
    assert!(verify_manifest(&digest, r).is_err());
}
#[test]
fn descriptor_identity_size_platform_schema_and_allocation_bounds_are_enforced() {
    for (field, value) in [
        ("digest", json!("sha256:bad")),
        ("size", json!(0)),
        ("mediaType", json!("text/html")),
        ("platform", json!({"os":"linux","architecture":"../other"})),
    ] {
        let mut v = document();
        v["manifests"][0][field] = value;
        let (digest, r) = reply(&v);
        assert!(verify_manifest(&digest, r).is_err());
    }
    let mut v = document();
    v["schemaVersion"] = json!(1);
    let (digest, r) = reply(&v);
    assert!(verify_manifest(&digest, r).is_err());
    let mut v = document();
    let desc = v["manifests"][0].clone();
    v["manifests"] = json!(vec![desc; 17]);
    let (digest, r) = reply(&v);
    assert!(verify_manifest(&digest, r).is_err());
}
