# Ctrl-Migration Phase 3 HTTP SQL Audit Extdeps Brief

**Status**: ACTIVE scaffold for Emission-Targets Mgr (`node://adhoc-157c31e8-7c6`), 2026-05-12.

**Authority**: `docs/briefs/ctrl-migration-day1-manager-dispatch.md` assigns this lane ownership of:

- `dsl/extdeps/http/server.dag`
- `dsl/extdeps/sql/migration.dag`
- `dsl/extdeps/audit/event.dag`

## Scope

Phase 3 defines emission target contracts for ctrl subsystem projections:

- HTTP server routes and request/response binding shape.
- SQL migration scripts and ordered schema-change steps.
- Audit event records and actor/outcome facts.

These are target contracts only. They do not claim runtime cut-over, framework choice, database execution, or audit sink authority.

## Gates

- Keep all three files staged until a subsystem projection consumes the target.
- Do not mark a subsystem model authoritative until parity tests pass against current `ctrl/` TypeScript behavior.
- Preserve transport layering: `extdeps.transports.rest` and `extdeps.transports.sql` remain execution/config transport authorities; these files model generated artifact shape.
- SG-0/P5 receipt: `INVARIANTS.md` row `src/v3/compiler/tests/integration/extdeps_sql_transport_test.rs` is the single hand-Rust receipt for this PR; it cites `ROADMAP.md` § **Nine lanes** row `T-PB-B` / `pb_rust_tests_outside_residual_zero` and lists the five `compile_to_dag` probes plus dissolution into `.dag`-native parse/authority coverage or a generated test harness.

## First Worker Slice

HTTP server projection should consume `HttpServerEmissionTarget` for one low-risk ctrl subsystem and produce a parity receipt that compares:

- route method and path template,
- path/query/header/body input binding,
- response status and body kind.

SQL migration and audit event consumers should follow only after the first HTTP projection receipt lands.
