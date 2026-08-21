# The first executed seven-boundary product receipt (2026-08-21)

**Subject line, stated first because every number below is worthless without it.**

| | |
|---|---|
| repository ref | `608248c15e0035a9e99a7937d5f9b8c124dced8f` (main), plus the four files this receipt's PR adds |
| producer | `v2.workflow.product_receipt_transport` `run_seven_boundary_product_receipt`, executed through `gunbc run` |
| binaries | `gunbc`, `discover_source_root_ingest`, `cssl_assemble`, all built `--release` from that tree in one local `ctrl-build --local` dispatch |
| host | session container, arm64 |
| configuration | seed linking ON (`cssl_assemble` IS the seed-link assembler; see §4), no hand shim, no shim `lib.rs`, no lane override |
| receipt artifact | `target/product-receipt/receipt.txt`, written by the run itself |

---

## 1. What was actually run, and why it is one transaction

Ten steps, one transaction directory, nothing cleaned between boundaries:

1. `discover_source_root_ingest --source-root src/v2 --entry <subject> --emit-dag-manifest target/product-receipt/manifest/host_source_root_ingest_manifest.dag`
2. the driver reads those exact bytes back and hashes them (`content_hash_atom`)
3. it launches ONE `gunbc run` whose source roots are `dag`, `src/v2`, **and that manifest directory**, passing the digest as `--arg manifest_identity=`
4. the stage process re-hashes the manifest file it was resolved against and compares
5. boundaries 2–7 run inside that one process, each consuming the artifact its predecessor produced

The existing gates on this path each emit a manifest into a per-run `mktemp` directory, run one claim, and delete it. That is why no boundary has ever consumed another boundary's artifact: the artifact does not survive the boundary. The fixed transaction directory is the whole mechanical difference.

**What is NOT claimed.** The manifest overlay's resolution is checked by a discriminator, not proved: the committed stub declares `host_source_root_ingest_content_hash = ""`, so the stage refuses (`ManifestOverlayNotResolved`) unless the imported module's own non-empty digest string is present inside the file the transaction wrote. That distinguishes stub from overlay. It does not prove the loader resolved *that* path rather than an identical one.

---

## 2. THE RECEIPT — subject `src/v2/compiler/00_compile.dag`

```
B1 host derives manifest
  standing: Completed
  consumed: <boundary not reached>
  produced: fnv1a64:7290479a745eca88
B2 manifest population admitted
  standing: Refused -- manifest coverage incomplete
  consumed: fnv1a64:7290479a745eca88
  produced: <boundary not reached>
B3..B7                     NotExercised

covers every boundary once: true
every boundary completed:  false
identity chain intact:     false
RECEIPT: RED
```

Two facts, and the second is the finding.

**B1 moved from NotExercised to Completed, and its artifact reached B2 by identity.** `7290479a745eca88` on both sides is the same file, hashed once by the host that wrote it and once by the process that was resolved against it. That link is the thing no per-component board could report.

**B2 refuses, and it refuses for a reason that has nothing to do with this subject's health.** The ingest emitter read 107 sources. `manifest_inline_list_max` is 64. Past that cap the INLINE carrier is refused and coverage is recorded as `SourceRootManifestElided { read_count: 107, cap: 64 }`, while the closure-**ref** carrier stays complete (`produced_row_count == ingest_read_count == 107`). `closure_emit_population_admission` then gates on `source_root_coverage_is_complete`, which answers `false` for `Elided`.

### 2.1 The finding, stated as a defect and deliberately NOT fixed here

`v2.compiler.self_host.compiler_closure_emit_driver` `closure_emit_population_admission` consumes the **ref** population and gates on a predicate about the **inline** population.

- The driver's own next arm, `source_root_closure_ref_transport_complete`, is the correct completeness question for the carrier it consumes, and at this subject it is TRUE.
- `v2.compiler.source_authority` `source_root_coverage_arms_note` states the split in its own words: `SourceRootManifestElided` is "the HOST emitter's refusal: a declared cap rejected the whole row list", and the manifest carrier note states that past the cap "the ref population stays complete".
- Consequence: **any closure larger than 64 modules is unemittable through this driver**, regardless of its contents. The compiler's own closure is 107. This is not a property of `00_compile.dag`; it is a property of every real subject.

It is not repaired in this PR, on purpose. The brief that opened this lane said the first run is expected RED and that its value is being the RIGHT red, and repairing an admission ladder in the same change that first executes it would make the receipt a claim about the repair rather than about the composition. The repair is one decision — whether `Elided` is a coverage deficit for a ref-carrier consumer, or a fact about a carrier that consumer does not read — and it belongs to whoever owns `source_authority`'s admission semantics.

---

## 3. Subject `src/v2/compiler/07_target_carriers.dag` — the control that reaches past B2

A 29-module closure sits under the cap, so coverage is `SourceRootCoverageComplete` and the same transaction advances. It is run for exactly one reason: without it, boundaries 3–7 of this receipt would be code that has never executed, which is the specification-without-execution failure DESIGN §5 names.

<!-- RESULT-07 -->

---

## 4. Two inherited standings that were wrong, corrected by execution

- **"B5 seed-link assembly NotExercised — `CSSL_STD_SEED_LINK=0`."** There is no `CSSL_STD_SEED_LINK` left in the tree: `grep -rn CSSL_STD_SEED_LINK` over `.rs`/`.dag`/`.yml` returns nothing, and the only hits are in `docs/probes/*` recording how the deleted shell probe runners were invoked. `cssl_assemble` IS the seed-linked assembly entry point (`cssl_std_seed_link_followup_note`), so the setting cannot be left at zero — a run either invokes the assembler or does not.
- **"B2 Refused — committed stub, `read_count = 0`."** True of a run with no overlay. With the overlay this transaction supplies, `read_count` is 107 and the refusal cause moves to coverage. The stub arm is still real and is still refused by the stage, under the distinct cause `ManifestOverlayNotResolved`.

---

## 5. What this receipt does not cover, named rather than left to be found

- **B4 materializes the DRIVER'S emission, not a second `gunbc compile` invocation.** `drive_compiler_closure_emit` returns one `Medium<String>`; it is written to `<txn>/candidate/src/lib.rs` and that file is what boundary 5 assembles. Substituting `gunbc compile --output-dir` here would produce a bigger, more impressive candidate and would be exactly the independent-green-components move this receipt exists to refuse. If the driver's single-medium emission cannot be assembled as a crate, boundary 5 refuses and the receipt says so; that seam is a finding, not a thing to route around.
- **The 29 `compile_error!("unsupported mock expression")` invocations** are counted on the emitted candidate BEFORE assembly (`EmittedHardRefusalsPresent { count }`), never on cargo's module graph. At subject `00_compile.dag` the count is not reached, because B2 refuses first.
- **This transport is not enrolled in CI.** The witness floor discovers the receipt ALGEBRA (`src/v2/test/claim/workflow/product_receipt_test.dag`, five claims including the discriminating RED); the transaction itself needs three release binaries and a cargo build and is a scheduled job, not a floor witness. Enrolling it is a separate decision under the CI re-add queue that DESIGN's CI paragraph declares.

---

## 6. Reproducing

```
cargo build --release -p v1-compiler --bin gunbc --bin discover_source_root_ingest --bin cssl_assemble
./target/release/gunbc run --source-root dag --source-root src/v2 \
  --entry src/v2/workflow/product_receipt_transport.dag \
  --function run_seven_boundary_product_receipt
cat target/product-receipt/receipt.txt
```

Any other subject:

```
./target/release/gunbc run --source-root dag --source-root src/v2 \
  --entry src/v2/workflow/product_receipt_transport.dag \
  --function run_product_receipt_for_entry --arg entry=<path.dag>
```
