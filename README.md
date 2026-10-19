# Zixcel Container Registry

Docker Hub repository observations and explicit local retirement, as the `docker`
extension of [Zix](https://github.com/zixcel/cli). V2 manifests require the
operator-write dispatcher prepared as 0.2.1 or newer; the 0.2.0 release cannot
install this extension. Verify that both accepted releases exist before installing.
Rust library and native CLI share
the same scope, pagination and deletion policy. No Docker daemon is needed.

```sh
zix docker repositories list --namespace example
zix docker tags list --namespace example --repository example-image
zix docker repositories delete plan --namespace example --repositories old-one,old-two
```

Reads and planning are suitable for local LLM tools: JSON output, bounded requests,
no interactive login, no implicit installation. The plan command returns a
`result.plan` and `result.sha256`. Store only the plan in a private file, inspect
its repository/tag inventory, then use:

```sh
zix docker repositories delete apply --namespace example \
  --plan-file /private/plan.json --approval-sha256 REVIEWED_SHA256 \
  --journal-directory /private/journal
```

The installed dispatcher requires an operator terminal and exact hash confirmation
for deletion. Unlocking credentials is not write approval. This is not unattended
remote execution or protection against malicious code running as the same user.
Namespace and repository names are caller input; no account is embedded in code.

## Authentication

Public observations work anonymously. For an authenticated operation, supply an
existing Docker Hub PAT via the hidden local prompt in
`bash scripts/retire-local.sh NAMESPACE REPOSITORY,REPOSITORY`. The script asks
for the Docker login username, reads the token once, exports it only for that
process session, and removes the shell variable on exit. It does not write Docker
config, tokens, passwords or a new key store. Do not put a token in argv or chat.

Alternatively, the selected process may receive `ZIX_DOCKER_HUB_TOKEN`. With the
non-secret `--username LOGIN` argument this is a PAT exchanged for a Hub bearer
token; without that argument it must already be a Hub bearer token. Authentication
errors do not trigger a login prompt or alternate credential discovery. Existing
PAT permissions determine access; the tool does not create tokens or change roles.

For repeated local operations, explicitly select a persistent OS collection with
`--username LOGIN --collection default`. The native provider uses the published
`crowsi-credential-broker` and GNOME's official Rust `libsecret` binding. PAT bytes
stay in the selected OS keyring; no custom vault, encryption or master-password
format is implemented. Namespace, username, provider origin and audience bind
lookup and one-use credential leases. A locked/unavailable/missing selection fails
closed, with no anonymous or environment-secret fallback. Supplying both a
collection and `ZIX_DOCKER_HUB_TOKEN` is rejected.

[Local custody setup](docs/local-custody.md) explains first-time import and session
unlock. Source support is distinct from a published native package. Never rerun a
retirement to test credential storage; use repository listing instead.

Use `zix docker repositories verify-absent --namespace example --repositories old-one,old-two`
for independent read-only observations. It does not send DELETE, recreate a plan,
or turn an unrelated 404 into proof that a write was accepted. The CLI returning
an accepted result is distinct from its `state` being `complete`; check the state.

## Boundaries and verification

- HTTPS is fixed to hub.docker.com; redirects, proxies and automatic retries are
  disabled. Response bodies, page counts, record counts, request count and time
  are bounded. Pagination is checked and never followed to another origin/scope.
- A deletion plan expires after ten minutes and binds the exact executable,
  manifest, package version, namespace, repository metadata and complete tag list.
- All targets are re-observed before the first DELETE. A private durable one-use
  journal is written first. A failed/unknown write cannot be replayed with that
  plan. Reconcile using read operations; never blindly rerun an unknown write.
- Acknowledgement is durably recorded before readback. All distinct approved
  targets are submitted once, then observed. A 2xx response with a still-visible
  repository returns `verification_pending`, not a false complete or an instruction
  to resend. The helper makes at most six additional read-only observations.
- Completed deletion requires a successful DELETE response and a later scoped 404. A standalone 404 is not proof that an account, workload or repository
  has been retired. This cannot remove externally cached/pulled image bytes.
- Hub provides no atomic batch or compare-and-delete guarantee. Remote changes
  after preflight can race deletion. Stop repository writers before retirement.
- The legacy repository DELETE route is vendor-source-backed, not part of the
  current advertised OpenAPI surface. [Contract evidence](docs/api-contract.md).
- This first slice does not implement OCI manifest verification, show/diff,
  last-successful snapshot management, image push/pull, Docker Engine operations,
  analytics or security scanning. Those remain separate inventory work.

`cargo test --locked --all-targets`, `cargo clippy --locked --all-targets -- -D warnings`,
and `cargo fmt --all --check` run without credentials or Docker. Synthetic API tests
do not establish real account deletion. A native release is separate from source
acceptance; do not advertise a source candidate as an available registry package.

Apache-2.0. [Security](SECURITY.md). Daily Dependabot proposes owner-local updates.
