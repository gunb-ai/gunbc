# extdeps / T-4 lane — worker closeout receipt

**Session:** `calm-bear-268` (child of `bright-crane-876`)  
**Work item:** `node://adhoc-1d49a1f3-bf2` — *bright-crane-876 extdeps/T-4 lane closeout*  
**Tree:** `gunbc` @ `17e98967e` (merge of [#3322](https://github.com/gunb-ai/gunbc/pull/3322) on `main`, 2026-05-18)  
**Authority:** `src/v4/TASKS.md` T-4 / parallel-fill block, `src/v4/DECISIONS.md`, per-file headers (not this receipt).

## What was verified

Header-level audit of `src/v4/extdeps/**` and the five Shape-A language files under `src/v4/extdeps/languages/`, plus cross-read of `docs/briefs/r4-program-dispatch-plan.md` §2 for drift.

### T-4 — five Shape-A language models (`extdeps/languages/*.dag`)

| File | Header signal (fact-bundle / resolver posture) |
|------|--------------------------------------------------|
| `rust.dag` | `CP-3229-GREEN-TERMINAL`; scalar / overflow carriers present; D2b deferrals cited |
| `python.dag` | `CP-3229-GREEN-TERMINAL`; numeric tower + singleton slice |
| `go.dag` | `CP-3229-GREEN-TERMINAL`; width/kind sums + overflow disposition |
| `cpp.dag` | **🟡** `feature:t4-cpp-scalar-ladder` — T29-ABI + D2-REV resolver slice; not parity with rust/go/python terminal posture |
| `typescript.dag` | **🟡** “T-4 fact-bundle Phase-3 rework after T-3/T-29/T-30/T-25-core” |

**Conclusion:** three of five Shape-A slices are on the green terminal ledger called out in headers; **cpp** and **typescript** remain explicitly gated. This does **not** satisfy a literal reading of “all five fact-bundles ratified (B).” The **keystone HOLD** on T-4 in `r4-program-dispatch-plan.md` (P1-KEYSTONE / T-25-core / T-30 / T-29 cluster, residual [#3277](https://github.com/gunb-ai/gunbc/pull/3277) per that plan) remains consistent with `TASKS.md` — no de-classification claimed here.

### T-4.5 — `process.dag` + `file_system.dag`

Both headers claim **T-4.5 modeled 2026-05-16** with POSIX anchors; `file_system.dag` retains **🟡** coproduct markers under `DECISIONS.md` OS-1 (expected until refinement / fact-density work lands).

### T-4.6 — `extdeps/formats/*`

Seven files on disk: `json`, `yaml`, `csv`, `toml`, `json_schema`, `openapi`, `spice`. There is **no** `src/v4/extdeps/formats/sql.dag` yet; SQL DDL under Theme-A remains the separately scheduled extension described in `TASKS.md` T-4.6 / T-16 notes (do not conflate with this formats directory census).

### T-4.7 / T-4.8

- `frameworks/react.dag` — explicit **scaffold** (“fill per TASKS.md T-4.7”).
- `coordination.dag` — **Modeled with tracked scaffolds** (B3 row in `DECISIONS.md` still relevant).

### T-4.9 … T-4.14 (stress / probe lanes)

On-disk headers contradict the *derived* “NOT STARTED” wording still shown for some rows in `r4-program-dispatch-plan.md` §2:

- **T-4.9** `verilog.dag` — header: **T-4.9 PASS (IN-B)** + CP-3229 terminal / side-ledger IDs.
- **T-4.10** `formats/spice.dag` — present (plan already marks LANDED; rework-obligation note remains operator-owned).
- **T-4.12** `llvm_ir.dag` — header: **T-4.12 PASS (B2-OMNI)** (aligns with plan).
- **T-4.13** `machine_code.dag` — substantial model + **🟢 P4-3208** ledger tag (not “absent”).
- **T-4.14** `ptx.dag` — header: **T-4.14 PASS (IN-B)**.

T-4.11 English boundary file path is per `TASKS.md`; not re-audited in this pass (Lane B / verification.dag coupling).

## Hand-off

- **T-4 manager (`vivid-carp-207`):** consider a **table-only** refresh of `docs/briefs/r4-program-dispatch-plan.md` §2 for T-4.9 / T-4.13 (and optionally T-4.6/sql row clarity) so the discussion artifact matches headers — without folding any unrelated freeze work (#3280 custodian remains sole owner per operator routing).
- **Operator / parent:** close the dashboard work item when this receipt + any manager edits are accepted; no PR was required for this leaf.
