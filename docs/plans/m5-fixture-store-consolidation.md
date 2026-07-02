# M5 — Fixture-store consolidation onto one Realization kernel

Node: `node://adhoc-3efc6449-ec5` · MODEL-FIRST SPIKE (escalate before load-bearing). Anchor: DESIGN.md §2 (one concept, every scale — Realization pattern), §3 (single authority, interface↔transport split), §5 (fail-closed). `ROADMAP.md` §2 *Minimal work — caching by realization*.

## 1. The fork

Two seeds each hand-roll a fixture store that answers the *same* question — "given an operation and its inputs, return the recorded response, fail-closed if absent/stale":

| axis | v1 `RecordedFixture` (`src/v1/stage0/src/recorded_fixture.rs`) | v2 `EffectIoFixtureStore` (`src/v2/extdeps/runtimes/v2_effect_io_pure.dag`) |
| --- | --- | --- |
| key | `(operation, input_hash = content_hash(inputs))` | `Symbol` locator — degenerate `^effect_io_pure_default_locator` pin (ignores the `RuntimeValue` arg) |
| key class | **ContentAddressedByValue** | **HandAuthoredString** (fail-open: distinct args silently alias) |
| value | `response: serde_json::Value` (faithful `Value` serialization) | `RuntimeValue` |
| placement | on-disk `{root}/{op_slug}/{hash}.json` — **PerHostFilesystem** | in-process `Map<Symbol, RuntimeValue>` — **InProcess** |
| read verify | stores `inputs`, verifies on lookup (collision safety) — **content_verified_on_read** | none |
| fail-closed | `Missing` / `Stale` / `InputMismatch` / `Expired` / `ResponseDrift` (purity oracle) | miss → rejected; backend mismatch → rejected |
| eviction | `Ttl` (FIXTURE_FRESHNESS_SECS = 30d) | `Never` (drops with eval) |

They are **not two concepts**. They are ONE artifact kind — *a wet-captured effect response, keyed by the content hash of the operation inputs* — realized at two placements. v1 is the mature content-addressed handler; v2 is an immature in-process handler whose key derivation is an explicitly-gated fail-open stub (`effect_io_pure_locator_identity`, dissolve-on: "derive fixture keys from RuntimeValue→Node reification").

## 2. The kernel already exists — `std.cache_interface`

This is not a new kernel. DESIGN/ROADMAP already declare the content-addressed cache kernel, and the fixture store maps onto it cleanly:

- `ArtifactIdentity<T> { subject_digest: ContentHash, artifact_kind }` **is** the `(operation, input_hash)` key — `subject_digest = content_hash(inputs)`, one new `artifact_kind = "hermetic_fixture"`.
- `CacheLookupResult<T> = Hit{receipt} | Miss | RejectedHit{reason}` **is** the lookup outcome.
- `CacheRejectReason` already covers most of v1's fail-closed variants (mapping below).
- `CacheInterfaceFacts` (locality / key_derivation / lookup·write·miss semantics / eviction / `content_verified_on_read`) **is** the per-handler descriptor — exactly the shape `parse_table_memo.dag` and `resolved_graph.dag` already cite in `extdeps/realization/`.

So the fixture store is a *missing pair of `CacheInterfaceFacts` rows + a shared lookup fold*, not new vocabulary. This spike lands the rows; §5 escalates the fold.

### Fail-closed reason mapping (the §5 win)

| v1 `FixtureError` | kernel surface |
| --- | --- |
| `Missing` | `Miss` |
| `Stale` (operation / input_hash mismatch) | `RejectedHit{ SubjectDigestMismatch }` |
| `InputMismatch` (stored inputs ≠ current — collision) | `RejectedHit{ ContentDigestMismatch }` |
| `InvalidDigest` | `RejectedHit{ BackendKeyMalformed }` |
| `Io` / `Json` / `ClockUnavailable` | `RejectedHit{ BackendUnavailable }` |
| `Expired` (TTL) | **GAP** — no expiry variant in `CacheRejectReason`; expiry is `eviction: Ttl` but v1 treats it as a *loud refusal*, not a silent evicted-Miss. Needs a `FreshnessExpired` reject reason (or an explicit policy that TTL-expiry is fail-closed). |
| `ResponseDrift` (re-record ≠ stored for same key) | **GAP (write path)** — this is the `WriteSemantics = WriteOnce` purity-oracle violation; the kernel has no write-outcome coproduct yet. |
| `DeserializationMismatch` / `UnknownTag` / `UnreplayableValue` | **GAP (transport faithfulness)** — value (de)serialization faithfulness of the `StructuredArtifact` transport encoding; handler-internal, but worth a typed `transport_encoding`-level reason rather than a stringly reason. |

These three gaps are the model work the load-bearing follow-up adds to `std.cache_interface` **before** the folds are rewired (so the rewire never has to fall back to a stringly error).

## 3. What this spike lands (NON-load-bearing — additive only)

1. `dag/extdeps/realization/hermetic_fixture.dag` — the two cited `CacheInterfaceCatalogFacts` rows (`hermetic_fixture_file_facts`, `hermetic_fixture_in_process_facts`), parallel to `parse_table_memo.dag` / `resolved_graph.dag`. They describe each handler **as it is today** — the fork shows up as a key_derivation divergence (`ContentAddressedByValue` + `content_verified_on_read` vs `HandAuthoredString` + not).
2. `dag/std/cache_identity.dag` — one additive `hermetic_fixture_artifact_kind` data decl (the shared artifact-kind both handlers realize). No fold touched (the only `match` over `CacheInterfaceProduct` is untouched).
3. `dag/test/claim/hermetic_fixture_realization_test.dag` — executing witness:
  - file handler is the content-addressed reference;
  - in-process handler is the fork **today** (RED-on-fabricated-convergence — fails closed if the row is marked converged before the fold lands);
  - both share kernel invariants (same value-shape, `MissIsDiagnostic` fail-closed);
  - one `ArtifactIdentity` routes to **both** backends — §2 "one spec → N handlers", executed.

  Verified green by execution; perturbing the in-process row to `ContentAddressedByValue` / `content_verified_on_read: true` turns it RED (discriminating).

No existing consumer changes; no Rust touched; no v2 fold touched.

## 4. The convergence target (what closes the fork)

The shared key is `subject_digest = content_hash(inputs)`. v1's interim `content_hash_service_inputs` (value_hash limbs) and v2's symbol pin both dissolve onto **`v2.std.node` content_hash over the reified operation-input `Node`** — one digest authority, used by both handlers. When v2's `effect_io_pure_locator_identity` derives the key from `RuntimeValue→Node` content_hash, the in-process row's two divergent fields flip and the fork is gone (the witness then expects the new state — its RED guard converts to the convergence assertion).

## 5. The load-bearing FOLD that changes — ESCALATE before touching

The rewire that makes both stores *dispatch through* the kernel (not just be described by it):

- **v1**: `eval_service_call` in `src/v1/stage0/src/v1_interpreter.rs` (the M4 hermetic-realization fold) calls `RecordedFixtureStore::lookup → Result<RecordedFixture, FixtureError>` and matches the bespoke error enum. Rewire: the v1 handler returns `CacheLookupResult<Value>` and the fold consumes the shared coproduct. `recorded_fixture.rs` `FixtureError` collapses onto `CacheRejectReason` (+ the three new variants from §2).
- **v2**: the effect dispatch in `src/v2/compiler/05_eval.dag` / `v2_effect_io_pure.dag::effect_io_pure_store_lookup` returns `Outcome<RuntimeValue>`. Rewire: lookup returns `CacheLookupResult<RuntimeValue>`, eval maps it to `Outcome`.

Both are load-bearing seed/substrate folds (`v1_interpreter.rs`, `recorded_fixture.rs`, `05_eval.dag`, `v2_effect_io_*`). Per the project spirit and the M5 brief, the design is escalated to the operator (via parent `calm-wren-408`) **before** any of this is implemented. Sequencing:

1. (this PR) cited rows + artifact-kind + witness — non-load-bearing, merges to anchor the model.
2. (escalated) add the three `std.cache_interface` gap surfaces (`FreshnessExpired`, write-outcome coproduct, transport-faithfulness reason) — model-first, additive.
3. (escalated, signed-off) rewire v1 fold → `CacheLookupResult`; collapse `FixtureError`.
4. (escalated, signed-off) v2 value-derived key + rewire v2 fold; in-process row converges; HTTP host door + emitted-host DryRunMode ride this seam (the M5 brief's remaining scope).

## Dissolution trigger (DESIGN §6)

Delete this doc when the v1 and v2 fixture stores both dispatch through `std.cache_interface` (the §5 fold rewired, `FixtureError` collapsed onto `CacheRejectReason` with the `FreshnessExpired` / write-outcome / transport-faithfulness gaps closed) and the in-process row has converged to `ContentAddressedByValue` + `content_verified_on_read` — at which point "one fixture concept, two handlers" is a witnessed property (the convergence assertion green by execution) and this tracker is redundant.
