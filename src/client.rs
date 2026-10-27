use crate::{Error, Method, Reply, Repository, Result, Transport};
use serde::Deserialize;
pub struct Hub<T: Transport> {
    pub(crate) transport: T,
}
impl<T: Transport> Hub<T> {
    pub fn new(transport: T) -> Self {
        Self { transport }
    }
    pub(crate) fn get(&mut self, path: &str) -> Result<Reply> {
        let r = self.transport.request(Method::Get, path)?;
        match r.status {
            200 | 404 => Ok(r),
            401 => Err(Error("auth_required")),
            403 => Err(Error("permission_denied")),
            429 => Err(Error("rate_limited")),
            _ => Err(Error("provider_http_failed")),
        }
    }
    pub fn repository(&mut self, namespace: &str, name: &str) -> Result<Option<Repository>> {
        crate::identifier(namespace)?;
        crate::identifier(name)?;
        let path = format!("/v2/namespaces/{namespace}/repositories/{name}");
        let r = self.get(&path)?;
        if r.status == 404 {
            return Ok(None);
        }
        let repo: Repository = crate::client::decode_repository(&r.body)?;
        if repo.namespace != namespace || repo.name != name {
            return Err(Error("identity_mismatch"));
        }
        Ok(Some(repo))
    }
}
pub(crate) fn decode_repository(bytes: &[u8]) -> Result<Repository> {
    #[derive(Deserialize)]
    struct Raw {
        namespace: String,
        name: String,
        last_updated: String,
        is_private: bool,
    }
    let raw: Raw = serde_json::from_slice(bytes).map_err(|_| Error("invalid_response"))?;
    crate::identifier(&raw.namespace)?;
    crate::identifier(&raw.name)?;
    if raw.last_updated.is_empty()
        || raw.last_updated.len() > 64
        || raw.last_updated.chars().any(char::is_control)
    {
        return Err(Error("invalid_response"));
    }
    Ok(Repository {
        namespace: raw.namespace,
        name: raw.name,
        last_updated: raw.last_updated,
        is_private: raw.is_private,
    })
}
