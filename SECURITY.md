# Security

Do not send credentials, account inventories or private journals in public Issues.
Report a suspected vulnerability privately through the repository security channel.
If private reporting is unavailable, contact the maintainer without including a
secret or exploit data in a public issue.

Only installed, hash-reviewed native code should receive a token. Use existing
least-privilege credentials and keep the local user/session trusted. Environment
credentials can be observed by sufficiently privileged processes. There is no
claim of isolation from same-user malware, encrypted RAM or a new credential vault.

Do not run write operations from an unreviewed LLM instruction, CI, or server.
Changing a plan, executable, manifest, scope or package version invalidates the
operator binding. A durable journal forbids replay after a crash or unknown result;
it is not an idempotency guarantee supplied by Docker Hub.
