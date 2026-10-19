# Docker Hub local OS custody

This provider delegates storage and leases to the published Crowsi broker. Linux
builds require libsecret; an existing Secret Service (for example GNOME Keyring)
must be available. Linux/WSL headless sessions can explicitly start the existing
OS provider using `bash scripts/open-credential-session.sh`. That script asks only
for the OS keyring password and opens a working shell; it installs no service or
CI credential. Exit ends that session, retaining encrypted OS data. Do not start a
competing daemon when a desktop/keyring session already owns the store.

Use the verified absolute `zix-docker` executable from the installed release:

```sh
bash scripts/credential-local.sh import /absolute/accepted/zix-docker NAMESPACE LOGIN default
bash scripts/credential-local.sh status /absolute/accepted/zix-docker NAMESPACE LOGIN default
zix docker repositories list --namespace NAMESPACE --username LOGIN --collection default
```

Import reads the existing PAT once with hidden input and sends it through stdin,
never argv, logs, a config file or a receipt. Existing entries are refused; an
unknown import result must be reconciled using status, not blindly repeated.
Cooperating imports are serialized using a private OS runtime lock. This is not
authentication or isolation from malicious code running as the same user.
Import is local storage only, not proof of remote PAT validity. Listing performs
the separate authenticated connection check. No new token or role is created.

Within the open shell, the existing bus is inherited. Owner scripts can also use
this owner's private, owned session-address marker. It contains only the local
DBus address, never a credential. The default alias must resolve to an existing
persistent collection; session-only collections are rejected. The provider does
not automatically unlock or discover another application's credentials.

For a separately authorized retirement use:

```sh
bash scripts/retire-local.sh NAMESPACE old-image --custody LOGIN default
```

The plan and exact hash approval still apply on every DELETE. A stored PAT does
not grant new remote permissions or bypass operator approval. Do not reuse a
completed/unknown plan. Status and listing are sufficient for custody acceptance.

Explicit `credential-lock`/`credential-unlock` act on the selected collection,
which may be shared with other applications. Lock is not PAT revocation. The
backend is software-backed OS custody, not hardware-nonexportable storage and not
protection against arbitrary malicious code already running as the same user.

Daily owner Dependabot covers Cargo and Actions. Ordinary tests cover scope and
credential policy; `bash scripts/acceptance/keyring.sh` additionally runs real
isolated GNOME Secret Service seed, restart, lock and unavailable-provider checks
with synthetic data. It never uses the caller's keyring or real PAT.
