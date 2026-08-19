# `materialization_carriers.dag` — Rows 2 & 3 root-cause packets

Measured against `origin/main` HEAD `a2b33f577a48b13e305440bae41d90406e8b4b7d` (rustc build log),
re-checked for drift against `073bbd4e08a40a01082c61896a2f2806d683c6c2` (current `origin/main` at
doc-write time — `git log a2b33f577a..073bbd4e08` touches none of `src/v1/05_emit_rust.dag`,
`src/v1/04_infer.dag`, `dag/std/cache_interface.dag`, `dag/std/cache_identity.dag`,
`dag/extdeps/uri.dag`, or `src/v2/compiler/materialization_carriers.dag`, so the measurement below
is unaffected by the advance). PR #8528 (branch `session/swift-moth-294`) is confirmed **OPEN,
unmerged** (`gh pr view 8528 --json state,mergedAt` → `{"state":"OPEN","mergedAt":null}`), so Row 3
carries all 5 of its original sites on current `main`, not the post-#8528 3.

**Scope of this doc: root-cause only.** No repair lands here. `src/v1/05_emit_rust.dag` and
`src/v1/04_infer.dag` were read-only for this investigation — neither file is touched by this
branch. See "What was NOT done" at the end of each packet.

## Measurement method

- Worktree `/tmp/gunbc-main-measure` pinned to `origin/main a2b33f577a48b13e305440bae41d90406e8b4b7d`.
- Built `gunbc`/`cssl_assemble` release binaries from that worktree.
- Ran the repo's single-authority probe, `docs/probes/curated_cargo_probe_one.sh
  src/v2/compiler/materialization_carriers.dag`, with `CSSL_STD_SEED_LINK=1` and
  `PROBE_KEEP_LOG_DIR=/tmp/mc_out` — plain-text `cargo build` log, not JSON (the script is the
  existing single authority for this probe per DESIGN §3; prior board docs used the same format).
  Total: 46 errors (`E0308:15 E0277:13 E0599:5 E0425:3 E0422:2 E0369:2 E0310:2 E0560:1 E0282:1
  unreachable_pattern:2`) — down from the prior rebaseline's 51 because #8539 (Row 1b) landed on
  `main` since; Row 2's 6 `E0308` sites and Row 3's 5 sites (2×`E0422` + 3×`E0425`) are unchanged.
- Ran `gunbc compile --target rust --output-dir /tmp/mc_emit_out` (raw, per-module, **not**
  `cssl_assemble`d) against the same closure to inspect per-module emitted Rust source directly,
  independent of `cssl_assemble`'s renumbering — this is how the differential controls below were
  found (a module can be grepped for a working sibling case without re-deriving cargo's line
  numbers).
- Every function cited below was read from `origin/main` at the pinned SHA in the measurement
  worktree; citations are module + function symbol per DESIGN §3, with the measured line as a
  convenience only.

---

## Packet 1 — Row 2, "Optional carrier fork" (6 `E0308` sites)

The board's single-row framing does **not** correspond to a single mechanism. Verified by
execution and code reading, the 6 sites split into three independent sub-mechanisms — same
rigor as the earlier 1a/1b Clone-bound split (`materialization_carriers_rebaseline_2026-08-19.md`).

| sub-row | sites | mechanism |
|---|---|---|
| 2a | `std_cache_interface.rs:638` | bare zero-field-variant identifier collides with an unrelated domain enum's same-named variant |
| 2b | `std_cache_interface.rs:695,699,703` | declared non-optional return type, body ends in cardinality-optional `.first()` |
| 2c | `extdeps_uri.rs:752,756` | same class as 2a, different specific collision (kernel `Optional`'s `Absent` vs. a domain enum's `Absent`) |

### 2a — `std_cache_interface.rs:638`

Authored source, `dag/std/cache_interface.dag:151`:

```
type AuthScope = None | FilesystemPerms | ApiKey | NetworkAcl
```

`AuthScope` is a closed coproduct with a zero-field variant literally named `None`. Every
zero-field variant of a closed coproduct is emitted as its own Rust unit struct
(confirmed in raw emission, `/tmp/mc_emit_out/src/std_cache_interface.rs:782`:
`pub struct None;`).

Elsewhere in the same module, `dag/std/cache_interface.dag:378-391` declares a genuinely
cardinality-optional field and constructs it with the native-optional absence literal:

```
type CacheReachCandidate { layer: CacheInterfaceId? ... }
fn cache_reach_candidate_recompute() -> CacheReachCandidate {
  CacheReachCandidate { layer: none ... }
}
```

Raw emission of the struct field itself is correct (`/tmp/mc_emit_out/src/std_cache_interface.rs:630`:
`pub layer: Option<CacheInterfaceId>,`), but the field's initializer at line 638 emits as
`layer: None,` — a bare, unqualified `None` that, because `pub struct None;` is declared in the
**same Rust module**, resolves to the domain unit struct instead of `std::option::Option::None`.

**Positive control, same module:** `/tmp/mc_emit_out/src/std_cache_interface.rs:650` emits
`layer: Some(id.clone()),` for a different `CacheReachCandidate` construction — i.e. the
`T?`→`Option<T>` mapping and `Some`-rendering work correctly elsewhere in this exact file. The
defect is specifically the **bare-identifier resolution of the absence literal**, not a systemic
break of cardinality-optional-to-`Option` lowering.

### 2b — `std_cache_interface.rs:695,699,703`

Authored source, `dag/std/cache_interface.dag:488-494` (and the sibling `cache_facts_for_id` for
site `:703`, same shape):

```
fn cache_layer_plan_primary(plan: CacheLayerPlan) -> CacheInterfaceId {
  plan.layers.first()
}
fn cache_layer_plan_fallback(plan: CacheLayerPlan) -> CacheInterfaceId {
  plan.layers.skip(n: 1).first()
}
```

`List<T>.first()` is cardinality-optional (returns `T?`), but each function's declared return type
is the non-optional `CacheInterfaceId`. `gunbc compile` on this closure reports **0 blocking
errors** for these three functions — the mismatch is caught only by rustc downstream, at the
generated-Rust boundary. This is evidence of a `.dag`-level typechecking gap: a declared Required
return type is not being checked against a body whose tail expression is cardinality-optional.

**Not traced further:** whether `04_infer.dag`/`04_types.dag` has *no* check here, or has a check
that is bypassed for this specific pattern (method-chain tail `.first()` vs. a plain field read),
was not determined — reading the return-type-vs-body-cardinality check in `04_infer.dag` was out
of scope for this read-only investigation (see "what was NOT done" below). This sub-row's DFS
position is therefore stated as: decided upstream of the emitter, inside `04_infer.dag`'s
handling of declared-return-cardinality vs. inferred-body-cardinality — **exact function
undetermined**.

### 2c — `extdeps_uri.rs:752,756`

Authored source, `dag/extdeps/uri.dag:447-463`, correctly uses the **modeled kernel** `Optional<T>`
coproduct (`Present`/`Absent`), not a domain enum:

```
fn uri_percent_encode_outcomes_first_refusal(
  outcomes: List<UriPercentEncodeScalarOutcome>,
) -> Optional<UriPercentEncodeRefusalCause> {
  fold(
    outcomes,
    init: Absent,
    f: fn(acc, outcome) {
      match acc {
        Present { value: _ } => acc
        Absent =>
          match outcome {
            UriPercentEncodeScalarRefused { cause: cause } => Present { value: cause }
            UriPercentEncodeScalarEncoded { wire: _ } => Absent
          }
      }
    },
  )
}
```

rustc's own diagnostics for both sites:

```
error[E0308]: mismatched types
   --> src/extdeps_uri.rs:752:43
752 | ...fold(Rc::new(NamedEdgeTargetLookup::Absent), |acc: Option<Rc<UriPercentEncodeRefusalCause>>, ...
    |         expected `Option<Rc<UriPercentEncodeRefusalCause>>`, found `Rc<NamedEdgeTargetLookup>`

error[E0308]: `match` arms have incompatible types
   --> src/extdeps_uri.rs:756:85
756 | ... UriPercentEncodeScalarEncoded { .. } => Rc::new(NamedEdgeTargetLookup::Absent),
    |     expected `Option<Rc<UriPercentEncodeRefusalCause>>`, found `Rc<NamedEdgeTargetLookup>`
```

Both sites are the two bare `Absent` **expression** occurrences in the source above (`fold`'s
`init:` argument, and the inner `match`'s `UriPercentEncodeScalarEncoded` arm) — `NamedEdgeTargetLookup`
is an unrelated coproduct elsewhere in the closure that happens to also declare a variant named
`Absent`. Same class as 2a (bare zero-field-variant identifier resolves to the wrong parent), but
this time in **expression** position rather than in a struct-field initializer, and the emitter
does have dedicated machinery for exactly this resolution:

- `emit_rust_expr_record_lit` (`src/v1/05_emit_rust.dag:7445`) is the renderer for any zero/N-field
  variant-construction expression, including a bare `Absent`. When no explicit `parent_enum` is
  attached to the node, it calls `contextual_variant_parent` (`src/v1/05_emit_rust.dag:7609`,
  called at `:7470`).
- `contextual_variant_parent` → `contextual_variant_parent_absent`
  (`src/v1/05_emit_rust.dag:7601`):
  ```
  fn contextual_variant_parent_absent(variant_name, resolved_type, emit_info, source_indices) -> String? {
    match resolved_type.return_cardinality {
      CardOptional =>
        if is_optional_variant_name(name: variant_name) { Present { value: "Optional" } }
        else { contextual_variant_parent_from_type_name(...) }
      _ => contextual_variant_parent_from_type_name(...)
    }
  }
  ```
  This function is logically correct **given its inputs**: when `resolved_type.return_cardinality
  == CardOptional`, it deliberately special-cases `Present`/`Absent`/`Some`/`None` to the kernel
  `Optional` parent (this is exactly the `is_optional_variant_name`/`is_optional_like_parent_name`
  machinery at `src/v1/05_emit_rust.dag:6408-6414`, sibling to the pattern-matching analogue
  `pattern_parent_enum`/`unique_variant_parent` at `:6384-6406`, which is a **different** call
  path used for `match`-pattern rendering, not this expression path — the two are structurally
  parallel but distinct functions). If it took the `CardOptional` branch here, the output would
  have correctly been `Optional::Absent` → `None`.
  Since the observed output is `NamedEdgeTargetLookup::Absent`, `resolved_type.return_cardinality`
  was **not** `CardOptional` for this node — execution fell to the `_` arm →
  `contextual_variant_parent_from_type_name` (`:7592`), which does a direct name lookup: it
  requires `resolved_type`'s own authored name to itself be an enum declaring a variant named
  `Absent` (`variant_belongs_to_enum(... enum_name: rt_name)`). For this to have returned
  `"NamedEdgeTargetLookup"`, `resolved_type`'s authored name at this node must itself have
  resolved to `NamedEdgeTargetLookup`.

**Root cause, stated precisely:** `contextual_variant_parent_absent` behaves correctly for the
inputs it is given; the defect is that the **`resolved_type` fed into it for this `Absent` node is
wrong** — it is not the kernel `Optional<UriPercentEncodeRefusalCause>` (or a cardinality-optional
type) that the `fold` call's own return type and the enclosing function's declared return type
both require, but the unrelated `NamedEdgeTargetLookup`. That resolution happens in
`04_infer.dag`, upstream of the emitter. `init: Absent` carries no local type annotation — its
type must be inferred either by unifying against `fold`'s generic accumulator-type parameter, or
by some fallback when no such unification succeeds. `infer_record_lit` (`src/v1/04_infer.dag:5027`)
takes an `expected: Node?` parameter specifically for this purpose; `infer_record_lit_structural`
(`:5043`) is its structural continuation. **Not confirmed by direct instrumentation:** whether
`fold`'s generic signature fails to propagate an `expected` type for the `init:` argument (leaving
`infer_record_lit`'s `expected` at `none`), and whether some fallback inside
`infer_record_lit`/`infer_record_lit_structural` then resolves the bare `Absent` identifier by a
global "any enum in the closure declaring this variant name" search analogous to
`unique_variant_parent` at the type layer — that is the leading hypothesis given the observed
output and the parallel structure of the two known corpus-wide-search functions, but it was not
verified by reading `infer_record_lit_structural`'s body or by instrumenting the actual `expected`
value at this call site.

### What was NOT done (Packet 1)

- `src/v1/05_emit_rust.dag` and `src/v1/04_infer.dag` were **not edited**.
- 2b's exact `.dag`-level check (or absence of one) for return-cardinality vs. body-cardinality
  was not traced to a specific function.
- 2c's upstream defect in `04_infer.dag` (the `expected`-type propagation into `fold`'s `init:`
  argument, and whatever fallback resolves an un-expected bare `Absent`) was not traced past the
  `infer_record_lit`/`infer_record_lit_structural` entry points — no instrumentation was added, no
  print/log was inserted, and the body of `infer_record_lit_structural` was not read line-by-line.
- No repro fixture was added; the existing corpus sites already discriminate the defect cleanly
  and reproducibly via `curated_cargo_probe_one.sh`, so a synthetic fixture was judged unnecessary
  for this doc's purpose (root-cause narration, not regression coverage).
- Whether 2a and 2c share a **single** fix (e.g., a corpus-wide rule that a bare zero-field-variant
  reference must prefer the kernel `Optional`/structurally-unique interpretation before falling
  back to a name search) or need independent fixes was not decided — this is exactly the kind of
  question a fix's author should re-verify by execution before choosing an approach, not something
  this doc asserts.

---

## Packet 2 — Row 3 residual, 3 `E0425` `NonEmptyStr` sites

Confirmed unmerged (#8528 open) — all 3 sites still present, unchanged location, on current `main`:

```
error[E0425]: cannot find value `NonEmptyStr` in this scope
   --> src/v2_compiler_materialization_carriers.rs:141
   --> src/v2_compiler_materialization_carriers.rs:145
   --> src/v2_compiler_materialization_carriers.rs:149
```

(#8528's own fix, landed at `src/v1/05_emit_rust.dag`'s `reference_derived_use_lines` via a new
`collect_anonymous_record_lit_heads` attestation arm, addressed a **different** 2-site `E0422`
sub-mechanism in the same row and explicitly deferred these 3.)

### Authored source

`src/v2/compiler/materialization_carriers.dag:103-112`:

```
fn parse_table_memo_provider_id() -> CacheInterfaceId {
  extdeps.realization.parse_table_memo.parse_table_memo_id
}
fn compile_stage_memo_provider_id() -> CacheInterfaceId {
  extdeps.realization.compile_stage_memo.compile_stage_memo_id
}
fn parse_table_memo_artifact_kind() -> ArtifactKindId {
  parse_table_artifact_kind
}
```

`CacheInterfaceId` and `ArtifactKindId` are zero-param closed brand aliases,
`dag/cache_identity.dag:8-9`:

```
type CacheInterfaceId = NonEmptyStr where brand("CacheInterfaceId")
type ArtifactKindId = NonEmptyStr where brand("ArtifactKindId")
```

Raw emission (`/tmp/mc_emit_out/src/v2_compiler_materialization_carriers.rs:141-149`):

```rust
pub fn parse_table_memo_provider_id() -> NonEmptyStr {
    crate::extdeps_realization_parse_table_memo::parse_table_memo_id()
}
pub fn compile_stage_memo_provider_id() -> NonEmptyStr {
    crate::extdeps_realization_compile_stage_memo::compile_stage_memo_id()
}
pub fn parse_table_memo_artifact_kind() -> NonEmptyStr {
    parse_table_artifact_kind()
}
```

The declared alias leaf `CacheInterfaceId` is lost; the renderer emits the alias's *base* type
name `NonEmptyStr` instead — which is not a real Rust item in this crate (no `use` brings a bare
`NonEmptyStr` into scope; the type alias is exported as `CacheInterfaceId`), hence `E0425`.

### Decisive differential: a working sibling in the same closure

`/tmp/mc_emit_out/src/extdeps_realization_parse_table_memo.rs:70` and
`extdeps_realization_compile_stage_memo.rs:49` — the **callees** referenced by the broken
functions above — render correctly:

```rust
pub fn parse_table_memo_id() -> CacheInterfaceId { ... }
pub fn compile_stage_memo_id() -> CacheInterfaceId { ... }
```

Same type (`CacheInterfaceId`), same zero-param brand alias, same closure, same emitter run —
one instance renders the alias leaf, the other does not. The only structural difference between
the broken and working functions is the **body shape**: the working functions build an ordinary
value (a record literal / normal expression). The broken functions' bodies are **bare, paren-free,
cross-module function references** — `extdeps.realization.parse_table_memo.parse_table_memo_id`
with no `()` in the `.dag` source (point-free style; emission adds the `()` to actually invoke it:
`crate::extdeps_realization_parse_table_memo::parse_table_memo_id()`).

### Render chain and where the leaf is lost

- `emit_fn_def` (`src/v1/05_emit_rust.dag:5865`, called at `:5058` with
  `inferred: resolved_type(n: item)`) feeds the function definition's own **inferred** field —
  not its raw authored return-type annotation node — forward to `emit_inferred`
  (`:6207-6248`) → `render_rust_fn_sig_type` (`:1125`).
- `resolved_type(n)` (`src/v1/04_types.dag:55-60`) reads `n.inferred` (`Resolved{node: rt} => rt`)
  — i.e. whatever `04_infer.dag` decided the function's return type resolves to, computed during
  inference of the function definition, not the declaration's authored text.
- `render_rust_fn_sig_type` preserves the authored alias leaf only when
  `closed_alias_peels_zero_param(env, n) && rust_fn_sig_preserves_authored_alias_leaf(name,
  decl_file)` (`:1139-1140`). Both conjuncts were checked independently:
  - `rust_fn_sig_preserves_authored_alias_leaf("CacheInterfaceId", decl_file)`
    (`:1104-1114`) evaluates to `true` by direct trace of its four branches: `CacheInterfaceId`
    is not a kernel type (`dag/std/types.dag:5-15`), not opaque-kernel-eligible
    (`rust_opaque_kernel_alias_type_eligible`, `:418-428`, only `Json`/`Bytes`/`Symbol`), not
    `"Nat"`, and `rust_seed_host_numeric_alias` (`:460-466`) returns `none` for it. **This guard
    is not the defect** — if reached with the right node it produces the correct answer, which is
    exactly why the sibling functions with an ordinary body render correctly.
  - `closed_alias_peels_zero_param(env, n)` depends on `closed_alias_peel_verdict(env, n)`
    (`:730-766`), which resolves via `lookup_type_for(env, n)` (keyed on `n.ident` if present) or
    falls back to `lookup_type_by_name(env, authored_name_at(n))`
    (`src/v1/04_env.dag:924-942`, `:862-875`). This is the seam whose behavior differs between the
    working and broken bodies: for the working functions, `n` (the return-type node fed forward)
    still authored-names `CacheInterfaceId` and the lookup finds a zero-param binding
    (`ClosedAliasPeelZeroParam`). For the broken functions, the differential evidence (the emitted
    leaf is already `NonEmptyStr`, not merely un-wrapped-but-labeled) indicates `n`'s own
    resolved/authored identity has **already lost the `CacheInterfaceId` name before reaching this
    guard** — i.e. `resolved_type(n: item)` for these three functions already carries the peeled
    base type, not the alias.

**Root cause, stated precisely:** this is not a bypass of `render_rust_fn_sig_type`'s
leaf-preservation guard (that guard is correct and reachable-but-not-reached), and it is not a
uniform "closed aliases never survive" defect (the two callees prove aliases render correctly by
default). The defect is specific to **how a function's declared return-type annotation is unified
against a body that is itself a bare reference to another function**, in `04_infer.dag`: something
in that unification (most plausibly, resolving the *referenced* function's own return type to
build the calling function's inferred type) discards the calling function's authored
`CacheInterfaceId` annotation and substitutes the structurally-peeled base type. Note the callee
itself (`parse_table_memo_id`) renders `CacheInterfaceId` correctly in isolation — so the loss
happens specifically in the **caller's** inference step when its body is a point-free reference,
not in resolving the callee's own signature.

### What a fix needs to preserve, and why

A fix must ensure that when a zero-arg function's declared return type is a closed zero-param
alias (`CacheInterfaceId`, `ArtifactKindId`, …) and its body is a bare reference to another
function whose own return type unifies with the alias's base type, the **caller's declared alias
identity wins** over the referenced function's resolved type — the same rule
`rust_fn_sig_preserves_authored_alias_leaf` already encodes at the emitter layer, but it needs to
be preserved through inference so that `resolved_type(n: item)` still carries `CacheInterfaceId`,
not `NonEmptyStr`, by the time it reaches the emitter. Two shapes of fix are possible and were
**not chosen between** here:

1. Fix in `04_infer.dag`: when unifying a function's authored return-type annotation against its
   body's inferred type, prefer the annotation's identity (the alias) over the body's resolved
   identity when the two are structurally compatible (alias vs. its own base) — this is the
   general rule and would also cover any future point-free case, not just these three sites.
2. Fix narrowly at these three call sites' `.dag` source, by adding explicit parens/an
   intermediate `let` binding or an explicit cast back to the alias — but this treats the symptom
   per-site rather than the general point-free/alias-unification gap, and (per DESIGN §6's
   "bare minimum cost" standing rule) a proven cost-shape/correctness defect in a general
   mechanism should be fixed at the mechanism, not patched per occurrence — especially since this
   exact point-free pattern (`fn wrapper() -> Alias { module.qualified.other_fn }`) is a normal,
   recurring authoring idiom in this corpus (both broken functions here use it, and it is not a
   one-off).

Approach 1 is preferable on these grounds, but **this doc does not implement either** — that
decision, and the exact unification-site fix, is out of scope here and belongs to whoever owns
`04_infer.dag`'s return-type unification (a hot file per this session's explicit scope
constraint).

### What was NOT done (Packet 2)

- `src/v1/04_infer.dag` was **not read past locating candidate seams** (`infer_record_lit`,
  `infer_record_lit_structural`, the general shape of return-type unification) — the exact
  function that unifies a function's declared return annotation against a point-free body's
  inferred type was **not located by symbol**. This doc's Packet-2 root cause is therefore stated
  at the "which layer, and what the fix must preserve" level, backed by a decisive differential
  (working callee vs. broken caller, same alias, same closure), not at the level of a single named
  function inside `04_infer.dag`.
- `src/v1/04_infer.dag` and `src/v1/05_emit_rust.dag` were **not edited**.
- No repro fixture was added, for the same reason as Packet 1 — the existing 3 sites already
  discriminate cleanly and reproducibly.
- The `parse_table_memo_artifact_kind` site (`ArtifactKindId`, the third E0425) was not separately
  re-derived in as much detail as the two `CacheInterfaceId` sites — it shares the identical
  point-free-body shape (`parse_table_artifact_kind`, no parens) and identical emitted symptom, so
  it is treated as the same mechanism rather than independently re-verified line-by-line; this is
  flagged rather than silently assumed.
