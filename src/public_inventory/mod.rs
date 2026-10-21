mod collect;
mod compare;
mod model;
mod oci;
mod oci_observation;
mod pages;
mod platform;
mod snapshot_validation;
mod validation;
pub use compare::{SnapshotDiff, compare, show};
pub use model::{
    Limits, ManifestObservation, Platform, PublicRepository, PublicSnapshot, PublicTag, State,
};
pub use oci::{ManifestReply, verify_manifest};
pub const SCHEMA: &str = "zix-docker-public-snapshot/v1";
pub const REGISTRY: &str = "https://registry-1.docker.io";
