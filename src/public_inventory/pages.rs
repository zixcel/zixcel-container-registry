use super::State;
use crate::{Hub, Method, Transport};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
#[derive(Deserialize)]
struct Page {
    count: usize,
    next: Option<String>,
    results: Vec<Value>,
}
pub(super) struct Rows {
    pub rows: Vec<Value>,
    pub state: State,
    pub reason: Option<String>,
}
impl Rows {
    fn stop(mut self, state: State, reason: &str) -> Self {
        self.state = state;
        self.reason = Some(reason.into());
        self
    }
}
pub(super) fn observe<T: Transport>(
    hub: &mut Hub<T>,
    path: &str,
    pages: usize,
    limit: usize,
) -> Rows {
    let mut out = Rows {
        rows: Vec::new(),
        state: State::Complete,
        reason: None,
    };
    let mut count = None;
    for page in 1..=pages {
        let reply = match hub
            .transport
            .request(Method::Get, &format!("{path}?page_size=100&page={page}"))
        {
            Ok(r) => r,
            Err(e) => return out.stop(State::Unavailable, crate::transport_error::label(e)),
        };
        if matches!(reply.status, 401 | 403) {
            return out.stop(State::Denied, "access_denied");
        }
        if reply.status != 200 {
            return out.stop(State::Unavailable, "hub_unavailable");
        }
        if reply.body.len() > 1_048_576 {
            return out.stop(State::Unavailable, "response_limit");
        }
        let body: Page = match serde_json::from_slice(&reply.body) {
            Ok(b) => b,
            Err(_) => return out.stop(State::Unavailable, "invalid_page"),
        };
        if count.is_some_and(|c| c != body.count)
            || body.results.len() > 100
            || out.rows.len() + body.results.len() > body.count
        {
            return out.stop(State::Partial, "inconsistent_page");
        }
        count = Some(body.count);
        let returned = body.results.len();
        if let Some(next) = &body.next
            && (returned == 0
                || out.rows.len() + returned >= body.count
                || !safe_next(next, path, page + 1))
        {
            return out.stop(State::Partial, "unsafe_pagination");
        }
        let room = limit.saturating_sub(out.rows.len());
        out.rows.extend(body.results.into_iter().take(room));
        if returned > room {
            return out.stop(State::Truncated, "record_limit");
        }
        if body.next.is_none() {
            return if out.rows.len() == body.count {
                out
            } else {
                out.stop(State::Partial, "missing_page")
            };
        }
        if out.rows.len() >= limit {
            return out.stop(State::Truncated, "record_limit");
        }
    }
    out.stop(State::Truncated, "page_limit")
}
fn safe_next(next: &str, path: &str, page: usize) -> bool {
    if next.len() > 2048 {
        return false;
    }
    let Ok(url) = reqwest::Url::parse(next) else {
        return false;
    };
    let pairs = url.query_pairs().collect::<Vec<_>>();
    let query = pairs.iter().cloned().collect::<BTreeMap<_, _>>();
    let keys = pairs
        .iter()
        .map(|(k, _)| k.as_ref())
        .collect::<BTreeSet<_>>();
    url.scheme() == "https"
        && url.host_str() == Some("hub.docker.com")
        && url.port().is_none()
        && url.username().is_empty()
        && url.password().is_none()
        && url.fragment().is_none()
        && url.path() == path
        && pairs.len() == 2
        && keys.len() == 2
        && query.get("page").is_some_and(|n| n == &page.to_string())
        && query.get("page_size").is_some_and(|n| n == "100")
}
