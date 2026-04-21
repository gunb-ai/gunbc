# SG-2c-2 — Next parser-owned table extraction `(M-L)`

## Context

PR #611 (SG-2c-1) proved the pattern: extract a parser-owned table from `parse_parser_body.txt` into `parse_tables.dag` (authority) + `parse_tables_generated.rs` (projection), consumed directly by the live parser path. The first extraction was the binary-operator symbol → `OperatorKind` table.

**SG-2c-2 continues the pattern with the next bounded data slice.** Explicitly framed as another bounded extraction — not parser retirement, not `parse.dag` authoring.

## Read first

- PR #611 — the prototype template. Read in full before proposing SG-2c-2 scope. What made it work: pure data table extraction, live consumer wiring in parser, fail-closed regen checks, no parser algorithm changes.
- `src/v3/compiler/parse_parser_body.txt` — the 1350-LOC staging file. Scan for additional tables that are pure data (strings → typed enum variants, keyword → item-form, etc.).
- `src/v3/compiler/parse_tables.dag` — existing authority; extend in the same shape.
- `src/v3/compiler/src/parse_tables_generated.rs` — generated projection; SG-2c-2 extends.
- `src/v3/compiler/src/bin/regen_parse_tables.rs` — regen pattern.
- `src/v3/compiler/tests/integration/sg2_parse_authority_test.rs` — the ratchet pattern for parser authority.

## Candidate tables (worker picks ONE)

Per the SG manager's note, the best candidates are:

1. **Item-keyword dispatch table** — maps source keywords (`fn`, `type`, `data`, `module`, `import`, `let`, etc.) to the item-form enum variants the parser dispatches on. Pure data, tokenizer-coupled. Likely ~30-50 LOC.

2. **Punctuation / operator mapping table** — maps single-character and multi-character punctuation tokens to their parser-facing identity. Pure data. Similar size.

Either is acceptable. Worker picks based on which has the smallest consumer-wiring surface at parse-time. Rationale goes in the PR body.

**Do not** pick:
- Tables that require parser-algorithm restructuring (SG-2c-2 is data extraction, not parser redesign).
- Tables whose consumers are scattered across many non-parser modules (widens the wiring surface).
- Tables smaller than ~20 LOC (not worth the ratchet overhead).

## Work

Follows the SG-2c-1 template:

1. **Verify** the chosen table is genuinely parser-owned data (not parsed semantically elsewhere) and not already covered by `operators.dag` / `tokenize.dag` / other existing authorities. Grep aggressively; if any equivalent exists, STOP and reclassify.

2. **Author** the table in `parse_tables.dag` alongside the existing binary-operator table. Match the naming convention and shape from SG-2c-1.

3. **Extend `regen_parse_tables.rs`** to emit the new table into `parse_tables_generated.rs`.

4. **Wire the live parser** in `parse_parser_body.txt` to consume the generated table instead of its inline data. Preserve parser behavior bit-identically.

5. **Extend the ratchet test** (`sg2_parse_authority_test.rs`) to cover the new table — structural snapshot or field-count check, matching the SG-2c-1 pattern.

6. **Verify** parse corpus output is bit-identical. Parse-side integration tests (`real_stdlib_parse_smoke`, `sg0_census`, etc.) all green.

## Acceptance

- One new table authored in `parse_tables.dag` with matching regen + generated projection
- Table consumed by live parser in `parse_parser_body.txt`; inline data deleted in the same PR
- Ratchet test extended to cover the new table
- Parse corpus snapshot unchanged (bit-identical parser output)
- `parse_parser_body.txt` LOC reduced by the extracted table's size (~30-50+ LOC)
- PR framed explicitly as **SG-2c-2 table extraction** — not parser retirement, not `parse.dag` authoring

## STOP-AND-ESCALATE

- **If Phase 0 verification reveals the chosen table is already covered elsewhere** — STOP, reclassify, don't duplicate authority. Either pick a different table or surface that the category is covered.
- **If consumer wiring requires changing the parser algorithm** — STOP. SG-2c-2 is data extraction, not parser redesign. Surface and propose as a separate lane.
- **If the ratchet reveals drift in the binary-operator table from SG-2c-1** — STOP and fix the drift first; don't layer a new table on top of a broken one.
- **If parse corpus bytes drift** after wire-in — STOP. Parser output must be identical.

## Non-goals

- **Not parser retirement** — `parse_parser_body.txt` stays (minus the extracted table's LOC).
- **Not `parse.dag` authoring** — that's SG-2c proper, still blocked on substrate capabilities (per SG-2c-1's framing).
- **Not touching tokenize/lower/infer/emit** — parser-only scope.
- **Not expanding ratchet semantics** — copy the SG-2c-1 pattern; don't generalize here.

## Size

M-L. Similar scope to SG-2c-1 (~1000-line PR with most of the weight in the generated projection + bootstrap regen chain).

Expected LOC delta:
- `parse_parser_body.txt`: -30 to -100 (extracted table)
- `parse_tables.dag`: +50 to +150 (authority + realization binding)
- `parse_tables_generated.rs`: +100 to +400 (generated projection)
- `regen_parse_tables.rs`: +20 to +50 (extended emit path)
- Test: +30 to +60 (ratchet extension)

Net hand-authored LOC delta: small negative on `parse_parser_body.txt`; generated content grows.

## Dispatch note

Director reviews. Primary signal: parser output unchanged + `parse_parser_body.txt` shrinks by the extracted table. Pattern-match against SG-2c-1's discipline throughout — same shape, same test disciplines, same framing.

After SG-2c-2 lands: ROADMAP SG-2c lane continues as "iterative table extraction." SG-2c-3, -4, etc. follow the same pattern until either (a) `parse_parser_body.txt` has no more pure data tables to extract, or (b) SG-2c proper (full `parse.dag` authoring) becomes unblocked by substrate work.
