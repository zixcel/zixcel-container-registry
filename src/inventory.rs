use crate::{Error, Hub, Repository, Result, Snapshot, Tag, Transport};
use std::collections::BTreeSet;
impl<T: Transport> Hub<T> {
    pub fn repositories(&mut self, namespace: &str) -> Result<Vec<Repository>> {
        crate::identifier(namespace)?;
        let raw = self.pages(&format!("/v2/namespaces/{namespace}/repositories"))?;
        let mut seen = BTreeSet::new();
        let mut out = Vec::new();
        for row in raw {
            let repo = crate::client::decode_repository(
                &serde_json::to_vec(&row).map_err(|_| Error("invalid_response"))?,
            )?;
            if repo.namespace != namespace || !seen.insert(repo.name.clone()) {
                return Err(Error("identity_mismatch"));
            }
            out.push(repo);
        }
        out.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(out)
    }
    pub fn snapshot(&mut self, namespace: &str, name: &str) -> Result<Snapshot> {
        let repository = self
            .repository(namespace, name)?
            .ok_or(Error("repository_not_found"))?;
        let raw = self.pages(&format!(
            "/v2/namespaces/{namespace}/repositories/{name}/tags"
        ))?;
        let mut tags = Vec::new();
        let mut seen = BTreeSet::new();
        for row in raw {
            let tag: Tag=serde_json::from_value(serde_json::json!({"name":row["name"],"digest":row["digest"],"last_updated":row["last_updated"]})).map_err(|_| Error("invalid_response"))?;
            if tag.name.is_empty()
                || tag.name.len() > 128
                || !tag
                    .name
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
                || !seen.insert(tag.name.clone())
                || tag.last_updated.len() > 64
                || tag.last_updated.chars().any(char::is_control)
            {
                return Err(Error("invalid_response"));
            }
            if let Some(d) = &tag.digest
                && !d
                    .strip_prefix("sha256:")
                    .is_some_and(zix_cli::files::valid_digest)
            {
                return Err(Error("invalid_response"));
            }
            tags.push(tag);
        }
        tags.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(Snapshot { repository, tags })
    }
}
