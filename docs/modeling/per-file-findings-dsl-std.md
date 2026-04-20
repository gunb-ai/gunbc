### dsl/std/

**types.dag** — 7.5/10
- M2: ~~GCP types duplicated (`ProjectId` vs `GcpProjectId`)~~ — DONE: renamed to `GcpProjectId`; deleted 5 dead types (GcpSecretId, GcpServiceAccountEmail, GcpSubjectToken, OidcAudience, OidcSubjectToken, WifAudience)
- M1: ~~`CloudSecretConfig` embeds policy defaults~~ — DONE: dead type deleted; operations define own typed inputs
- M2: ~~`ContentEncoding` may overlap with `encoding.dag`~~ — DONE: consolidated to `Encoding` in encoding.dag with BoundedLattice (meet/join); ContentEncoding deleted from types.dag; FileClassification moved to filesystem.dag

**encoding.dag** — 9/10 (foundation chain)
- Single authority for `Encoding` type with BoundedLattice (meet/join)
- ~~Reconcile with `ContentEncoding` in `types.dag`~~ — DONE: one definition only

**containers.dag** — 4/10
- Skeletal, no type definitions — either define container types or delete

**errors.dag** — 7/10 (after cleanup)
- Provider-specific shapes are spec-grounded (GitHub, GCP, Anthropic, OpenAI)
- Generic types removed (HttpErrorShape, AuthError, etc. were invented canonicalizations)

**resources.dag** — 7/10
- M1: `ResourceHandle.type` and `.resource_id` are strings — should be branded
- Good: opaque handles with capabilities, explicit I/O boundaries

**patterns.dag** — 8/10
- Incomplete: `retry` is a stub
- Good: compositional `ensure`, `upsert`, `transaction` patterns

**symbols.dag** — 8/10
- M4: `SymbolId` is a 35-variant flat enum — no structural grouping
- M5: `resolve_symbol` returns empty string on miss instead of erroring

**fidelity.dag** — 6.5/10
- M5: Wildcard `_ => Xl` in transport_depth — silent fallback
- Cost mappings lack justification (why 30s for Xs?)

**fermi.dag** — 6.5/10
- M7: Timeout data duplicated as both `data` and function body
- Good: ordinal pattern, composition via `fermi_max`

**render.dag** — 7.5/10
- Dead code: `RenderMode` enum never referenced
- Good: two-layer architecture, Fragment sum type

**filesystem.dag** — 8/10
- Good: layered tautology, exhaustive matching, no wildcards

**languages.dag** — 8/10
- Good: 13 faithful language models from real language specs

