#![forbid(unsafe_code)]
//! Docker Hub policy belongs here; dispatcher integrity belongs to zix-cli.
mod absence;
mod apply;
mod auth;
mod client;
mod inventory;
mod journal;
pub mod manifest;
mod model;
mod pages;
mod plan;
pub mod provider;
mod transport;
mod transport_error;
pub use absence::Absence;
pub use apply::apply;
pub use client::Hub;
pub use model::{Repository, Snapshot, Tag};
pub use plan::{DeletePlan, plan};
pub use transport::{Https, Method, Reply, Transport};
pub use zix_cli::{Error, Result};

pub fn identifier(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 100
        || !value.starts_with(|c: char| c.is_ascii_alphanumeric())
        || !value.ends_with(|c: char| c.is_ascii_alphanumeric())
        || !value
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
    {
        return Err(Error("invalid_identifier"));
    }
    Ok(())
}

pub(crate) fn validate_targets(namespace: &str, names: &[String]) -> Result<()> {
    identifier(namespace)?;
    if names.is_empty()
        || names.len() > 10
        || names
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len()
            != names.len()
    {
        return Err(Error("invalid_input"));
    }
    for name in names {
        identifier(name)?;
    }
    Ok(())
}
