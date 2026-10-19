use crate::{Error, Result, custody};
use crowsi_credential_broker::{CredentialStore, LinuxCustodyStore};
use serde_json::{Value, json};
use std::io::Read;
use zeroize::Zeroizing;
/// Local OS custody only: never contacts Docker or changes remote permissions.
pub fn run(args: &[String]) -> Result<Value> {
    if args.len() != 4
        || std::env::var_os("GITHUB_ACTIONS").is_some()
        || ![
            "credential-import",
            "credential-status",
            "credential-lock",
            "credential-unlock",
        ]
        .contains(&args[0].as_str())
    {
        return Err(Error("invalid_arguments"));
    }
    let _guard = if args[0] == "credential-import" {
        Some(crate::enrollment_lock::acquire()?)
    } else {
        None
    };
    custody::collection(&args[3])?;
    let reference = custody::reference(&args[1], &args[2])?;
    let store = LinuxCustodyStore::connect(&args[3]).map_err(|_| Error("custody_unavailable"))?;
    let state = match args[0].as_str() {
        "credential-import" => {
            let mut bytes = Zeroizing::new(Vec::new());
            std::io::stdin()
                .take(8193)
                .read_to_end(&mut bytes)
                .map_err(|_| Error("invalid_credential"))?;
            let token = std::str::from_utf8(&bytes).map_err(|_| Error("invalid_credential"))?;
            custody::enroll(&store, &args[1], &args[2], token)?;
            "stored"
        }
        "credential-status" => {
            if store
                .is_locked()
                .map_err(|_| Error("custody_unavailable"))?
            {
                "locked"
            } else {
                store
                    .metadata(&reference)
                    .map_err(|_| Error("credential_unavailable"))?;
                "available"
            }
        }
        "credential-lock" => {
            store.lock().map_err(|_| Error("custody_unavailable"))?;
            "locked"
        }
        "credential-unlock" => {
            store.unlock().map_err(|_| Error("custody_unavailable"))?;
            "unlocked"
        }
        _ => return Err(Error("invalid_arguments")),
    };
    Ok(json!({"state":state,"namespace":args[1],"username":args[2],"collection":args[3]}))
}
