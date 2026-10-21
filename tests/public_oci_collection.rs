mod support;
use serde_json::json;
use support::*;
use zixcel_container_registry::{
    Error, Hub, Method, Reply, Result, Transport,
    public_inventory::{Limits, ManifestReply, State},
};
struct OciFake {
    hub: Fake,
}
impl Transport for OciFake {
    fn request(&mut self, m: Method, p: &str) -> Result<Reply> {
        self.hub.request(m, p)
    }
    fn ensure_budget(&self, n: usize) -> Result<()> {
        self.hub.ensure_budget(n)
    }
    fn manifest(
        &mut self,
        namespace: &str,
        repository: &str,
        digest: &str,
    ) -> Result<ManifestReply> {
        assert_eq!((namespace, repository), ("example", "one"));
        assert_eq!(digest, format!("sha256:{}", "a".repeat(64)));
        Err(Error("oci_denied"))
    }
}
#[test]
fn denied_manifest_and_absent_digest_remain_partial_per_tag_observations() {
    for missing in [false, true] {
        let mut hub = Fake::new();
        hub.push(
            Method::Get,
            "/v2/namespaces/example/repositories?page_size=100&page=1",
            200,
            page(vec![repo("one")], None),
        );
        let mut row = tag();
        if missing {
            row["digest"] = json!(null);
        }
        hub.push(
            Method::Get,
            "/v2/namespaces/example/repositories/one/tags?page_size=100&page=1",
            200,
            page(vec![row], None),
        );
        let s = Hub::new(OciFake { hub })
            .public_snapshot(
                "example",
                1000,
                Limits {
                    oci: true,
                    ..Limits::default()
                },
            )
            .unwrap();
        assert_eq!(s.state, State::Partial);
        assert_eq!(s.repositories[0].state, State::Partial);
        assert_eq!(
            s.repositories[0].tags[0].state,
            if missing {
                State::Unavailable
            } else {
                State::Denied
            }
        );
        let proof = &s.repositories[0].tags[0].oci;
        if missing {
            assert!(proof.is_none());
        } else {
            assert_eq!(proof.as_ref().unwrap().state, State::Denied);
        }
    }
}
