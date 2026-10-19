//! PAT custody is delegated to Crowsi and the selected OS collection.
use crate::{Error, Result};
use crowsi_credential_broker::{
    AccessRequest, Broker, BrokerError, CredentialEntry, CredentialStore, LeaseBinding, SecretRef,
    SecretScope, SecretValue,
};
use std::sync::Arc;
use zeroize::Zeroizing;
const HOST: &str = "hub.docker.com";
const AUDIENCE: &str = "zix-docker";
pub fn reference(namespace: &str, username: &str) -> Result<SecretRef> {
    crate::identifier(namespace)?;
    crate::identifier(username)?;
    let tenant = zix_cli::files::digest(namespace.as_bytes());
    let id = zix_cli::files::digest(format!("https://{HOST}\n{username}").as_bytes());
    let scope = SecretScope::new(&tenant, "docker-hub", "personal-access-token", AUDIENCE)
        .map_err(|_| Error("invalid_credential_scope"))?;
    SecretRef::new(&id, scope).map_err(|_| Error("invalid_credential_scope"))
}
pub fn collection(alias: &str) -> Result<()> {
    if alias == "session"
        || alias.is_empty()
        || alias.len() > 64
        || !alias
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"_-".contains(&b))
    {
        return Err(Error("persistent_collection_required"));
    }
    Ok(())
}
pub fn enroll(
    store: &impl CredentialStore,
    namespace: &str,
    username: &str,
    token: &str,
) -> Result<()> {
    crate::auth::valid_secret(token)?;
    let reference = reference(namespace, username)?;
    match store.metadata(&reference) {
        Err(BrokerError::NotFound) => {}
        Ok(_) => return Err(Error("credential_already_enrolled")),
        Err(_) => return Err(Error("custody_unavailable")),
    }
    let entry = CredentialEntry::new(
        reference,
        [HOST.to_owned()],
        SecretValue::new(token.as_bytes().to_vec()).map_err(|_| Error("invalid_credential"))?,
    )
    .map_err(|_| Error("invalid_credential"))?;
    store
        .put(entry)
        .map_err(|_| Error("custody_import_unknown_do_not_replay"))
}
pub fn load<S: CredentialStore>(
    store: Arc<S>,
    namespace: &str,
    username: &str,
    operation: &str,
) -> Result<Zeroizing<String>> {
    let resource = reference(namespace, username)?;
    let binding = LeaseBinding::new(
        "local-user",
        "local-session",
        "spiffe://zix/local/docker",
        "invocation",
        resource.id(),
        operation,
        "os-session",
    )
    .map_err(|_| Error("invalid_credential_scope"))?;
    let request = AccessRequest::new(resource, AUDIENCE, HOST, 30)
        .map_err(|_| Error("invalid_credential_scope"))?;
    let broker = Broker::new(store).map_err(|_| Error("custody_unavailable"))?;
    let grant = broker
        .issue_bound(request, binding.clone())
        .map_err(|_| Error("custody_unavailable"))?;
    let used = broker
        .consume_bound(grant.token(), AUDIENCE, HOST, &binding)
        .map_err(|_| Error("custody_unavailable"))?;
    used.secret.expose(|bytes| {
        let token = Zeroizing::new(
            String::from_utf8(bytes.to_vec()).map_err(|_| Error("invalid_credential"))?,
        );
        crate::auth::valid_secret(&token)?;
        Ok(token)
    })
}
