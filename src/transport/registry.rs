use super::*;
use crate::public_inventory::{ManifestReply, REGISTRY};
use zeroize::Zeroizing;
const ACCEPT: &str = "application/vnd.oci.image.index.v1+json, application/vnd.oci.image.manifest.v1+json, application/vnd.docker.distribution.manifest.list.v2+json, application/vnd.docker.distribution.manifest.v2+json";
impl Https {
    pub(super) fn public_manifest(
        &mut self,
        namespace: &str,
        repository: &str,
        digest: &str,
    ) -> Result<ManifestReply> {
        crate::identifier(namespace)?;
        crate::identifier(repository)?;
        if !digest
            .strip_prefix("sha256:")
            .is_some_and(zix_cli::files::valid_digest)
        {
            return Err(Error("invalid_digest"));
        }
        self.ensure_budget(2)?;
        // Public pull token only. Never forward Hub credentials or follow a challenge-supplied URL.
        let auth =
            self.registry_request(self.client.get("https://auth.docker.io/token").query(&[
                ("service", "registry.docker.io"),
                (
                    "scope",
                    &format!("repository:{namespace}/{repository}:pull"),
                ),
            ]))?;
        let auth_bytes = Zeroizing::new(read_body(auth, 16384)?);
        #[derive(serde::Deserialize)]
        struct Token {
            token: String,
        }
        let value: Token =
            serde_json::from_slice(&auth_bytes).map_err(|_| Error("oci_unavailable"))?;
        let token = Zeroizing::new(value.token);
        if token.is_empty() || token.len() > 8192 || token.chars().any(char::is_control) {
            return Err(Error("oci_unavailable"));
        }
        let mut bearer = HeaderValue::from_str(&format!("Bearer {}", token.as_str()))
            .map_err(|_| Error("oci_unavailable"))?;
        bearer.set_sensitive(true);
        let response = self.registry_request(
            self.client
                .get(format!(
                    "{REGISTRY}/v2/{namespace}/{repository}/manifests/{digest}"
                ))
                .header("Authorization", bearer)
                .header("Accept", ACCEPT),
        )?;
        let content_type = response
            .headers()
            .get("content-type")
            .and_then(|h| h.to_str().ok())
            .ok_or(Error("oci_unavailable"))?
            .to_owned();
        let digest_header = response
            .headers()
            .get("docker-content-digest")
            .and_then(|h| h.to_str().ok())
            .ok_or(Error("oci_unavailable"))?
            .to_owned();
        Ok(ManifestReply {
            content_type,
            digest_header,
            body: read_body(response, 1_048_576)?,
        })
    }
    fn registry_request(
        &mut self,
        request: reqwest::blocking::RequestBuilder,
    ) -> Result<reqwest::blocking::Response> {
        self.ensure_budget(1)?;
        self.remaining -= 1;
        let response = request
            .timeout(
                self.deadline
                    .saturating_duration_since(Instant::now())
                    .min(Duration::from_secs(8)),
            )
            .send()
            .map_err(crate::transport_error::classify)?;
        if response.status().as_u16() != 200 {
            return Err(Error("oci_unavailable"));
        }
        Ok(response)
    }
}
fn read_body(response: reqwest::blocking::Response, limit: usize) -> Result<Vec<u8>> {
    let mut body = Vec::new();
    response
        .take(limit as u64 + 1)
        .read_to_end(&mut body)
        .map_err(|_| Error("transport_body_failed"))?;
    if body.len() > limit {
        return Err(Error("response_limit"));
    }
    Ok(body)
}
