use super::{PublicTag, validation};
use crate::{Error, Result};
use serde_json::Value;
pub(super) fn tag(row: &Value) -> Result<PublicTag> {
    let name = row["name"].as_str().ok_or(Error("invalid_response"))?;
    validation::tag(name)?;
    let updated = row["last_updated"]
        .as_str()
        .ok_or(Error("invalid_response"))?;
    validation::timestamp(updated)?;
    let digest = match &row["digest"] {
        Value::Null => None,
        Value::String(d) => {
            validation::digest(d)?;
            Some(d.clone())
        }
        _ => return Err(Error("invalid_digest")),
    };
    Ok(PublicTag {
        name: name.into(),
        digest,
        last_updated: updated.into(),
        oci: None,
    })
}
