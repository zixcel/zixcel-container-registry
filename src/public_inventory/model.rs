use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum State {
    Complete,
    Partial,
    Denied,
    Unavailable,
    Truncated,
    Unobserved,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManifestObservation {
    pub state: State,
    pub registry: String,
    pub digest: String,
    pub media_type: Option<String>,
    pub platforms: Vec<Platform>,
    pub reason: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Platform {
    pub os: String,
    pub architecture: String,
    pub variant: Option<String>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicTag {
    pub state: State,
    pub reason: Option<String>,
    pub name: String,
    pub digest: Option<String>,
    pub last_updated: String,
    pub oci: Option<ManifestObservation>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicRepository {
    pub name: String,
    pub last_updated: String,
    pub state: State,
    pub reason: Option<String>,
    pub tags: Vec<PublicTag>,
}
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PublicSnapshot {
    pub schema: String,
    pub provider_version: String,
    pub namespace: String,
    pub observed_at_epoch_s: u64,
    pub public_only: bool,
    pub state: State,
    pub reason: Option<String>,
    pub repositories: Vec<PublicRepository>,
}
#[derive(Clone, Copy)]
pub struct Limits {
    pub repositories: usize,
    pub tags: usize,
    pub pages: usize,
    pub oci: bool,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            repositories: 4,
            tags: 4,
            pages: 2,
            oci: false,
        }
    }
}

impl Limits {
    pub fn validate(&self) -> crate::Result<()> {
        if !(1..=8).contains(&self.repositories)
            || !(1..=8).contains(&self.tags)
            || !(1..=4).contains(&self.pages)
        {
            return Err(crate::Error("invalid_inventory_limits"));
        }
        Ok(())
    }
}
