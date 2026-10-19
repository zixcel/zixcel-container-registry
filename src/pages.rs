use crate::{Error, Hub, Result, Transport};
use serde::Deserialize;
use serde_json::Value;
#[derive(Deserialize)]
struct Page {
    count: usize,
    next: Option<String>,
    results: Vec<Value>,
}
impl<T: Transport> Hub<T> {
    pub(crate) fn pages(&mut self, path: &str) -> Result<Vec<Value>> {
        let mut records = Vec::new();
        let mut expected = None;
        for page in 1..=8 {
            let r = self.get(&format!("{path}?page_size=100&page={page}"))?;
            if r.status != 200 {
                return Err(Error("incomplete_inventory"));
            }
            let body: Page =
                serde_json::from_slice(&r.body).map_err(|_| Error("invalid_response"))?;
            if body.count > 800
                || expected.is_some_and(|c| c != body.count)
                || body.results.len() > 100
            {
                return Err(Error("incomplete_inventory"));
            }
            expected = Some(body.count);
            let empty = body.results.is_empty();
            records.extend(body.results);
            if records.len() > body.count {
                return Err(Error("incomplete_inventory"));
            }
            match body.next {
                None if records.len() == body.count => return Ok(records),
                Some(next) if !empty && records.len() < body.count => {
                    let expected = format!(
                        "https://hub.docker.com{path}?page={}&page_size=100",
                        page + 1
                    );
                    if next.len() > 2048 {
                        return Err(Error("unsafe_pagination"));
                    }
                    let url = reqwest::Url::parse(&next).map_err(|_| Error("unsafe_pagination"))?;
                    let want =
                        reqwest::Url::parse(&expected).map_err(|_| Error("unsafe_pagination"))?;
                    let q = url
                        .query_pairs()
                        .collect::<std::collections::BTreeMap<_, _>>();
                    let w = want
                        .query_pairs()
                        .collect::<std::collections::BTreeMap<_, _>>();
                    if url.query_pairs().count() != 2
                        || url.scheme() != want.scheme()
                        || url.host_str() != want.host_str()
                        || url.port() != want.port()
                        || url.path() != want.path()
                        || !url.username().is_empty()
                        || url.password().is_some()
                        || url.fragment().is_some()
                        || q != w
                    {
                        return Err(Error("unsafe_pagination"));
                    }
                }
                _ => return Err(Error("incomplete_inventory")),
            }
        }
        Err(Error("incomplete_inventory"))
    }
}
