# Ctrl-Migration HTTP Emission Target Brief

**Status**: READY-FOR-DISPATCH once the Emission-Targets Mgr exists.

**Authority**: Ctrl-Migration project plan §4 and §6 from PR #2775.

## Output

Author the first HTTP server emission-target design and substrate slice for ctrl replacement:

- `dsl/extdeps/http/server.dag` or a narrower first slice if the existing extdeps layer requires a different path.
- Route declaration carrier.
- Request body/query/path parameter carriers.
- Response status/body carriers.
- Handler binding shape that can point at a `.dag` service operation.

## Scope

This target enables generated ctrl HTTP handlers; it does not need to replace SQL persistence, audit events, browser automation, or cron in the same PR.

## Required Reuse Audit

Inspect before adding carriers:

- `dsl/extdeps/transports/rest.dag`
- `dsl/std/http_path.dag`
- `dsl/extdeps/github/*.dag`
- `src/v3/std/services.dag`
- `docs/lane4-completion.md`

## Acceptance Gates

1. Route shape composes from existing REST/path/service primitives where possible.
2. Every enum/sum with at least two variants has a Practice 4 receipt.
3. The target can express `POST /api/internal-work-items` and `POST /api/nodes/:id/declare` as examples without hardcoding ctrl-specific routes.
4. The brief names the first consumer subsystem, expected to be work-item API or inbox delivery.
5. No cut-over claim is made before a generated handler passes parity.

