use crate::{Error, Result};
use zeroize::Zeroizing;
pub(crate) fn load(
    namespace: &str,
    username: Option<&str>,
    collection: Option<&str>,
    operation: &str,
) -> Result<Zeroizing<Option<String>>> {
    let supplied = Zeroizing::new(std::env::var("ZIX_DOCKER_HUB_TOKEN").ok());
    if let Some(alias) = collection {
        if supplied.is_some() {
            return Err(Error("ambiguous_credential_source"));
        }
        crate::custody::collection(alias)?;
        let username = username.ok_or(Error("custody_username_required"))?;
        let store = crowsi_credential_broker::LinuxCustodyStore::connect(alias)
            .map_err(|_| Error("custody_unavailable"))?;
        let mut token =
            crate::custody::load(std::sync::Arc::new(store), namespace, username, operation)?;
        return Ok(Zeroizing::new(Some(std::mem::take(&mut *token))));
    }
    Ok(supplied)
}
