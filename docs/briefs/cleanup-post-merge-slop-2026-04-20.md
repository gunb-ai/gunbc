# Holistic cleanup — post-2026-04-20 merge slop `(M)`

## Context

Today's merge wave dissolved ~5K LOC of hand-authored Rust (emit_go, emit_python, parse, serialize, variant_payload all truly deleted). But SG-0 census went from 74 → 80 (+6), with three integrity gaps:

1. **`src/v3/compiler/parse_parser_body.txt`** (1350 lines of hand-authored recursive-descent parse algorithm) is on main, NOT on SG-0 census. The ratchet's `.rs`-only pattern skips it. PR #589 retired `parse.rs` from the census but the algorithm moved into this `.txt` file — net measurement is misleading.

2. **Ratchet-escape class**: any `.txt` scaffold file under `src/v3/compiler/` currently escapes SG-0's measurement. Future workers can repeat the pattern silently.

3. **P0 render regression test runs 29.7s** (tripped the 2s ratchet on #595's merge commit; fixed post-merge by adding to `scripts/slow-test-exemptions.txt`). 15× over budget is a real paydown target, not a permanent exemption.

Plus: **38 tests total on the slow-test-exemption list** — worth auditing that each has a reason + paydown reference.

## Read first

- `src/v3/compiler/parse_parser_body.txt` — 1350-line scaffold; header comment names the scaffold class honestly
- `src/v3/compiler/tests/integration/sg0_census_test.rs` — `EXPECTED_HAND_AUTHORED` list + the partitioning logic (where it filters by path pattern)
- `src/v3/compiler/build.rs` — `REGEN_OUTPUTS` (producer manifest; the generated-files source)
- `dsl/gunbc/compiler.dag` — `hand_maintained_src` list (related-but-separate authority)
- `scripts/check-test-timeout.sh` — 2s ratchet implementation
- `scripts/slow-test-exemptions.txt` — current 38 entries
- ROADMAP.md §"Tracked debts — 2026-04 analyses" — for the debt-row pattern
- My review comment on #589 for the Option B rationale: https://github.com/gunb-ai/gunbc/pull/589#issuecomment-4281812427

## Work

**1. Extend SG-0 ratchet to count non-.rs hand-authored fragments.**
   - `sg0_census_test.rs` currently partitions `.rs` files only. Extend to also walk `src/v3/compiler/` (non-recursive? or with explicit inclusion rules) for scaffold files whose extension isn't `.rs` but whose content is hand-authored Rust (or hand-authored anything that dissolves when the corresponding `.dag` authority lands).
   - Suggested inclusion rule: `.txt` files that are `include!`'d or text-inlined into generated Rust.
   - Add to `EXPECTED_HAND_AUTHORED` with a clear prefix distinguishing them (or a separate `EXPECTED_HAND_AUTHORED_FRAGMENTS` constant if the partitioning logic differs).

**2. Add `parse_parser_body.txt` to both census authorities.**
   - Add `"src/v3/compiler/parse_parser_body.txt"` to `EXPECTED_HAND_AUTHORED` (via the new fragment rule from step 1, or directly if simpler).
   - Add `"parse_parser_body.txt"` to `hand_maintained_src` in `dsl/gunbc/compiler.dag`.
   - This makes the SG-0 census measurement match reality: when the algorithm dissolves to structural `parse.dag`, both rows come off together.

**3. Add named-debt row to `ROADMAP.md` §"Tracked debts — 2026-04 analyses" (or a new "Post-merge debt" subsection if it's cleaner).**
   - Entry: **`parse_parser_body.txt` — 1350 LOC hand-authored recursive-descent parse algorithm**. 1-line fact + dissolution trigger ("structural `parse.dag` ownership via SG-2b proper or SG-3f surface reflection follow-on"). Owner: queued behind SG-3f.

**4. P0 render regression test paydown investigation.**
   - Profile `p0_std_render_repeat_string_test::std_render_repeat_string_and_indent_text_match_interpreter` — where is the 29.7s going?
   - Expected hot paths: v2 `compile_to_resolved` + interpreter oracle (per the current exemption reason).
   - Candidates for paydown: shared bootstrap via `OnceLock`, smaller fixture, split into narrower sub-tests with shared cache, or mark specific heavy assertions `#[ignore]` with explicit thesis-coverage notes.
   - Goal: under 2s for the default run. If truly intractable, document the structural blocker (e.g. "requires shared v2 bootstrap cache landing on TM-X"). Don't leave at 29.7s without a specific paydown trigger.

**5. Slow-test exemption list audit.**
   - 38 entries. Verify each line has: (a) a clear reason, (b) a ROADMAP or track reference for paydown.
   - Current format examples are good (the `four_fixture_*` entries name "ROADMAP Lane 3 Stage 3c prep"). Flag any entries that just have a test name with no reason/reference.
   - Output: a brief comment in the PR description listing any entries that needed updating, OR a note that all 38 are well-annotated.

## Acceptance

- `.txt` hand-authored fragments under `src/v3/compiler/` are counted by the SG-0 ratchet (test + logic update)
- `parse_parser_body.txt` appears on **both** census authorities (`EXPECTED_HAND_AUTHORED` + `hand_maintained_src`)
- `ROADMAP.md` has a named debt row for `parse_parser_body.txt` with dissolution trigger pointing at SG-3f / SG-2b-proper
- Either: (a) `p0_std_render_repeat_string_test` drops below 2s via documented optimization and is removed from the exemption list, OR (b) the exemption entry names a specific paydown trigger beyond "cold compile"
- Every entry in `scripts/slow-test-exemptions.txt` has a reason + paydown/roadmap reference
- SG-0 census count reflects honest hand-authored surface (may go UP by ~1 for `parse_parser_body.txt`; that's the correct direction)
- CI green (including the new ratchet logic)

## STOP-AND-ESCALATE

- **If the `.txt` ratchet extension reveals OTHER non-.rs hand-authored files** that aren't on the census — STOP, list them, propose rules separately. Don't silently extend the ratchet across a broader class than parse_parser_body.txt warrants.
- **If the P0 render test can't be sped up below 2s without unreasonable work** — STOP, name the specific structural blocker (e.g., v2 interpreter path doesn't share cache with v3), propose the paydown lane, keep the exemption with an explicit named trigger.
- **If the exemption list audit reveals entries that are paydown-ambiguous** (multiple tests might cover the same concern, or the "reason" doesn't justify permanent exemption) — list them; don't arbitrarily delete entries.
- **If adopting rule 1 forces a significant SG-0 ratchet refactor** (>200 LOC in the test itself) — STOP, propose the change separately before implementing.

## Non-goals

- **Not dissolving `parse_parser_body.txt`** — that's SG-2b proper / SG-3f surface reflection, a separate lane with XXL scope.
- **Not redesigning the exemption list infrastructure** — just audit the entries.
- **Not touching `.dag` parse authority or regen_parse** — the parse algorithm's dissolution is out of scope.
- **Not touching stage0 regen mechanics** — just the census measurement.
- **Not renaming `parse_parser_body.txt` to `.rs`** — that would confuse the regen pipeline; extending the ratchet is cleaner.

## Size

M. Expected delta:
- `sg0_census_test.rs`: +30 to +80 LOC (ratchet logic + new entries)
- `compiler.dag`: +1 line
- `ROADMAP.md`: +3 lines (debt row)
- `scripts/slow-test-exemptions.txt`: 0-20 LOC updates (audit annotations)
- Possibly `p0_std_render_repeat_string_test.rs`: optimization or annotation
- Expected: 1 PR, single worker, ~3-5 hour scope.

## Dispatch note

Claude-review (director) will review when the PR opens. Tag directly on STOP-AND-ESCALATE.
