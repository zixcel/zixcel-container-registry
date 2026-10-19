use crate::Error;
/// Stable diagnostics only: never serialize reqwest's URL, headers, body or source error.
pub(crate) fn classify(error: reqwest::Error) -> Error {
    if error.is_timeout() {
        Error("transport_timeout")
    } else if error.is_connect() {
        Error("transport_connect_failed")
    } else if error.is_body() {
        Error("transport_body_failed")
    } else {
        Error("transport_request_failed")
    }
}
pub(crate) fn label(error: Error) -> &'static str {
    match error.0 {
        "transport_timeout"
        | "transport_connect_failed"
        | "transport_body_failed"
        | "transport_request_failed"
        | "transport_failed"
        | "request_budget"
        | "auth_required"
        | "rate_limited"
        | "provider_http_failed"
        | "invalid_response"
        | "identity_mismatch"
        | "credential_missing"
        | "unsafe_target" => error.0,
        _ => "transport_failed",
    }
}
