# Docker Hub contract evidence

Reviewed 2026-10-19. This file records interface facts and original policy, not
copied vendor SDK implementation or a promise of upstream compatibility.

| Interface | Evidence | Local policy |
| --- | --- | --- |
| GET /v2/namespaces/{namespace}/repositories | [Current Hub API](https://docs.docker.com/reference/api/hub/latest/) | Complete scoped pagination; at most 8 pages / 800 entries |
| GET /v2/namespaces/{namespace}/repositories/{repository} | Same current API | Exact namespace/name identity |
| GET /v2/namespaces/{namespace}/repositories/{repository}/tags | Same current API | Complete list, unique names, optional SHA256 digest |
| POST /v2/users/login/ | Current API: Create an authentication token | Existing username/PAT only; no password or OTP prompt, no refresh loop |
| DELETE /v2/repositories/{namespace}/{repository}/ | [Docker hub-tool vendor source, immutable revision](https://github.com/docker/hub-tool/blob/74bc560cdbc56bd232c49033c2f37194b7b40e62/pkg/hub/repositories.go) `RemoveRepository` | Explicit local operator approval; one request, no retry; scoped readback |

Docker archived hub-tool in May 2026. The current published Hub API reference
does not advertise repository deletion. Consequently DELETE is a legacy
compatibility capability and requires a real authenticated acceptance before
claiming it works for an account. Do not substitute a different endpoint, delete
only tags and claim the repository is gone, or retry after an ambiguous response.
The documented official Web fallback is
[Settings → Delete repository](https://docs.docker.com/docker-hub/repos/delete/).

HTTP 401/403 is auth_required, 429 is rate_limited; redirects, malformed JSON,
identity mismatch, incomplete pagination, and changed state fail closed. Reads
do not retry either; users may explicitly start a fresh observation. This is not
the broader retry/challenge/OCI inventory acceptance.

Protocol, manifest and plan contracts reuse zixcel/cli revision
f03584c875069187339f164b1c2d684e98b00bc0. No arbitrary API specification can enable
another operation. Dependency/contract updates require source review, tests,
owner CI and a new accepted package; no remote schema is executed dynamically.
