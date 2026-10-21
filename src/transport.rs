use crate::{Error, Result, public_inventory::ManifestReply};
mod registry;
use reqwest::{blocking::Client, header::HeaderValue};
use std::{
    io::Read,
    time::{Duration, Instant},
};
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Method {
    Get,
    Delete,
}
pub struct Reply {
    pub status: u16,
    pub body: Vec<u8>,
}
pub trait Transport {
    fn request(&mut self, method: Method, path: &str) -> Result<Reply>;
    fn ensure_budget(&self, requests: usize) -> Result<()>;
    fn manifest(&mut self, _: &str, _: &str, _: &str) -> Result<ManifestReply> {
        Err(Error("oci_unavailable"))
    }
}
pub struct Https {
    client: Client,
    authorization: Option<HeaderValue>,
    deadline: Instant,
    remaining: u8,
}
impl Https {
    pub fn new(username: Option<&str>, token: Option<&str>) -> Result<Self> {
        let client = Client::builder()
            .https_only(true)
            .no_proxy()
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never())
            .connect_timeout(Duration::from_secs(4))
            .timeout(Duration::from_secs(8))
            .build()
            .map_err(|_| Error("transport_failed"))?;
        let mut this = Self {
            client,
            authorization: None,
            deadline: Instant::now() + Duration::from_secs(34),
            remaining: 32,
        };
        this.authorization = crate::auth::authorization(&this.client, username, token)?;
        Ok(this)
    }
}
impl Transport for Https {
    fn manifest(
        &mut self,
        namespace: &str,
        repository: &str,
        digest: &str,
    ) -> Result<ManifestReply> {
        self.public_manifest(namespace, repository, digest)
    }
    fn ensure_budget(&self, requests: usize) -> Result<()> {
        if usize::from(self.remaining) < requests || Instant::now() >= self.deadline {
            return Err(Error("request_budget"));
        }
        Ok(())
    }
    fn request(&mut self, method: Method, path: &str) -> Result<Reply> {
        if self.remaining == 0 || Instant::now() >= self.deadline {
            return Err(Error("request_budget"));
        }
        if !path.starts_with("/v2/")
            || path.contains(['#', '%', '\\'])
            || path.contains("..")
            || path.contains("://")
        {
            return Err(Error("unsafe_target"));
        }
        self.remaining -= 1;
        let verb = match method {
            Method::Get => reqwest::Method::GET,
            Method::Delete => reqwest::Method::DELETE,
        };
        let mut request = self
            .client
            .request(verb, format!("https://hub.docker.com{path}"))
            .header("Cache-Control", "no-cache")
            .header("Pragma", "no-cache")
            .timeout(
                self.deadline
                    .saturating_duration_since(Instant::now())
                    .min(Duration::from_secs(8)),
            );
        if let Some(auth) = &self.authorization {
            request = request.header("Authorization", auth.clone());
        } else if method == Method::Delete {
            return Err(Error("credential_missing"));
        }
        let response = request.send().map_err(crate::transport_error::classify)?;
        let status = response.status().as_u16();
        // A known DELETE status must not be lost to reading an unused response body.
        if method == Method::Delete || status != 200 {
            return Ok(Reply {
                status,
                body: Vec::new(),
            });
        }
        let mut body = Vec::new();
        response
            .take(1048577)
            .read_to_end(&mut body)
            .map_err(|_| Error("transport_failed"))?;
        if body.len() > 1048576 {
            return Err(Error("response_limit"));
        }
        Ok(Reply { status, body })
    }
}
