# The qualified-pattern-head cost is not the pattern head (2026-08-23)

**Subject:** the cost observation carried forward, unrepaired, from gunbc#9004 —
`test.claim.qualified_pattern_head_witness_test.qualified_pattern_head_binds_the_instantiation`
measured `cpu_ms 58579`, `rss delta +1.21GB`, `outcome=timed_out` against a 5000ms budget under
the required-floor harness, while its bare control passed. That PR named the arms, ruled out two
causes, named a next discriminator, and explicitly did **not** claim the head was the cause.

**This probe runs the discriminator the PR named, and the attributed subject is EXONERATED.**
A qualified pattern head costs nothing measurable. The cost attaches to a *qualified type
annotation*, it is **one-time per process**, and it is paid by whichever claim happens to compile
the first dotted type annotation in that process.

## Method — a factorial probe, not a re-measurement of the witness

The witness's two arms differ in **two** ways at once: the pattern head spelling AND the parameter
type annotation spelling. Both of its arms are therefore confounded for the question "what costs".
`dag/test/claim/qualpat_cost_probe.dag` splits the two axes over one fixture
(`test.fixture.qualpat_provider`, unchanged), all five probes carrying identical imports:

| probe | pattern head | type annotation |
|---|---|---|
| A | qualified | qualified |
| B | bare | bare |
| C | bare | **qualified** |
| D | **qualified** | bare |
| E | (no match, no import — trivial control) | — |

Each probe is one `compile_dag_rust_emit_check` call. All five run in ONE `claim_batch` process, so
the entry resolve is paid once and outside the fold; the per-claim `cpu` figures below are the
marginal cost of the compile itself. Release build, remote amd64 runner, `--source-root dag
--source-root src/v2`.

**The order is varied deliberately**, because the first measurement showed the premium on whichever
probe ran first and a single ordering cannot distinguish "this probe is expensive" from "the first
probe is expensive".

## Measured — four orderings

| run order | A | B | C | D | E |
|---|---|---|---|---|---|
| A,B,C,D,E | **256** | 5 | 5 | 5 | 1 |
| E,B,C,D,A | 5 | 6 | **222** | 5 | 40 |
| D,B,E,C,A | 6 | 6 | **201** | 44 | 1 |
| C,A,D,B,E | 7 | 5 | **251** | 5 | 1 |

(cpu ms per claim; **bold** = the premium payer in that run. All five probes PASS in every run.)

## What the table says

1. **The qualified pattern head is free.** D carries a qualified head and a bare annotation. It
   costs 5ms in every run where it is not first, and 44ms when it runs first — which is the same
   as the trivial control E costs when *it* runs first (40ms), i.e. generic first-call warmup and
   nothing more. Running D first does **not** discharge the premium: C still pays 201ms afterwards.
2. **A qualified type annotation carries the premium, once.** In every ordering the premium lands
   on the first probe whose source contains a dotted *type annotation* — A when A leads, C in all
   three orderings where a bare-annotation probe leads. Once paid, every later dotted-annotation
   compile costs 5ms: in run 1, A pays 256ms and C costs 5ms; in run 4, C pays 251ms and A costs 7ms.
3. **It is one-time per process, not per call**, and therefore attributed to an arbitrary claim —
   whichever one happens to reach a dotted type annotation first.

That is sufficient to retire the witness's implied attribution: the arm named
`qualified_pattern_head_binds_the_instantiation` was charged for a cost its pattern head does not
cause. Its two arms differed in the annotation too, and the annotation is the axis that pays.

## Ruled out, each by reading the producer

- **Name lookup itself.** `symbol_index_lookup` is a `map_get` over `entries: Rc<HashMap<..>>`;
  the `.clone()` at that call site is an `Rc` refcount bump, not a map copy. `global_bare_lookup`
  on a miss is likewise a `map_get` returning `Absent`.
- **The census fill.** `compile_dag_rust_emit_check` reaches `compile_sources`, which takes
  `default_compile_pipeline_options()`, whose `census_only_sources` is `[]`. So
  `parse_census_fill_sources` — the whole-tree parse that `gunbc.ci_spec`'s clamp note measures at
  ~23s local — has an empty input on this path and cannot be the term. Worth stating positively:
  the qualified reference resolves here with **no** census to resolve against.
- **The floor prepared-inventory digest.** `floor_prepared_inventory_digest()` clones a `String`
  precomputed on the authority; it does not re-hash the corpus per call.
- **`module_path_index`.** Cached per thread and, per #9004's own ledger reading, paid outside the
  fold. (Noted in passing, not as this probe's finding: `build_module_path_index` returns
  `index.clone()` — a full deep copy of the whole-corpus `HashMap<String,String>` — on every call
  including every cache HIT. That is a copied-accumulator cost shape under DESIGN §6, far too
  small to be this defect, and is not repaired here.)

## What is NOT claimed

**The magnitude gap is open and is not papered over.** The premium measured here is ~200-250ms;
the floor observed 58579ms and +1.21GB. This probe establishes the *shape* (one-time per process)
and the *trigger* (first dotted type annotation), and it exonerates the pattern head. It does
**not** establish that the same term accounts for 58.6s and 1.21GB at floor corpus scale — that
would be the rung inflation DESIGN §4b names as worse than sitting low. The remaining work is to
name the lazily-built structure and show it is corpus-denominated.

## THE MECHANISM — found, and it is not name resolution at all

Qualified-name resolution is cheap. What a qualified type annotation does is emit an **advisory
diagnostic**, and the cost is in **classifying that diagnostic's severity**.

```
compile_dag_rust_emit_check
  -> compile_sources                       (the compile itself: ~8ms of trace_mark phases)
  -> result.diagnostics.filter(compile_clean_diagnostic_is_hard)
       CompilerDiagnostic::UnlistedImportUse { .. }
         -> compile_clean_unlisted_import_use_blocks_cached()      <-- thread_local CACHED
              -> compile_clean_unlisted_import_use_blocks_from_policy()
                   let roots = default_source_roots();             <-- THE WHOLE TREE
                   resolve_entry_graph_shared(&roots, "dag/gunbc/compile_clean_diagnostic_policy.dag")
                   run_in_context_with_args(ctx, "compile_clean_unlisted_import_use_blocks", &[])
```

**A whole-tree resolve + typecheck of a separate entry closure, to evaluate one nullary `Bool`.**
It is `thread_local`-cached, so it is paid **once per process** by whichever claim first produces
an `UnlistedImportUse`, and it is **independent of what was compiled**.

**Why only the qualified spelling triggers it.** `UnlistedImportUse` comes from the masked type-ref
arm of `resolve_node_bounded` (`v1_compiler_infer_resolve.rs`), which fires when the **authored**
type name is absent from `env.source_visible_names`. The import list contributes **bare** names
(`QualpatResult`), so the authored name `test.fixture.qualpat_provider.QualpatResult` misses and the
advisory is emitted; a bare annotation hits and emits nothing. A qualified **pattern head** never
enters that type-ref arm at all — it goes through `lookup_variant_in_type` / `symbol_index_lookup` —
which is exactly why probe D is free.

So the qualified/bare asymmetry that made this look like a resolution-cost defect is really:
*the qualified spelling is the only one that produces the diagnostic whose severity lookup is
corpus-denominated.*

### Confirming control

Order B then D — both bare annotations, D carrying the qualified pattern head: 72ms / 8ms, and
`dag/gunbc/compile_clean_diagnostic_policy.dag` **does not appear at all** in `span_nanos_by_entry`.
In both qualified-annotation orderings it appears exactly once, at 259ms and 269ms — the premium,
to within ~10ms. The axis switches the policy resolve on and off.

## This is a KNOWN, DOCUMENTED, UNFIXED defect — and the floor number matches its original measurement

`gunbc.ci_spec` `gunbc_ci_floor_batch_clamp_note` already carries it, measured 2026-07-25:

> **(B) THE POLICY TAIL, after the compile:** `compile_clean_unlisted_import_use_blocks_from_policy`
> calls `default_source_roots()` (WHOLE TREE) + `resolve_entry_graph_shared` to evaluate ONE nullary
> Bool fn. ~34-42s, **once per process**, and INDEPENDENT of what was compiled — proven by running
> the same module with only its own source root: zero pool, all phases 0ms, still 42.4s wall. […]
> a RED compile never reaches the gate (measured 27.4s total, ~2s tail) while a **GREEN compile pays
> it in full (57.8s total, ~34s tail)**.

**57.8s there; `wall_ms=57337` on the floor line that opened this investigation.** That note's
`dissolve-on` names the repair — *"scope the policy resolve to the policy module's own import
closure"* — and it was never discharged. The defect did not reappear; it never left. What is new
here is **why one arm and not the other**: the trigger is the authored-spelling miss in
`source_visible_names`, which is what makes a green qualified compile pay the tail and a green bare
compile skip it.

## Superlinearity — answering the shape question directly

**It is not superlinear in the witness. It is a fixed, corpus-denominated, once-per-process tail.**
Two consequences, and the second is the one that matters:

- It does **not** grow with the subject compiled. Nothing about the probe's size, import count, or
  qualified-name length changes it; a second qualified compile in the same process costs 5ms.
- It **does** grow with the corpus, because `default_source_roots()` is the whole tree and the
  resolve is a full entry closure. Measured, same probe, same process, varying only the roots
  `claim_batch` was given:

| roots given to claim_batch | B (bare) | C (first qualified) |
|---|---|---|
| `dag` + `src/v2` | 48ms | **216ms** |
| `dag` only | 12967ms | **44886ms** |

The dag-only arm is ~208× more expensive — because `from_policy` asks for `default_source_roots()`
regardless of what the process was given, so its whole-tree resolve is a **different key** from the
warm shared index and pays a cold whole-tree load. **44.9s locally, against 57.3s on the floor** —
the same order, on the same mechanism. That closes the magnitude gap this probe had left open.

So the urgency test the brief posed is answered: the cost is constant per process but
**corpus-denominated**, and it is paid on a *cold* index whenever the policy resolve's root set
diverges from the caller's. Both halves grow with the repository.

## Is this path scheduled for deletion? (do not optimise a corpse)

Partly, and it changes who should fix it rather than whether. `cli_run.rs` is deleted wholesale by
`integration/cli-run-cut`, and `compile_clean_unlisted_import_use_blocks_from_policy` lives there.
But no bounded event retires it: v1 is *semantics frozen, maintenance active* with no cutover date
(DESIGN §3, `gunbc.v1_maintenance_standing`), and the required floor runs this path on every run
today. The named repair — scope the policy resolve to the policy module's own import closure — is
small, is already written down as that note's dissolve-on, and does not extend the seed's surface.

## What is claimed, and what is not

**Claimed, by execution:** the qualified *pattern head* costs nothing (four orderings, plus a
control in which the policy entry is absent from the span table); the premium is the first
`UnlistedImportUse` in a process; the payer is `compile_clean_unlisted_import_use_blocks_from_policy`
resolving a whole-tree entry closure to compute one `Bool`; the cost is corpus-denominated and
reaches 44.9s locally on a cold root set.

**Not claimed:** that the floor's 1.22GB is *entirely* this term. The floor line was not
re-measured under this probe — the witness is quarantined as of #9031 and the offline recipe is
what ran here. The wall figures agree (44.9s local cold vs 57.3s floor, and the 57.8s the 2026-07-25
note measured for exactly this tail), and the RSS shape is consistent with a whole-tree entry
resolve, but "consistent with" is not "measured", and this document does not upgrade it.

## THE REPAIR — landed in this change, and re-measured

`compile_clean_unlisted_import_use_blocks_from_policy` no longer calls `default_source_roots()`.
It assembles the policy entry's **own import closure** (`compile_clean_policy_entry_closure_sources`)
and resolves that explicit source set through `resolved_graph_from_sources(.., Strict)`.

**Every failure arm refuses; none widens.** An import naming no module in the roots, or a file that
cannot be read, returns a located `Err` naming the module and the path. It must never fall back to
the whole tree: that arm would restore today's cost, zero the deficit's frequency by construction,
and make the widening unrankable ever after (DESIGN §5, the absorbing fallback). This is also why
the closure builder is a new function rather than a reuse of `resolve_virtual_source_with_imports`,
whose BFS *silently skips* an import it cannot resolve — a silent skip here would answer the policy
question from a graph missing the module the answer depends on.

### Measured, same probe, same orderings, post-fix

| roots | probe | pre-fix | post-fix |
|---|---|---|---|
| `dag` only | C (first qualified) | 44886ms | **109ms** |
| `dag` only | B (bare) | 12967ms | 12881ms |
| `dag` only | A (second qualified) | 6ms | 6ms |
| `dag` + `src/v2` | C (first qualified) | 216ms | **151ms** |
| `dag` + `src/v2` | A, D, B, E | 5-7ms | 6-8ms |

All probes PASS, so the narrowed closure still resolves and still returns the policy `Bool` — the
repair removed work, not evidence.

**The asymmetry is gone rather than reduced.** On the cold root set the qualified arm went from
3.5× the bare arm to *cheaper* than it (109ms against 12881ms, the bare arm now merely paying the
generic first-call warmup). There is no longer a qualified-spelling premium to attribute.

### The residue this exposes — reported, not absorbed

**`B` did not move: 12967ms → 12881ms on the `dag`-only root set.** That cost is paid by the *bare*
arm too, so it was never part of the qualified asymmetry and this repair does not touch it. It is a
separate cold-index term for a root set that does not match the process's warm index, and it is
named here rather than folded into this row's result — a fix that quietly widened its own claim to
cover a neighbouring cost would be the same conflation this document exists to undo.

## The RSS claim is NOT upgraded by the repair

The floor's 1.22GB was never attributed by execution and still is not. Three wall figures agree
(44.9s local cold, 57.3s floor, 57.8s in the 2026-07-25 note) and a whole-tree entry resolve is a
plausible shape for GB-scale growth, but plausible-shape is not measurement. If the memory line
survives on the floor after this repair, that is a **second defect to find**, not a result to absorb
into this one.

## Apparatus — re-run recipe

The probe module is reproduced here rather than left in the tree: it has no consumer, so as a
committed `.dag` it would be experimental residue (DESIGN §6). Write it to
`dag/test/claim/qualpat_cost_probe.dag` (the name deliberately does NOT end in `_test.dag`, so
floor discovery cannot enrol it), then run — one remote dispatch, since the runners are amd64 and
session containers are arm64:

```
ctrl-build --remote -- bash -lc 'cargo build --release --bin claim_batch && \
  ./target/release/claim_batch --entry dag/test/claim/qualpat_cost_probe.dag \
    --functions probe_c_bare_head_qual_annot,probe_a_qual_head_qual_annot,probe_d_qual_head_bare_annot \
    --source-root dag --source-root src/v2'
```

Vary the `--functions` order to move the premium; drop `--source-root src/v2` for the cold arm.

```dag
module test.claim.qualpat_cost_probe

fn qualpat_cost_check(src: String, path: String) -> Bool {
  compile_dag_rust_emit_check(src, path, [], [])
}

test fn probe_a_qual_head_qual_annot() -> Bool {
  qualpat_cost_check(
    "module qualpat_a_mod\n\nimport test.fixture.qualpat_provider \{ QualpatResult, QualpatPayload, QualpatOk, QualpatErr \}\n\nfn qualpat_a_read(r: test.fixture.qualpat_provider.QualpatResult<test.fixture.qualpat_provider.QualpatPayload>) -> String \{\n  match r \{\n    test.fixture.qualpat_provider.QualpatOk \{ value: v \} => v.root\n    test.fixture.qualpat_provider.QualpatErr \{ code: _ \} => \"\"\n  \}\n\}\n",
    "src/qualpat_a_mod.rs"
  )
}

test fn probe_b_bare_head_bare_annot() -> Bool {
  qualpat_cost_check(
    "module qualpat_b_mod\n\nimport test.fixture.qualpat_provider \{ QualpatResult, QualpatPayload, QualpatOk, QualpatErr \}\n\nfn qualpat_b_read(r: QualpatResult<QualpatPayload>) -> String \{\n  match r \{\n    QualpatOk \{ value: v \} => v.root\n    QualpatErr \{ code: _ \} => \"\"\n  \}\n\}\n",
    "src/qualpat_b_mod.rs"
  )
}

test fn probe_c_bare_head_qual_annot() -> Bool {
  qualpat_cost_check(
    "module qualpat_c_mod\n\nimport test.fixture.qualpat_provider \{ QualpatResult, QualpatPayload, QualpatOk, QualpatErr \}\n\nfn qualpat_c_read(r: test.fixture.qualpat_provider.QualpatResult<test.fixture.qualpat_provider.QualpatPayload>) -> String \{\n  match r \{\n    QualpatOk \{ value: v \} => v.root\n    QualpatErr \{ code: _ \} => \"\"\n  \}\n\}\n",
    "src/qualpat_c_mod.rs"
  )
}

test fn probe_d_qual_head_bare_annot() -> Bool {
  qualpat_cost_check(
    "module qualpat_d_mod\n\nimport test.fixture.qualpat_provider \{ QualpatResult, QualpatPayload, QualpatOk, QualpatErr \}\n\nfn qualpat_d_read(r: QualpatResult<QualpatPayload>) -> String \{\n  match r \{\n    test.fixture.qualpat_provider.QualpatOk \{ value: v \} => v.root\n    test.fixture.qualpat_provider.QualpatErr \{ code: _ \} => \"\"\n  \}\n\}\n",
    "src/qualpat_d_mod.rs"
  )
}

test fn probe_e_trivial_no_import() -> Bool {
  qualpat_cost_check(
    "module qualpat_e_mod\n\nfn qualpat_e_read(x: Int) -> Int \{ x \}\n",
    "src/qualpat_e_mod.rs"
  )
}
```
