use crate::{Error, Result};
pub(super) fn timestamp(value: &str) -> Result<()> {
    let bytes = value.as_bytes();
    if !(20..=30).contains(&bytes.len())
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes.last() != Some(&b'Z')
    {
        return Err(Error("invalid_timestamp"));
    }
    let number = |start, end| {
        value
            .get(start..end)
            .filter(|v| v.bytes().all(|b| b.is_ascii_digit()))
            .and_then(|v| v.parse::<u32>().ok())
    };
    let year = number(0, 4).ok_or(Error("invalid_timestamp"))?;
    let month = number(5, 7).ok_or(Error("invalid_timestamp"))?;
    let day = number(8, 10).ok_or(Error("invalid_timestamp"))?;
    let leap = year.is_multiple_of(4) && (!year.is_multiple_of(100) || year.is_multiple_of(400));
    let days = match month {
        2 => {
            if leap {
                29
            } else {
                28
            }
        }
        4 | 6 | 9 | 11 => 30,
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        _ => 0,
    };
    if year < 1970
        || day == 0
        || day > days
        || number(11, 13).is_none_or(|n| n > 23)
        || number(14, 16).is_none_or(|n| n > 59)
        || number(17, 19).is_none_or(|n| n > 59)
        || (bytes.len() > 20
            && (bytes[19] != b'.'
                || bytes.len() < 22
                || !bytes[20..bytes.len() - 1].iter().all(u8::is_ascii_digit)))
    {
        return Err(Error("invalid_timestamp"));
    }
    Ok(())
}
pub(super) fn tag(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || value.starts_with(['.', '-'])
        || !value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"._-".contains(&b))
    {
        return Err(Error("invalid_tag"));
    }
    Ok(())
}
pub(super) fn digest(value: &str) -> Result<()> {
    if !value
        .strip_prefix("sha256:")
        .is_some_and(zix_cli::files::valid_digest)
    {
        return Err(Error("invalid_digest"));
    }
    Ok(())
}
