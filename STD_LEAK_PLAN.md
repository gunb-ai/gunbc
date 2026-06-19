<!-- WORKING ARTIFACT (not a committed parallel-ledger). Produced 2026-06-18 by an
audit of dsl/std/ for §3 layer-inversion leaks (operator briansrls flagged MoneyMicros/
Milliwatt). 54-agent workflow: 6 partition auditors + cross-cutting critic → per-finding
adversarial verification → synthesis. 35 confirmed leaks / 6 needs-operator-decision /
5 deliberately-not-flagged. Delete once the slices land. -->

# std/ §3 Layer-Inversion Correction Plan

## Diagnosis

One §3 inversion repeated across `dsl/std/`: the most-abstract layer **reaches downward** —
fixing the `Measure<Q,S,M>` scale param to a *consumer's* denomination (Micro because Hetzner
quotes `*_eur_micros`; Milli because a DRAM chip draws ~0.4 W; Nano because `CostAccount.time`
ticks in ns), embedding *cited vendor decode-specs* and *provider identifier specs* (SK-Hynix
DDR4 ORGANIZATION, GCP project-id/SA-email regexes), holding *per-vendor realization rows + the
match-dispatch that selects them* (cache backends, per-language emit rows, ANSI glyph table),
siting whole *product/ctrl domain models* (`placement_supply.dag`, `process_algebra.dag`) in
std, and even literally `import extdeps.shell`. In every case the **import arrow is inverted**.
The fix is almost always *additive-in-the-consumer-layer + delete/reword-in-std* — the agnostic
carriers (`Measure<Q,S,M>`, `MoneyAmount<S>`, `Watt`, the `CacheInterfaceFacts` shape, the
`*Syntax` coproducts, `CredentialSource`, `SymbolId`/`SemanticColor`) all correctly **stay**.

**Verified keystone:** `dsl/std/credentials.dag:13 import extdeps.shell` is the *only* literal
upward import in std — and `src/v2/lens/layering_imports.dag` does **not** catch it (it forbids
only `std/extdeps → v2.compiler.*`, never `std → extdeps`, despite classifying every file
`LayerStd | LayerExtdeps`). Tightening that lens to the actual §3 rule converts this one-time
audit into a permanent fail-closed gate.

---

## Work-slices (CONFIRMED leaks)

### Slice 1 — `credentials.dag` upward import (MECHANICAL, do first)
- `dsl/std/credentials.dag:13` `import extdeps.shell`; `pattern env_credential` (31-34, calls `shell.Env.Get`).
- §3: the only `import extdeps.*` in std; `env_credential` is a transport/realization handler ("sits peripheral, never in the interface").
- Home: move `env_credential` → `dsl/gunbc/auth/credentials.dag` (beside `gcp_secret_credential`); repoint sole caller `dsl/extdeps/github/auth.dag:39` (carry its `NonEmptyStr` cast). Keep agnostic `type CredentialSource = EnvVar { name }` in std; delete the import.
- Blockers: none std-internal. Ratified: no. Effort: **S**.

### Slice 2 — `placement_supply.dag` whole-file relocation (MECHANICAL)
- Whole module (`HostIdentity`, `PlacementSupplyRow`, `cpu_capacity_hz_row`).
- §3: a product/ctrl supply/demand/placement *market* (first-fit, cordoning), not a universal framework; header cedes `ComputeHost` authority to product and cites ctrl consumers downward.
- Home: `dsl/product/compute_fabric.dag` (already owns `ComputeHost` + `placement_supply_row` + first-fit allocator) or a sibling `dsl/product/placement_supply.dag`. Two importers repoint to product: `product/compute_fabric.dag` and `gunbc/ci_fleet.dag` (lone `HostIdentity` import) — both already product-or-further-out, direction stays valid.
- Ratified: no. Effort: **M**.

### Slice 3 — `process_algebra.dag` whole-file → ctrl (MECHANICAL)
- Whole module (`ProcessOperation`, `ProcessClosureRefusal`, `Attestation`, `ProcessGraph`, …).
- §3: header self-declares "models the ctrl/ decomposition algebra as substrate"; 25 ctrl citations (ctrl PRs #1192/#1195/#1197, `canCloseNode` reason codes).
- Home: `dsl/ctrl/process_algebra.dag`, `module ctrl.process_algebra` (the **public** `dsl/ctrl/` STAGED-contract layer — NOT `~/ctrl` private). `std.algebra`/`std.effects`/`std.types` imports stay valid (`ctrl ← std`).
- Blockers: none (zero consumers anywhere). Ratified: no (🟡 STAGED). Effort: **S**.

### Slice 4 — cache_interface: cited vendor rows + match-dispatch → extdeps (NET-NEW DIR)
- 5 cited-vendor `data … : CacheInterfaceFacts` rows (`gha_actions_cache_facts`, `sccache_local_facts`, `buildbuddy_cas_facts`, `cargo_target_dir_facts`, `rustup_toolchain_store_facts`, each with `VendorCitation`, 230-444); 2 gunbc-internal rows (`resolved_graph_cache_facts`, `parse_table_memo_facts`); the realization-selecting `cache_key_projection_provider_key` match (552-559) + in-file helpers; vendor-named enum tags (`SccacheInternalEncoding`/`CargoFingerprintEncoding`/`RustcInvocation`/`ToolchainSpec`).
- §3: mirror-image of the blessed `std/os.dag` (projection-only) vs `extdeps/os/*` (dispatch + cited rows). "A std projection that *matched* over its realizations would have to name them, inverting the import arrow."
- Home: new `dsl/extdeps/cache/{github_actions,sccache,buildbuddy,cargo,rustup}.dag` for third-party rows + the dispatch; the 2 gunbc-internal rows → `extdeps/realization/` (named in `realization.dag`'s own dissolution trigger) or product. Keep in std: `CacheInterfaceFacts` shape, all closed axis enums, agnostic encoding variants, projection fns. **Sub-fix:** de-literalize `cap_bytes: 10737418240` (10 GiB) on `resolved_graph_cache_facts:466` (business-policy default).
- Blockers: none — `realization.dag`/`cache_identity.dag` reference cache_interface only in comments; extdeps consumers pull only agnostic enums; zero consumers of the row symbols. Ratified: no (🟡 STAGED). Effort: **L** (net-new dir, mechanical).

### Slice 5 — per-language emit rows → extdeps/languages (DE-FORK, not a move)
- `languages.dag:~496-1419` — `rust_language`/`go_language`/`python_language`/`typescript_language`, `rust_spec`/`go_spec`/`python_spec`, ~64 per-target `data` rows (`rust_statements`, `rust_serialization`[serde], `rust_async`[tokio], `python_literals`, manifests `Cargo.toml`/`go.mod`, …).
- §3/§4: "a new target language is rows authored in `extdeps/languages/`, never an edit to the fold." serde-over-miniserde / tokio-over-async-std are ecosystem opinions.
- Home: `dsl/extdeps/languages/{rust,go,python}/*.dag` — **which already exist and already author "Mirrors …" duplicates** (`rust/emit.dag:155` "Mirrors rust_scaffold from std.languages"). This is a **reconcile/de-fork onto one type-set**, not a copy. typescript needs a new dir. Keep agnostic shapes in std.
- Blockers: none std-internal (`render.dag:23` imports only shapes; std `data` rows have zero live consumers) — but crosses std/extdeps + dual v1/v2 representation → route with a §6 dissolution marker. Ratified: no. Effort: **L**.

### Slice 6 — ANSI glyph realization → extdeps render-target (NET-NEW DIR)
- `symbols.dag:185-204` — `AnsiMapping`, `ansi_mappings` (cited `\x1b[38;5;NN` table), `ansi_code` (filter-over-realization dispatch); comment "Matches gunb.ai/pkg/fermi/colors.go".
- §3: ANSI is one of N (ANSI/CSS/CI); a concrete escape table tuned to a named product binary = `st_mode` single-realization-privilege + selecting dispatch.
- Home: new `dsl/extdeps/terminal/ansi.dag`; restate the comment on ECMA-48 / xterm-256, not the product binary. Keep agnostic axes in std (`SemanticColor`, `Tier`, `SymbolId`, `resolve_symbol`, `symbol_color`).
- Blockers: none (`AnsiMapping`/`ansi_mappings`/`ansi_code` have zero consumers; `render.dag:22` imports only `SemanticColor`/`SymbolId`/`Tier`/`resolve_symbol`). Effort: **M**.

### Slice 7 — vendor decode-specs: delete duplicated cited rules from std (MECHANICAL)
- `memory/types.dag:50` (SK-Hynix DDR4 `ORGANIZATION {4=x4,8=x8,6=x16}`) and `:55-56` (rank-derivation/dmidecode rule).
- §3: single-authority — the identical anchor already lives in `extdeps/memory/sk_hynix.dag:27-41`. **Delete** the duplicated text; keep the universal framing (`chip_width: BitWidth`, `rank_count: Int`). `DramManufacturer` enum **stays** (sanctioned, CpuVendor-analogous).
- Blockers: none. Effort: **S**.

### Slice 8 — GCP provider types → extdeps/cloud/gcp (BLOCKED on Slice 9)
- `types.dag:360-365` `GcpProjectId` / `ServiceAccountEmail` (GCP project-id/SA-email regexes); header `// --- GCP / OIDC types ---`.
- §3: provider identifier specs are "what the API returns" → extdeps; `extdeps/cloud/gcp/gcp.dag` already models `GcpProject{project_id}` / `GcpServiceAccount` (single-authority fork).
- **ORDERING blocker:** `CredentialFlow.WorkloadIdentity` (`types.dag:394`) embeds `service_account: ServiceAccountEmail?` — moving the GCP types up forces a forbidden upward import unless CredentialFlow's GCP arm is de-fused first (Slice 9). Also reconcile the parallel `WorkloadIdentity{service_account: NonEmptyStr}` in `gunbc/workflow/types.dag:303`. Effort: **M**.

### Slice 9 — CredentialFlow acquisition-strategy → gunbc.auth / consolidate (DE-FORK)
- `types.dag:384-398` `CredentialFlow = WorkloadIdentity | InteractiveAuth | Stored | PlatformInjected | Chained`.
- §3: acquisition strategy = dispatch = realization → peripheral; a near-duplicate **already exists, correctly placed**, at `gunbc/workflow/types.dag:300-308` (`CredentialResolution`/`CredentialBinding`).
- Home: delete `CredentialFlow` from std and **consolidate onto** `CredentialResolution` (single authority), not re-home verbatim. Blockers: zero `.dag` consumers (only the v1 bootstrap-seed mirror, which §7 shrinks to zero). Effort: **M**.

### Slice 10 — `policy.dag` dead orphan (DELETE)
- `policy.dag:1-7` `WarningPolicy = DenyAll | Default`. Dead (zero consumers); live realization is `lint_args: ["-D","warnings"]` in `tools/rust_gates_ci.dag`. **Delete.** Effort: **S**.

### Slice 11 — `HostRamSupplyFacts` → product (or delete)
- `memory/types.dag:131-137` (`HostRamSupplyFacts`; "srv1 degraded case"; "shared stopgap with Hetzner catalog"). Nominal-vs-observed *supply* is a deployment fact; `ComputeSupplyFacts` already exists at `product/compute_fabric.dag:230`. Zero consumers anywhere → relocate or delete; scrub srv1/Hetzner citations. Effort: **S**.

### Slice 12 — comment-only downward citations (BATCH, MECHANICAL reword)
One prose pass; no symbol moves, imports intact:
- `measure.dag:78-86` (Quantity.Power names `product.hardware_selection` + `std.cpu tdp_watts`) — reword to SI (W=J/s) + point at the witness file. **⚠ touches Q-Unit-5 RATIFIED trail → flag gunbc#828.**
- `currency.dag:52-53` (`Eur` "live consumer (Hetzner)") — reword to ISO 4217 only; enum stays.
- `os/types.dag:11-13` (optional distro "because Hetzner binds at provision time"; srv1/srv2) — reword to "kernel intrinsic, distro contingent".
- `process.dag:10-23, 28-29, 43-47` — drop phantom result-type names + `cli_run.rs` narrative; keep POSIX `ProcessExit`. `exit_code_misuse=2` dead → delete; `exit_code_general_error=1` → ground on a standard or move to `examples/`.
- `behavioral.dag:3-4` ("ported from the-gunbai … Lane 4 extdeps") — reword to universal operation-behavior vocab. (Same "Ported from the-gunbai" debt at `types.dag:385`.)
- `medium.dag:1, 29-30` (project-phase tag; `product.compute_fabric.StorageMedium` pointer) — reword. **⚠ inside a "NAME DISCIPLINE §3 — SIGNED OFF eager-boar-790" block → coordinate with sign-off owner.**
- `markup.dag:4-9, 150-159` + `html_markup.dag:7-8` + `react_markup.dag:3` (gunbhub #4927 / React #4934 / PR numbers) — reword to "format-agnostic element tree, one convergent serializer".
- `realization_measurement.dag:41` `data v1_eval_expr_measure_handler_id = "host:v1-interpreter.eval_expr_self_time"` — a *value*, not just a comment: a v1 host-binding literal in std → move to new `extdeps/realization/v1_interpreter.dag`. Keep `RealizationMeasureEffect`.
- Effort: **M** overall (S each; two carry escalation flags).

### Slice 13 — CI-floor schedule policy → workflow (BLOCKED on v2 self-compile)
- `realization_schedule.dag:99-179` `ScheduleLensViolation` / `schedule_lens_verdict_for_ci_floor` (hardcodes batch0==1 / compile-first / `"__discovery_corpus__"`-not-first / batch1≥2).
- §3: "compile-clean gates the rest" is the gunbc CI workflow fact the *plan* adds — workflow, not universal scheduling.
- Home: workflow, beside `src/v2/workflow/ci_floor_plan.dag`. Keep an agnostic gate-first/fan-out lens in std, parameterized on **both** gate label and corpus label.
- **Blocker:** gated — `realization.dag` re-exports the symbol; per `ci_floor_plan.dag`'s documented bridge, the verdict can't move to v2.workflow until v2 reaches self-compile-clean (a `dsl→v2` import would drag v2 into `DslCompileCleanGate`). Sequencing, not a unilateral edit. Effort: **M** (after blocker clears).

---

## Ordering

**Wave A — safe, mechanical, no std-internal consumer, no ratified substrate (do now, parallelizable):**
Slice 1 (credentials upward import) · Slice 3 (process_algebra → ctrl) · Slice 7 (SK-Hynix delete) · Slice 10 (policy.dag delete) · Slice 11 (HostRamSupplyFacts) · Slice 2 (placement_supply → product) · Slice 6 (ANSI → extdeps/terminal) · Slice 4 (cache rows + dispatch → extdeps/cache, L but zero breakage).
**+ Tighten `layering_imports.dag` to forbid any `std → extdeps` import and CI-enroll it** (the permanent gate; catches Slice 1's class).

**Wave B — de-fork / cross-boundary, need a §6 dissolution marker + reconcile vs an existing duplicate:**
Slice 5 (per-language rows — reconcile "Mirrors …") · Slice 9 (CredentialFlow → `CredentialResolution`) · Slice 8 (GCP types, **must follow Slice 9**).

**Wave C — comment reword batch (mechanical, two items brush ratified/sign-off threads):**
Slice 12; `measure.dag:78-86` flagged to gunbc#828; `medium.dag` coordinated with eager-boar-790.

**Wave D — blocked on external sequencing:**
Slice 13 — gated on v2 self-compile-clean.

**Ratified-substrate note:** `measure.dag` carries Q-Unit-1..5 RATIFIED markers (gunbc#828). The
agnostic carriers — `Measure<Q,S,M>`, `Quantity`/`Scale`, `MoneyAmount<S>`, `Watt = Measure<Power,One,Nat>`
— **STAY** and are NOT changed. The scale-fixed aliases (`Milliwatt`, `MoneyMicros`, `NanosecondDuration`)
are *additions* on the ratified carrier; relocating them is additive-in-consumer + delete-from-std,
not a ratified-type change. BUT Q-Unit-5 *deliberately* adopted "mint the scale alias with its named
consumer" (construction-over-ratchets / anti-speculation), so reversing that policy routes through
gunbc#828 (see open questions).

---

## Needs-operator-decision (open questions)

1. **MoneyMicros** (`measure.dag:332`): (A) move alias+constructors to `product/` (compute_fabric/hardware_selection are the typed consumers; Micro driven by Hetzner) and delete from std; or (B) keep the alias in std, remove only the "Demanded by Hetzner" citation. 0 std-internal consumers; Q-Unit-5 RATIFIED → gunbc#828.
2. **Milliwatt** (`measure.dag:285`): (A) move to `product/hardware_selection.dag` (sole consumer); or (B) add agnostic `PowerAmount<S> = Measure<Power,S,Nat>` to std (mirroring `MoneyAmount<S>`) and instantiate `<Milli>` downstream. **B is the symmetric/cleaner fix.** 0 std-internal consumers; gunbc#828.
3. **NanosecondDuration** (`measure.dag:256`): has **3 std-internal consumers** (`realization_schedule`/`realization_measurement`/`verification`), themselves realization-shaped. (A) relocate the whole cost/realization Nano cluster together to `extdeps/realization`/product; or (B) keep it as the ratified std timing grain. *Not cleanly extractable — likely (B) unless the cluster moves wholesale.*
4. **TargetArchitecture** (`architecture_profile.dag:18`): byte-identical within-std nickname for `CpuArchitecture` (DESIGN §3 names this exact pair). Dissolve-on gated on **snappy-stag-903** (`std.types` must export `Option`, reached via `NonEmptyStr?` in `std.cpu.types`); extdeps inhabitants reference `TargetArchitecture::*`. (A) wait then repoint+update extdeps; or (B) unblock Option-export first.
5. **GCP types + CredentialFlow arm** (Slices 8/9): (A) move both GCP types to extdeps + CredentialFlow's `WorkloadIdentity` takes an agnostic principal/subject; or (B) relocate the GCP-specific arm with the types. Also: reconcile parallel `WorkloadIdentity{NonEmptyStr}` in `gunbc/workflow/types.dag:303`.
6. **`standard_symbols` glyph table** (`symbols.dag:103-160`): no render/terminal realization layer exists yet (whole render stack is in std); table is in flux ("restore when tokenizer handles UTF-8"); sole consumer is std-internal (`render.dag span_width`). (A) build out `extdeps/render`/`extdeps/terminal` + dispatch and migrate the glyph rows (pairs with Slice 6); or (B) accept as a sanctioned canonical default for now.
7. **`max_dram_die_density`** (`memory/types.dag:107`): the 2 GiB ceiling is JEDEC-cited (sanctioned) but DDR4-pinned with no generation axis. (A) add a `Ddr4|Ddr5` enum + make the ceiling a function of generation with per-gen JEDEC ceilings as cited extdeps rows; or (B) leave as DDR4 scaffold. *Modeling-debt, not a leak per se — the realizability guard stays in std regardless.*

---

## Sanctioned / deliberately NOT flagged (don't re-flag)

- **Closed categorical vendor enums** — `DramManufacturer = SkHynix|Samsung|Micron`, `CurrencyCode = Eur|Usd`, single-variant distro enums. DESIGN §3 blesses these (CpuVendor pattern). Only attached *cited decode-specs/rows* move, never the enum.
- **Base-scale SI carriers** — `Watt = Measure<Power,One,Nat>`, `Duration := Measure<Time,One>`, `ByteSize`, `BitWidth`, the agnostic `Measure<Q,S,M>` / `MoneyAmount<S>` / `Quantity`/`Scale`.
- **`types.dag` MimeType + file_types comment** — RFC/IANA citation + dead comment; set-theory-grounded partition. Stale-comment hygiene at most.
- **`realization.dag` / `realization_measurement.dag` module headers** — correctly-directed §3 layer-split docs + §6 dissolution triggers (point downward *as destination* — the correct arrow). (The one real def-level leak there, `v1_eval_expr_measure_handler_id`, is in Slice 12.)
- **`serialization.dag` VariantNaming/Encoding/WireFormat** — parameterized transforms (`StripPrefixAndSnakeCase{prefix: String}`); affixes live as extdeps rows; comments name generic classes.
- **`markup.dag:174-236` `validate_href`/`href_is_allowlisted`** — fail-closed XSS allowlist (the §5 mechanism), the consolidating single authority.

---

## The right-up-front fix (recommendation)

**Yes to both, and they reinforce each other.** (a) Adopt as a standing rule that **std holds only
parametric agnostic carriers** (`Measure<Q,S,M>`, `MoneyAmount<S>`, `PowerAmount<S>`, the
`*Syntax`/`CacheInterfaceFacts` shapes, the closed categorical enums) and **every scale-fixed alias,
cited vendor row, and realization-selecting dispatch lives in the consumer's layer** — so almost all of
this is *additive-in-extdeps/product + delete-from-std* and the ratified `measure.dag` carriers never
change. (b) Build a **compile-time lens over the Node tree that gates `dsl/std/` against downward
citations**, fail-closed on: (i) any `import extdeps.*`/`product.*`/`gunbc.*`/`ctrl.*` originating in
`std/` (would have caught `credentials.dag:13`, which `layering_imports` misses today), (ii) any std
`match` over realization-tagged variants (the cache/ANSI dispatch tell), (iii) — best-effort pending
the structural model — std prose naming a known downstream symbol/vendor/PR. Tighten
`layering_imports.dag` to the actual §3 rule and CI-enroll it: this converts the one-time audit into a
permanent gate so the inversion cannot silently re-accrete.
