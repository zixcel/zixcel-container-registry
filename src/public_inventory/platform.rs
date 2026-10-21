use super::Platform;
use crate::{Error, Result};
pub(super) fn validate_platform(value: &Platform) -> Result<()> {
    for v in std::iter::once(&value.os)
        .chain(std::iter::once(&value.architecture))
        .chain(value.variant.iter())
    {
        if v.is_empty()
            || v.len() > 64
            || !v
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
        {
            return Err(Error("invalid_oci_platform"));
        }
    }
    Ok(())
}
