use crate::{Error, Result};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use zix_operation_contracts::Binding;
mod invoke;
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    schema: String,
    id: String,
    operation: String,
    arguments: BTreeMap<String, String>,
    limits: Limits,
    binding: Binding,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Limits {
    timeout_ms: u64,
    maximum_response_bytes: usize,
}
impl Request {
    pub fn validate(&self, binding: &Binding) -> Result<()> {
        if self.schema != "zix-provider/v1"
            || self.id.len() != 32
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || self.binding != *binding
            || self.limits.timeout_ms != 40000
            || self.limits.maximum_response_bytes != 131072
        {
            return Err(Error("invalid_provider_request"));
        }
        let manifest = crate::manifest::manifest()?;
        let op = manifest
            .commands
            .iter()
            .find(|c| c.id == self.operation)
            .ok_or(Error("unknown_operation"))?;
        let mut args = op.words.clone();
        for (name, value) in &self.arguments {
            args.extend([format!("--{name}"), value.clone()]);
        }
        zix_cli::select(&manifest, &args)?;
        Ok(())
    }
}
pub fn run() -> Result<Value> {
    let bytes = zix_cli::files::bounded(std::io::stdin().lock(), 8192)?;
    let request: Request =
        serde_json::from_slice(&bytes).map_err(|_| Error("invalid_provider_request"))?;
    let binding = crate::manifest::binding()?;
    request.validate(&binding)?;
    let mut response =
        json!({"schema":"zix-provider/v1","id":request.id,"operation":request.operation});
    let mut tokens = zeroize::Zeroizing::new(
        std::env::var("ZIX_DOCKER_HUB_TOKEN")
            .ok()
            .into_iter()
            .collect::<Vec<_>>(),
    );
    match invoke::invoke(&request, &binding, &mut tokens) {
        Ok(value) => response["result"] = value,
        Err(error) => response["error"] = json!(error.0),
    }
    if zix_cli::response::contains_secret(&response, &tokens) {
        return Err(Error("credential_rejected"));
    }
    if serde_json::to_vec(&response)
        .map_err(|_| Error("invalid_response"))?
        .len()
        > request.limits.maximum_response_bytes
    {
        return Err(Error("response_limit"));
    }
    Ok(response)
}
