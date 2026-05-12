# Ctrl Migration Emission Targets Phase 3: HTTP / SQL Extdeps Audit

**Date:** 2026-05-12
**Work item:** `adhoc-157c31e8-7c6`
**Scope:** classify HTTP and SQL emission-target candidates for the `ctrl/` to
`.dag` substrate migration.

## Finding

HTTP and SQL are external dependency transports, not compiler emission targets.
They should remain under `dsl/extdeps/transports/` and be consumed by service
or provider models. A future target-language emitter may render calls that use
these transports, but the transport facts themselves are provider/runtime facts,
not Rust/Python/Go-style language targets.

This follows the same placement rule used by the workflow runtime dispatch
canvas: provider artifacts stay provider-faithful, and gunbc-specific emission
choices live in gunbc-owned projection calls rather than on extdeps carriers.
It also follows `dsl/extdeps/extdeps.md`: every extdep module models an actual
external system and cites upstream specifications.

## HTTP

`dsl/extdeps/transports/rest.dag` already carries the HTTP/REST transport
surface. It is grounded in:

- RFC 9110 HTTP Semantics
- RFC 7235 HTTP Authentication
- RFC 3986 URI syntax

The current shape is sufficient as the Phase 3 audit target:

- `RestTransportConfig.base_url` is the endpoint authority.
- `auth_token` / `auth_header` represent protocol authentication placement.
- `headers` carries request header facts.
- `response_format` connects the transport to `std.serialization.WireFormat`.

No new `HttpEmissionTarget` carrier is warranted. HTTP is a transport consumed
by service declarations.

## SQL

SQL did not have a transport extdep at HEAD. This PR adds
`dsl/extdeps/transports/sql.dag` as the provider-neutral relational database
transport home.

The added shape is intentionally transport-level:

- `SqlStatementKind` names observable statement classes.
- `SqlParameterStyle` captures bind-marker syntax used by SQLite, PostgreSQL,
  and MySQL prepared statements.
- `SqlTransactionMode` records transaction-boundary expectations.
- `SqlTransportConfig` records connection identity, parameter style,
  transaction mode, and result wire format.

This does not model a specific database provider, SQL dialect grammar, or query
planner. Provider-specific SQL modules can import this transport and add
dialect facts later.

## Non-Targets

Do not add any of the following as emission targets for this slice:

- `HttpEmissionTarget`
- `SqlEmissionTarget`
- language-spec rows for HTTP or SQL
- compiler branches that treat HTTP or SQL like Rust/Python/Go

The valid next consumer shape is a service/provider declaration using
`transport rest` or `transport sql` and target-language emitters mechanically
rendering client code from those facts.

## Acceptance Receipt

- HTTP audited: existing `extdeps.transports.rest` is the authoritative home.
- SQL audited: new `extdeps.transports.sql` establishes the authoritative home.
- Parser receipts: `rest_transport_dag_compiles_cleanly` and
  `sql_transport_dag_compiles_cleanly` pin both transport extdeps as parseable
  substrate.
- Extdeps fidelity preserved: the new module cites upstream SQL/prepared
  statement sources and avoids gunbc-specific policy.
- Emission-target boundary preserved: neither HTTP nor SQL is introduced as a
  compiler emission target.
