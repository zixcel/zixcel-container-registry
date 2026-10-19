#![allow(dead_code)]
use serde_json::{Value, json};
use std::collections::VecDeque;
use zixcel_container_registry::{Error, Method, Reply, Result, Transport};
pub struct Fake {
    pub replies: VecDeque<(Method, String, Result<Reply>)>,
    pub budget_failure: bool,
}
impl Fake {
    pub fn new() -> Self {
        Self {
            replies: VecDeque::new(),
            budget_failure: false,
        }
    }
    pub fn push(&mut self, method: Method, path: &str, status: u16, value: Value) {
        self.replies.push_back((
            method,
            path.into(),
            Ok(Reply {
                status,
                body: serde_json::to_vec(&value).unwrap(),
            }),
        ));
    }
    pub fn error(&mut self, method: Method, path: &str) {
        self.replies
            .push_back((method, path.into(), Err(Error("transport_failed"))));
    }
    pub fn snapshot(&mut self, name: &str) {
        self.push(
            Method::Get,
            &format!("/v2/namespaces/example/repositories/{name}"),
            200,
            repo(name),
        );
        self.push(
            Method::Get,
            &format!("/v2/namespaces/example/repositories/{name}/tags?page_size=100&page=1"),
            200,
            page(vec![tag()], None),
        );
    }
}
impl Transport for Fake {
    fn ensure_budget(&self, _: usize) -> Result<()> {
        if self.budget_failure {
            Err(Error("request_budget"))
        } else {
            Ok(())
        }
    }
    fn request(&mut self, method: Method, path: &str) -> Result<Reply> {
        let (m, p, r) = self
            .replies
            .pop_front()
            .expect("Unexpected network request");
        assert_eq!((method, path), (m, p.as_str()));
        r
    }
}
impl Drop for Fake {
    fn drop(&mut self) {
        if !std::thread::panicking() {
            assert!(self.replies.is_empty(), "Unconsumed expected requests");
        }
    }
}
pub fn repo(name: &str) -> Value {
    json!({"namespace":"example","name":name,"last_updated":"2024-01-01T00:00:00Z","is_private":false,"unprojected":"not exposed"})
}
pub fn tag() -> Value {
    json!({"name":"latest","digest":format!("sha256:{}","a".repeat(64)),"last_updated":"2024-01-01T00:00:00Z"})
}
pub fn page(rows: Vec<Value>, next: Option<&str>) -> Value {
    json!({"count":rows.len(),"next":next,"results":rows})
}
pub fn digest(plan: &zixcel_container_registry::DeletePlan) -> String {
    zix_operation_contracts::digest(&serde_json::to_value(plan).unwrap())
}
pub fn directory() -> tempfile::TempDir {
    use std::os::unix::fs::PermissionsExt;
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    d
}
