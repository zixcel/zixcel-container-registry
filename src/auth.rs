use crate::{Error, Result};
use reqwest::{blocking::Client, header::HeaderValue};
use serde_json::Value;
use std::io::Read;
use zeroize::Zeroizing;
pub(crate) fn authorization(
    client: &Client,
    username: Option<&str>,
    token: Option<&str>,
) -> Result<Option<HeaderValue>> {
    if let Some(token) = token {
        valid_secret(token)?;
        let bearer = if let Some(username) = username {
            crate::identifier(username)?;
            let payload = Zeroizing::new(
                serde_json::to_vec(&serde_json::json!({"username":username,"password":token}))
                    .map_err(|_| Error("invalid_credential"))?,
            );
            let response = client
                .post("https://hub.docker.com/v2/users/login/")
                .header("Content-Type", "application/json")
                .body(payload.to_vec())
                .send()
                .map_err(|_| Error("authentication_failed"))?;
            match response.status().as_u16() {
                200 => {}
                401 => return Err(Error("auth_required")),
                403 => return Err(Error("permission_denied")),
                429 => return Err(Error("rate_limited")),
                _ => return Err(Error("authentication_http_failed")),
            }
            let mut bytes = Zeroizing::new(Vec::new());
            response
                .take(16385)
                .read_to_end(&mut bytes)
                .map_err(|_| Error("authentication_failed"))?;
            if bytes.len() > 16384 {
                return Err(Error("response_limit"));
            }
            let mut value: Value =
                serde_json::from_slice(&bytes).map_err(|_| Error("authentication_failed"))?;
            match value["token"].take() {
                Value::String(s) => Zeroizing::new(s),
                _ => return Err(Error("authentication_failed")),
            }
        } else {
            Zeroizing::new(token.to_owned())
        };
        valid_secret(&bearer)?;
        let value = Zeroizing::new(format!("Bearer {}", *bearer));
        let mut header = HeaderValue::from_str(&value).map_err(|_| Error("invalid_credential"))?;
        header.set_sensitive(true);
        return Ok(Some(header));
    } else if username.is_some() {
        return Err(Error("credential_missing"));
    }

    Ok(None)
}
pub(crate) fn valid_secret(token: &str) -> Result<()> {
    if token.is_empty() || token.len() > 8192 || !token.bytes().all(|b| b.is_ascii_graphic()) {
        return Err(Error("invalid_credential"));
    }
    Ok(())
}
