# materialization_carriers: re-baseline on current main, board unchanged site-for-site (2026-08-19)

**Session:** `eager-ant-366` (measurement authority for this module, dashboard node
`adhoc-dfa0b3ca-c9f`, succeeds `witty-heron-413`/PR #8460, archived).
**Subject:** `src/v2/compiler/materialization_carriers.dag`, same instrument as #8460.

Nothing here is transcribed from the predecessor's document. Every number was produced by a run
made for this receipt, against current `main`.

## 1. Instrument (unchanged from #8460 §1 — single authority, not re-derived)

```
gunbc compile --source-root dag --source-root src/v2 \
  --entry src/v2/compiler/materialization_carriers.dag --target rust \
  --dependency-pool-index primary-precedence --output-dir <out>
cssl_assemble --out-dir <out> --entry-dag src/v2/compiler/materialization_carriers.dag --root .
cd <out> && cargo build --release --lib --message-format=json
```

`CSSL_STD_SEED_LINK=1`, no lane shim (raw cssl-assembled `lib.rs`), `Cargo.toml` rendered via
`docs/probes/lib/render_cssl_probe_lib_cargo_toml.sh` (the harness's own single authority for
that file — not a parallel heredoc). Counts are errors as rustc reports them in the JSON stream,
**at (code, primary span, message, children) grain** — never a keyword/text scan.

Binaries: `gunbc` and `cssl_assemble` built locally on this branch
(`CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin gunbc --bin cssl_assemble`)
immediately before this run, so the binaries and the measured tree agree.

## 2. Base

`main` at `72676cf0be992a6305d312d2e93ca7fc1fd7edef` (HEAD at measurement time) — the tip after
#8460, #8410, and #8417, plus three unrelated merges since #8460 (#8465 grammar sync tokens,
#8458 delete fleet-converge unit renderer, #8453 systemd unit_file; none touch
`materialization_carriers.dag`, its emitter files, or `dag/std/algebra.dag`/`cache_interface.dag`
per file history).

`gunbc compile` itself: 0 blocking errors, 187 advisory diagnostics (policy
`gunbc.compile_clean_diagnostic_policy`) — unchanged in kind from #8460, not a new defect class.

## 3. The count: **51, unmoved**

```
cargo build --release --lib --message-format=json  →  exit 101, 51 error diagnostics
E0277 16 · E0308 15 · E0599 9 · E0425 3 · E0422 2 · E0369 2 · unreachable_patterns 2 · E0560 1 · E0282 1
```

This is the same total #8460 §5 reported against `origin/main 2c65eeacf3` (the base that already
included #8410). **Stated explicitly per the instruction to pair every zero (here: zero movement)
with verification, not silence:** this is not just a total-count coincidence. Every one of the 51
diagnostics was matched by primary span + code against #8460 §7's board and §10.3's 1a/1b split,
and every one lands in the same row, at the same file:line:col, as before. Nothing merged since
#8460 touches this closure. The unmoved 51 is a real "nothing changed here" result, not an
instrument reading a stale base — confirmed by re-deriving the board from scratch below rather
than by trusting the total alone.

## 4. The board, restated against `72676cf0be`, row-for-row identical to #8460 §7/§10.3

| # | mechanism | pop | unchanged? |
|---|---|---:|---|
| 1a | Clone bound missing on generic **derived impls** | 14 | **corrected 2026-08-19, see §4a** — 10 E0277 (`im::Vector<T>` Debug/Serialize/Deserialize, `v2_std_algebra.rs:43,45,84,88`) + 2 E0369 (derived `PartialEq`, `v2_std_algebra.rs:45,88`) + 2 E0599 `CacheLookupResult<T>` (`std_cache_interface.rs:564,580`) |
| 1b | Clone bound missing on generated **fn/inherent-impl signatures** | 14 | **corrected 2026-08-19, see §4a** — 6 E0277 (`bind_outcome`/`resolve_probe` bounds, `v2_std_staging.rs`, `v2_compiler_materialization_carriers.rs`) + 6 E0599 (`no method clone for type parameter A`, same two files) + 2 E0308 (`v2_std_algebra.rs:118`, `v2_std_staging.rs:31`) |
| 2 | Optional carrier fork | 6 | yes — same 4 E0308 `std_cache_interface.rs:638,695,699,703` + 2 E0308 `extdeps_uri.rs:752,756` |
| 3 | Unsynthesized use-line (root K) | 5 | yes — same 2 E0422 `ProviderRetention` + 3 E0425 `NonEmptyStr` |
| 4 | Int literal into branded-string field | 4 | yes — same 4 E0308, `compile_stage_memo.rs:95,101`, `parse_table_memo.rs:116,122` |
| 5 | Type alias materialised twice | 3 | yes — same 3 E0308, `v2_std_node.rs:69,78,84` |
| 6 | Nested coproduct pattern flattened | 2 | yes — same 2 `unreachable_patterns`, `v2_std_node.rs:533,537` |
| 7 | ContentHash carrier vs `String` (T7) | 1 | yes — same 1 E0599 `partial_cmp`, `v2_std_node.rs:1334` — attributed to `calm-lynx-547`, **who is archived; see §5** |
| 8 | Record literal through shared-wrapped alias | 1 | yes — same 1 E0560, `std_verification.rs:22` |
| 9 | Type annotations needed | 1 | yes — same 1 E0282, `std_realization_measurement.rs:197` |

`14+14+6+5+4+3+2+1+1+1 = 51`. Every row's specimen file:line:col was re-checked against this
run's JSON, not assumed carried forward.

### 4a. 1a/1b split correction (flagged by `smart-ram-730`, 2026-08-19)

This section originally split 1a/1b as 12/16, disagreeing with #8460 §10.3's settled 14/14.
Settled by re-reading the JSON `message`/`children` text for the two disputed sites
(`std_cache_interface.rs:564,580`, both `E0599 CacheLookupResult<T> clone`), not by re-asserting
either total:

- Those two sites read *"the method `clone` exists for enum `CacheLookupResult<T>`, but its
  trait bounds were not satisfied"* / *"trait bound `T: Clone` was not satisfied"* — the
  **derived-impl-with-unsatisfiable-bound** shape (`#[derive(Clone)]` emits `impl<T: Clone> Clone
  for CacheLookupResult<T>`; calling `.clone()` on `T`-not-`Clone` fails through that generated
  impl). Same species as the `im::Vector<T>` Debug/Serialize/Deserialize/PartialEq sites in 1a —
  a derived trait implementation whose bound the emitter didn't add, not a fn signature.
- The genuine 1b E0599 sites (`v2_std_staging.rs`, `v2_compiler_materialization_carriers.rs`)
  read *"no method named `clone` found for type parameter `A` in the current scope"* — a bare
  generic fn/inherent-impl body calling `.clone()` on an unbounded type parameter, no derive
  involved. A structurally different mechanism from the `CacheLookupResult<T>` sites.

Verdict at time of writing: the two `CacheLookupResult<T>` sites belong in **1a**, not 1b. §4's
table above is corrected to 14/14, matching #8460 §10.3 exactly. This was treated as **arm (ii) —
a categorization slip in this session's re-derivation**, not genuine movement between bases
(consistent with §3's site-for-site match: nothing about *which* 51 sites exist changed, only which
row two of them were filed under).

**CONTESTED, 2026-08-19, by `smart-ram-730` — pending `deep-swift-570`'s executed answer.** The
reasoning above reads the derive as the defect: "a `derive(Clone)`-generated impl whose bound the
emitter didn't add." That is not what `derive(Clone)` emits on `CacheLookupResult<T>` — it emits
`impl<T: Clone> Clone for CacheLookupResult<T>`, a *conditional* bound that is present, not missing.
The candidate counter-reading: the defect is on the **caller** — `realize_route<T>` /
`classify_write<T>` are generic fns whose *emitted signatures* carry no `T: Clone`, so inside their
bodies the conditional impl doesn't apply and rustc reports "trait bounds were not satisfied" —
which would make these two sites the **1b mechanism (missing bound on a generated fn/inherent-impl
signature) wearing 1a's error text**, not 1a. Under that reading the derived-impl-vs-fn-signature
axis is a proxy that fails exactly at a call site inside a generic fn that touches a
conditionally-derived impl — the same shape the code-keyed (E0599-vs-E0599) partition failed on
earlier.

**The discriminating question, decidable by execution, not by re-reading text a third time:** does
adding `T: Clone` to `realize_route<T>`/`classify_write<T>`'s emitted signature clear both errors,
with no change to the `derive` on `CacheLookupResult<T>`? Yes → 1b, and this table needs a second
correction. No → 1a stands as corrected above. `deep-swift-570` holds these two sites as part of
their in-flight `emit_fn_def` repair and was asked to answer this directly by execution; **this
document is not being re-corrected a third time on prose alone — only on that executed answer.**

**Update, same day: a claim arrived, and it is being held rather than acted on.** `deep-swift-570`
reported their branch's `git diff main...HEAD -- src/v1/trait_derive_emit.dag` is empty — only
`emit_fn_def` and a new `trait_bound_witness.dag` decision core changed — and concluded the two
sites are 1b. `smart-ram-730` flagged, correctly, that this under-powers the question: an empty
diff on the derive-emission path proves proposition **A** (the repair touches only signatures), not
proposition **B** (a signature-only repair actually clears both `E0599`s). "The derive was never
touched" and "the derive needed touching and wasn't" are indistinguishable from a diff alone — only
running the fix and reading the result decides between them. **Holding this row at provisional,
still 1a per §4 above, until `deep-swift-570`'s CI run against `561bf1166b1` reports which of three
outcomes actually happened:** both `:564`/`:580` gone with nothing new → B holds, correct to 1b;
gone but a new error appears → the defect moved, neither classification is safe yet, needs its own
row; either site survives → B is false, 1a stands. No further correction lands on diff-scope or
message-phrasing reasoning alone — only on that executed result.

**The axis, stated explicitly per the ask that follows from this exchange:** "1a = derived impls,
1b = fn/inherent-impl signatures" is ambiguous precisely where a call site sits inside a generic fn
whose body invokes a conditionally-derived impl — both disputed rows sit there, and it is why the
same two sites have now been reclassified by error-text reading and contested by mechanism reading
in one day. The load-bearing question is not *which diagnostic phrasing appears* but **whose
emitted signature is missing the bound** — the derive's (1a) or the caller's generic fn (1b). Error
text is evidence toward that question, not the axis itself; the settlement above followed message
phrasing, and phrasing is exactly what this counter-reading shows can point at the wrong signature
when a conditional impl is involved. Until `deep-swift-570`'s executed answer lands, treat §4's
14/14 row assignment for `std_cache_interface.rs:564,580` as **provisional**, not settled — the
totals (14/14, 51 overall) are unaffected either way; only the row these two sites file under is
open.

**SETTLED, 2026-08-19, by execution — 1a stands.** `deep-swift-570` ran the actual build, both
against their pushed head `561bf1166b1` and separately against `origin/main`: both `:564`/`:580`
errors are byte-identical (text, line, column) on both sides — outcome three of the three named
above. Proposition B is false: a signature-only repair does not clear these errors, so the 1b
counter-reading is refuted by the same standard that would have confirmed it, and §4's 14/14 row
assignment (both sites in **1a**) is no longer provisional.

**Why, and a genuinely new finding — not folded into either row.** The emitted match scrutinees at
these two sites are `(*lookup.clone()).clone()` / `(*existing.clone()).clone()` — an `Rc` clone
followed by a redundant deep clone of the pointee, done purely to support a match. Neither a
missing derive bound (1a) nor a missing fn-signature bound (1b) is what a T:Clone-bound repair
would actually reach here; `deep-swift-570` reports their original signature-bound fix doesn't
even reach these two sites in practice, and `smart-ram-730` separately flagged that adding the
bound would be the wrong direction — a match by reference would need no `Clone` at all. So the
board keeps these two sites at **1a** (that assignment is what execution confirmed — unchanged
under a signature-only repair), but the *actual* repair, if and when one lands, likely won't be a
derive fix or a signature fix — it's plausibly a distinct **redundant-clone-in-match-lowering**
mechanism in the Rc-deref-clone emission path. Not reclassifying speculatively while this is
unsettled; flagging it here so a future landing doesn't get filed into 1a or 1b by row-proximity
when it's neither.

## 5. T7 / "99 E0308 sites" — resolved for `stern-fox-619`, restated against this base

Re-confirmed at `72676cf0be`: this module's **live T7 footprint is row 7 above — 1 site, and it
is E0599 (`partial_cmp`), not E0308.** #8410 already retired all 8 of this module's prior T7 E0308
sites (per #8460 §11.2, measured across `11254b04fc` = 8 T7 rows, `2c65eeacf3` = 1, branch head =
1). The "99 E0308 sites" figure is `docs/probes/e0308_root_partition_2026-08-18.md`'s corpus-wide,
13-mechanism-root denominator (root T7, 99/408 sites, 24.3%, across 11 entry modules) — a
different, larger population than any one module's reachable-today count. `stern-fox-619`'s task
title quoting "99 E0308 sites" is quoting that whole-corpus census, not this module: scoping the
NARROW T7 repair to `materialization_carriers` alone will dent this module's board by at most the
1 E0599 row above (and that row's own ownership is stale — see below), out of scope for a
table-absent-names NARROW fix per prediction B below — the "99 sites" headline is a corpus-wide
claim, not something this module's build can deliver on its own.

**On T7's local yield, said loudly (per `smart-ram-730`'s request):** the reshaping is real. If
this module's entire live T7 footprint is one row, and #8410 already retired the other eight, then
T7 is not a `materialization_carriers` blocker — it is a corpus-wide class (99/408 sites, 11 entry
modules) whose yield in *this* module is one E0599 site. `stern-fox-619`'s repair's value is the
class fix across the corpus, not a dent in this board.

**Ownership check (per `smart-ram-730`'s ask):** row 7 was attributed to `calm-lynx-547`, checked
via `dashboard-ops` — that session is **archived**. This row's owner is stale, not confirmed live.
Flagging here rather than silently carrying the attribution forward: if no lane has since picked up
T7-for-this-module, row 7 was **unowned**, and it is exactly the kind of single leftover row that
goes unnoticed until someone else drives the board to zero and finds it still there.

**Update, same day:** `smart-ram-730` has since assigned row 7 to `stern-fox-619` — the stale
`calm-lynx-547` attribution above is superseded, row 7 is no longer unowned, and no further routing
is needed from this session.

## 6. Predictions, restated against this base, before the causing PRs land

Both predictions from #8460 §11 hold verbatim; restated here against `72676cf0be` so the base is
current when each PR lands.

**Prediction A** (`silent-raven-853`, reachability-gate fix): this module's closure exercises the
generic-container checkpoint-scalar mechanism **zero times** — none of the 51 diagnostics mention
`Nat`, `Magnitude`, or `CommutativeSemiring` in message/labels/children (re-checked against this
run's JSON, not carried forward from #8460). Prediction: the fix moves this module's count by
**zero**. Any movement is composition with something else, reported as its own delta, never folded
into row totals above.

**Prediction B** (`stern-fox-619`, T7 NARROW to table-absent names): row 7 (T7, 1 site) is the only
T7-attributable row in this module. Prediction: table-present names (`Int`/`String`/`Hash`
answering from the table) leave every other row (1a/1b/2–6/8/9, 50 sites) untouched. Any movement
outside row 7 means the NARROW scope reached wider than declared.

## 7. Standing offer

Unchanged from #8460, continued: any lane sends a head SHA, this session re-runs the instrument
above against it and publishes the delta, with two-arm discipline (the fix measured on two bases)
whenever a landing might be confused with base drift. Addressed to `silent-raven-853`,
`deep-swift-570`, `stern-fox-619`, and `smart-ram-730`.
