use super::Request;
use crate::{
    Error, Https, Hub, Result,
    public_inventory::{Limits, PublicSnapshot, compare, show},
};
use serde_json::Value;
use std::{
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};
pub(super) fn invoke(request: &Request) -> Result<Value> {
    let get = |name: &str| {
        request
            .arguments
            .get(name)
            .map(String::as_str)
            .ok_or(Error("missing_argument"))
    };
    let namespace = get("namespace")?;
    crate::identifier(namespace)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| Error("clock_unavailable"))?
        .as_secs();
    let load = |name| -> Result<PublicSnapshot> {
        show(
            &zix_cli::files::read(Path::new(get(name)?), 98_304, false)?,
            namespace,
            now,
        )
    };
    match request.operation.as_str() {
        "docker.snapshot-collect" => {
            let number = |name, default| -> Result<usize> {
                match request.arguments.get(name) {
                    Some(v) => v.parse().map_err(|_| Error("invalid_inventory_limits")),
                    None => Ok(default),
                }
            };
            let oci = match request.arguments.get("oci").map(String::as_str) {
                None | Some("false") => false,
                Some("true") => true,
                _ => return Err(Error("invalid_inventory_limits")),
            };
            let limits = Limits {
                repositories: number("maximum-repositories", 4)?,
                tags: number("maximum-tags", 4)?,
                pages: number("maximum-pages", 2)?,
                oci,
            };
            // Validate arguments before constructing even an anonymous HTTP client.
            limits.validate()?;
            serde_json::to_value(
                Hub::new(Https::new(None, None)?).public_snapshot(namespace, now, limits)?,
            )
            .map_err(|_| Error("invalid_response"))
        }
        "docker.snapshot-show" => {
            serde_json::to_value(load("snapshot-file")?).map_err(|_| Error("invalid_response"))
        }
        "docker.snapshot-diff" => {
            serde_json::to_value(compare(&load("before-file")?, &load("after-file")?, now)?)
                .map_err(|_| Error("invalid_response"))
        }
        _ => Err(Error("unknown_operation")),
    }
}
