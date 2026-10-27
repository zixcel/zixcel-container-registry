# Operation guidance

Requires Zix 0.7.0 or later. The native `docker` extension 0.3.0 declares every registered operation in its source-owned `api/guidance.json` (under the native CLI crate when this repository has multiple crates). Metadata inspection reads no credentials and sends no provider requests.

```sh
zix capabilities --compact --format json
zix capabilities --namespace docker --compact --format json
zix describe docker --operation OPERATION_ID --compact --format json
```

Replace `OPERATION_ID` with an operation ID returned by the second command. Add `--argument NAME=VALUE` for known selectors and resource identifiers. Missing context is returned before credential resolution or remote execution. Use the returned literal argument vectors; do not invent command flags.

The compact view lists intent, keywords, effects, required arguments, credential audience and access requirements. The full view retains provenance and conditional next actions. Requirements describe prerequisites; they do not prove token validity, consent, role assignment or resource access. Credential storage never grants provider permissions.

Authentication errors require authentication inspection. Authorization errors require reviewing the exact resource, audience and permission named by the operation. Do not expand privileges automatically or repeat a denied request. Partial observations are not proof of absence. For an uncertain write outcome, inspect current state and reconcile the retained journal before making a new plan; never replay the write automatically.

`zix-provider/v2` replies carry bounded structured diagnostics. Legacy standalone or library interfaces, where present, retain their own documented contracts.
