# The first executed seven-boundary product receipt (2026-08-21)

**Subject line first, because every number below is worthless without it.**

| | |
|---|---|
| repository ref | current `main` after gunbc#8754 (`559197bf92b`), merged into this branch |
| producer | `v2.workflow.product_receipt_transport` `run_seven_boundary_product_receipt`, executed through `gunbc run` |
| binaries | `gunbc`, `discover_source_root_ingest`, `cssl_assemble`, all `--release` from that tree |
| host | session container, arm64, interpreter (not a compiled realization — see §6) |
| configuration | seed linking ON (`cssl_assemble` IS the seed-link assembler; §5), no hand shim, no shim `lib.rs`, no lane override |
| receipt artifact | `target/product-receipt/receipt.txt`, written by the run itself |

---

## 1. What was run, and what makes it one transaction

1. `discover_source_root_ingest --source-root src/v2 --entry <subject> --emit-dag-manifest target/product-receipt/manifest/host_source_root_ingest_manifest.dag`
2. the driver reads those exact bytes back and hashes them
3. it launches ONE `gunbc run` whose source roots are `dag`, `src/v2`, **and that manifest directory**, passing the digest as `--arg manifest_identity=`
4. the stage re-hashes the file it was actually resolved against and compares
5. boundaries 2–7 run inside that one process, each consuming what its predecessor produced

Every existing gate on this path emits its manifest into a per-run `mktemp` directory, runs one claim, and deletes it. No boundary has ever consumed another boundary's artifact because the artifact does not survive the boundary. The fixed, uncleaned transaction directory is the whole mechanical difference.

**Identities are computed in-transaction, never stored, and that is load-bearing rather than incidental.** The manifest digest for `00_compile.dag` read `7290479a745eca88` this morning and `dd93bab9f42c8a58` this afternoon — same subject, different runs, because the manifest is regenerated per run and `main` moved between them. A receipt pinned to a recorded digest would have carried a stale subject forward inside the very instrument built to catch wrong-subject claims. Nothing here is pinned, so that class is unrepresentable rather than merely avoided.

---

## 2. THE RECEIPT — subject `src/v2/compiler/00_compile.dag`, the compiler's own closure

```
B1 host derives manifest        Completed
   produced fnv1a64:dd93bab9f42c8a58 over 1 file(s)
B2 manifest population admitted Completed
   consumed fnv1a64:dd93bab9f42c8a58 over 1 file(s)
   produced fnv1a64:c1d2109befa8f88d over 107 file(s)
B3 refs read and hash-verified  Completed
   consumed fnv1a64:c1d2109befa8f88d over 107 file(s)
   produced fnv1a64:c1d2109befa8f88d over 107 file(s)
B4 closure candidate emitted    Refused -- emitter rejected the admitted population, diagnostics: 40
      7  parse_grammar_choice_overlap_residue
     32  body_lowering_reason_wrapper_retained_emitted
      1  body_lowering_reason_unsupported_form
   consumed fnv1a64:c1d2109befa8f88d over 107 file(s)
B5 seed-linked assembly         NotExercised
B6 Cargo.toml from authority    NotExercised
B7 cargo                        NotExercised

covers every boundary once: true
every boundary completed:  false
identity chain intact:     false
RECEIPT: RED
```

**Three boundaries execute with artifact identity carried through every hand-off.** The manifest digest appears on both sides of B1→B2; the population digest appears on both sides of B2→B3 and again at B4's consumed. That chain is the thing no per-component board reports.

**Boundary 4 is a subject-specific refusal carrying a partition, not a label.** This is the first real number at that boundary in this repository.

**AND THE PARTITION IS MOSTLY CARRIAGE, NOT CAUSE (added 2026-08-21).** 32 of the 40 come from `body_lower_wrapper_retained_shell` and 7 from `grammar_validate_and_analyze`, both of which return `Accepted`; they enter this list through `rejected_with_pending`. Exactly ONE diagnostic per subject rejects -- `body_lowering_reason_unsupported_form` here, `fold_lowering_shape_invalid` on the 29-module control below. §3.1's ranking of the wrapper-retained families is therefore a ranking of non-blocking observations, and this sentence's "first real number at that boundary" is true of the count and false of the door. -> [b4 causal versus carried](b4_causal_versus_carried_2026-08-21.md).

**THE DOOR HAS SINCE MOVED, AND THIS BLOCK'S NUMBERS ARE THE BEFORE SIDE (2026-08-21).** The single rejecting diagnostic was `dag_surface_let_expr` -- the ordinary statement-form `let`, whose Bind body is the enclosing statement spine's remainder and therefore is not reachable from the let node at all. With that lowered at `dag_surface_stmt_seq`, boundary 4 refuses at a DIFFERENT producer: 45 diagnostics, `7 parse_grammar_choice_overlap_residue / 37 body_lowering_reason_wrapper_retained_emitted / 1 normalized_tree_reason_wrapper_retention_not_normalized`. Note what changed about the wrapper-retained population: it is no longer merely carried -- the new door-holder is the normalized-tree admission refusing BECAUSE retention is present, so those 37 are now upstream of the refusal rather than beside it. -> [b4 door-holder: the statement-form let](b4_door_holder_statement_let_2026-08-21.md).

---

## 3. The 29-module control — subject `src/v2/compiler/07_target_carriers.dag`

Run so that boundaries 2–4 are not code that has only ever executed on one subject.

```
B1 Completed  produced fnv1a64:7923995631fb3acd over 1 file(s)
B2 Completed  consumed 7923995631fb3acd  produced fnv1a64:fbe68561dd25a62c over 29 file(s)
B3 Completed  consumed fbe68561dd25a62c  produced fbe68561dd25a62c over 29 file(s)
B4 Refused -- emitter rejected the admitted population, diagnostics: 18
      7  parse_grammar_choice_overlap_residue
     10  body_lowering_reason_wrapper_retained_emitted
      1  fold_lowering_shape_invalid
B5..B7 NotExercised
RECEIPT: RED
```

### 3.1 What the two partitions say when differenced

This is why the partition is the measurement and the total is a summary of it. Two subjects, one 3.7× the other:

| cause | 29-module closure | 107-module closure |
|---|---|---|
| `parse_grammar_choice_overlap_residue` | 7 | **7** |
| `body_lowering_reason_wrapper_retained_emitted` | 10 | 32 |
| `body_lowering_reason_unsupported_form` | 0 | 1 |
| `fold_lowering_shape_invalid` | 1 | 0 |

- **`parse_grammar_choice_overlap_residue` is 7 on both, and the mechanism is already named in the corpus.** `v2.test.claim.long.compile_door_ledger_test` `compile_door_ledger_witness_note` records it as **a fork between two parse entry points** — the packaged `parse()` the admission pre-parse uses rejects a grammar-choice overlap that `parse_production` tolerates — classified `MigrationOwned`, dissolving with the packaged/`parse_production` reconciliation.

  That upgrades the two-point observation from a pattern to an explanation, and the explanation predicts the invariance rather than merely matching it: the population is the set of constructs on which the two parse paths disagree, which is a property of the GRAMMAR and was never indexed by the subject, so it cannot grow with the frontier. A third subject is therefore not needed to settle it — a `7` there would corroborate without explaining, and a coincidence at two sizes would look the same. If a later run reports a non-7, that is a genuine surprise worth stopping on rather than a data point.

  **Corroborated three times over since, and by the strongest available control (2026-08-21).** `7` also holds on a 4-module closure, on a 3-module closure, and on a **single-file closure with no imports at all** — the case where the subject contributes as little as a subject can. A constant that survives the closure shrinking to one file is not indexed by the subject, which is what the mechanism predicted and what no amount of agreement between two large subjects could have shown. Recorded here as corroboration only: the explanation above was already load-bearing and is unchanged. Note that these same runs FALSIFIED a neighbouring claim in this document — the invariance survived the scrutiny that the wrapper-retained denominator did not.

  **The number is the new fact, not the mechanism.** The reconciliation was already a known-good migration with no cost attached to it. It is now worth exactly **7 diagnostics at boundary 4 on every subject, independent of closure size** — which is what lets it be ranked against work whose cost does grow.
- **`body_lowering_reason_wrapper_retained_emitted` is 10 → 32** and dominates both totals (56% and 80%). ~~It is the population any B4 work should be denominated against.~~ **CORRECTED 2026-08-21, same day, by execution: it is not a population and must not be used as a denominator.** Both figures are ONE MODULE'S diagnostics, not their closures'. The emission stops at a refusing module and reports that module, so `10` and `32` describe two different single modules rather than one population at two scales — which is, exactly, the confusion the bullet below congratulates the partition for avoiding. Five subjects settle it, including a **1-module closure that reports the same as a 4-module closure containing it** and a 3/4/29-module trio that report byte-identically because they share one refusing dependency. The evidence and what it destroys: → [b4 wrapper-retained census](b4_wrapper_retained_census_2026-08-21.md), §0.
- **The two singletons are different causes on the two subjects**, which is exactly the case a bare total cannot express: 40 and 18 would have looked like one population at two scales.

---

## 4. What this receipt found, and what was done about it

The first run of this transaction — before gunbc#8754 — refused at **boundary 2** with `manifest coverage incomplete`, on a subject whose contents had nothing to do with it. `closure_emit_population_admission` consumes the closure-**ref** population but gated on `source_root_coverage_is_complete`, a question about the **inline** carrier it never reads. Past `manifest_inline_list_max = 64` the inline carrier is refused as `SourceRootManifestElided` while the ref population stays complete, so every closure over 64 modules was unemittable regardless of contents. The compiler's own closure is 107.

Repaired in gunbc#8754 (`source_root_ref_transport_coverage_admits`, narrowed not deleted) and merged before this receipt was taken. The pre-fix red is banked in that PR as the repair's discriminating evidence rather than landed on main as a known-wrong standing.

---

## 5. Two inherited standings that were wrong, corrected by execution

- **"B5 NotExercised — `CSSL_STD_SEED_LINK=0`."** There is no `CSSL_STD_SEED_LINK` in the tree. A sweep over `.rs`/`.dag`/`.yml` returns nothing; the only hits are in `docs/probes/*` recording how the deleted shell probe runners were invoked. `cssl_assemble` IS the seed-link assembly entry point (`cssl_std_seed_link_followup_note`), so there is no switch to leave at zero — a run either invokes the assembler or does not. A producer field outlived its producer, and "seed link OFF" was being read as a measured condition.
- **"B2 Refused — committed stub, `read_count = 0`."** That is the no-overlay path. With the overlay this transaction supplies, `read_count` is 107. The stub arm survives as a distinct cause, `ManifestOverlayNotResolved`.

---

## 6. What this receipt does NOT cover, named rather than left to be found

- **Boundaries 5, 6 and 7 have never executed.** They are wired — `cssl_assemble`, the repository's `cssl_v1_compiled_probe_lib_cargo_toml` Cargo authority, `cargo.Build.BuildManifest` — and no subject has yet emitted cleanly enough to reach them. Five-sevenths of the transport is code that has not run.
- **B4 materializes the DRIVER's emission**, not a second `gunbc compile --output-dir` invocation. `drive_compiler_closure_emit` returns one `Medium<String>`, written to `<txn>/candidate/src/lib.rs`, and that file is what B5 would assemble. The bigger many-file candidate the rustc board measures comes from a different producer over the same subject; substituting it here would be the independent-green-components move this receipt exists to refuse. If the single-medium emission cannot be assembled as a crate, B5 refuses and names that seam. The file count carried beside every digest is what keeps the two distinguishable at a glance, and carriage requires it to match — a same-digest-different-shape hand-off is refused.
- **The overlay's resolution is discriminated, not proved.** The committed stub declares an empty content hash, so the stage refuses (`ManifestOverlayNotResolved`) unless the imported module's own non-empty digest appears inside the file the transaction wrote. That separates stub from overlay; it does not prove the loader chose that path over an identical one.
- **The cause partition depends on `symbol_lexeme`, a host-intercepted bridge.** It is available to the interpreted transport and would not survive a compiled realization of it unchanged. The receipt would otherwise look portable across realizations when one of its fields is not.
- **The 29 `compile_error!("unsupported mock expression")` invocations** are checked on the emitted candidate BEFORE assembly (`EmittedHardRefusalsPresent { count }`), never on cargo's module graph — an assembly that omitted one of those files would let a cargo-only check pass over a known-broken emission. Neither subject reaches that check, because B4 refuses first.
- **This transport is not enrolled in CI.** The witness floor discovers the receipt ALGEBRA (`src/v2/test/claim/workflow/product_receipt_test.dag`, seven claims). The transaction needs three release binaries and a cargo build; enrolling it is a separate decision under the CI re-add queue.
- **One declared cost:** boundaries 2–3 read the admission through `closure_emit_population_admission` and boundary 4 calls `drive_compiler_closure_emit`, which runs that admission again — every ref is read twice. `product_receipt_stage_double_read_note` records why the cheaper composition was refused, with its dissolution trigger.

---

## 7. Reproducing

```
cargo build --release -p v1-compiler --bin gunbc --bin discover_source_root_ingest --bin cssl_assemble
./target/release/gunbc run --source-root dag --source-root src/v2 \
  --entry src/v2/workflow/product_receipt_transport.dag \
  --function run_seven_boundary_product_receipt
cat target/product-receipt/receipt.txt
```

Any other subject — this is how a third data point for §3.1 is produced:

```
./target/release/gunbc run --source-root dag --source-root src/v2 \
  --entry src/v2/workflow/product_receipt_transport.dag \
  --function run_product_receipt_for_entry --arg entry=<path.dag>
```

Wall clock in the interpreter: ~25 min for the 107-module subject, ~15 min for the 29-module one.
