use crate::{Hub, Result, Transport};
use serde::{Deserialize, Serialize};
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Absence {
    pub namespace: String,
    pub observed_absent: Vec<String>,
    pub present: Vec<String>,
    pub all_observed_absent: bool,
    pub absence_is_not_deletion_proof: bool,
}
impl<T: Transport> Hub<T> {
    /// Read observations only. A 404 without a preceding acknowledged write is not retirement proof.
    pub fn observe_absence(&mut self, namespace: &str, names: &[String]) -> Result<Absence> {
        crate::validate_targets(namespace, names)?;
        let mut out = Absence {
            namespace: namespace.into(),
            observed_absent: Vec::new(),
            present: Vec::new(),
            all_observed_absent: false,
            absence_is_not_deletion_proof: true,
        };
        for name in names {
            if self.repository(namespace, name)?.is_none() {
                out.observed_absent.push(name.clone());
            } else {
                out.present.push(name.clone());
            }
        }
        out.all_observed_absent = out.present.is_empty();
        Ok(out)
    }
}
