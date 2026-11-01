# P3 design note: a "respell to declaring identity" row kind in `v2.workflow.roster_import_sweep` (NOTHING BUILT)

Assignment: tidy-ant-251 (P3, queued behind the P2 verdict). Design first, code after review. Status of P2: run r2 refused at resolve on a defect in the witness itself (fixed at 93cd69197b00); the wave-1 table is still unmeasured, so every count below that comes from the witness is a bet until r3.

## What the sweep does today, and the gap

`RosterHeadRow { call, key, import_name, home, labels, head }` moves a FREE CALL HEAD: `KeepHead` appends the import, `RenameHead` renames the head (plus positional labels and appended constant args). `RosterRowSelection` picks rows (`EveryRowOfHome | OnlyCall | OnlyOccurrences`). It rewrites call heads and the import. XL3 respells REFERENCES (type positions, value references, call heads) to the declaring identity, with resolution unchanged. No existing row can express "this name, at these sites, spelled as this path", because a row today binds a call spelling to a home and ADDS an import; XL3 adds no import and changes no binding.

## Row shape (a sibling sum, not a flag on RosterHeadRow)

```
type RosterRespellRow {
  name: String              // the bare spelling as authored: "Bool"
  key: String               // row identity, as RosterHeadRow.key
  declaring: QualifiedName  // the ONE declaring path: std.types.Bool
}
```
One row per (bare spelling, declaring path); a spelling that resolves to two declarations in one wave is two rows selected by `OnlyOccurrences`-style site keys, never a heuristic. Rows are DATA (the wave-1 table of the witness is the same fact: declaring path + count), so the table and the row set are one authority: the witness table is derived from, or compared with, the rows, never retyped.

## What it rewrites

Only reference sites whose bare spelling is one the row names AND which the resolver binds to `row.declaring` in the file's own ingest (the same `occurrence_resolutions` read the witness uses). A site binding elsewhere (a local, a parameter, a same-leaf wrong declaration) is untouched and, if the row demanded it, REFUSES (`RefusalSpellingBindsElsewhere`), matching the existing no-fallback rule for occurrence keys. Imports are unchanged. Open point (not decided here): whether the module must keep importing the name for the qualified spelling to resolve; `std.declaring_identity_spelling` `authored_is_suffix_of_canonical` is the authority on the accepted suffix relation and the P2 run shows it empirically.

Not rewritten: strings, comments, namespace tokens, and anything the route cannot see (the known route limit: the ingest-fed collector does not see qualified references in some positions). Those are dispositions, not a fallback.

## How the identity witness gates each batch

Per batch (one touched module set): (1) the witness at BASE must hold on the unmodified tree (table = rows); (2) the generator emits the batch; (3) the SAME witness at HEAD must hold with identical declaring-path counts, so a respelling that moved a reference to another declaration changes a count or adds an out-of-table path and refuses; (4) the wrong-declaration RED (same leaf, other declaration) must refuse at both. A batch lands only with all four. The witness is one claim module per wave (`src/v2/test/claim/long/`); the generator never reads the census TSV style inputs as a landed artifact.

Cost fact from r2: the native context for one wave witness was about 24 minutes wall at 13.9 GB cgroup peak, so a batch gate is a ledger job, not an inline step; batching by wave is cheaper than per-module runs only if several modules share one resolved tree (wave 2 imports wave 1's helpers for that reason).

## Composition with KeepHead / RenameHead: one sweep, one pass

The respell rows are a second row list on the SAME sweep input, not a second sweep. The sweep already reads each module once (the companion pass of stack/sweep-cost); it gains one more rewrite family applied in that pass:

- Head rows (`KeepHead`, `RenameHead`) select call-head occurrences by `home`/`key`; respell rows select reference occurrences by `name` bound to `declaring`. The two select over disjoint occurrence kinds, so one occurrence is claimed by at most one family; an occurrence both families claim REFUSES (`RefusalOccurrenceClaimedTwice`), never first-wins.
- Edits from both families are spliced into one text per module from the one occurrence read, so a batch emits one candidate text per touched module and the identity witness runs once per batch at base and at head, over that single result.
- The import edit stays the head family's alone: respell rows carry none, so a wave using only respell rows leaves every import line byte-identical, which is what the witness's unchanged-resolution claim then checks.
- Row selection (`EveryRowOfHome | OnlyCall | OnlyOccurrences`) applies to head rows as today; a respell row's site set is `OnlyOccurrences`-keyed or every bound site of the name in the module, and the choice is part of the row, so the batch reading is data.

## Consumers and the executing route

The row kind's consumer is the existing generator entry (`roster_import_sweep` run over a `RosterHomeScope`), exercised by a control that respells a fixture and a RED that refuses a site binding to a wrong declaration. No new gate and no census.

## Open questions for review

1. Is the respell row the same type as `RosterHeadRow` with a third `RosterHeadRewrite` arm, or a sibling? I lean sibling: `home` and `import_name` mean something different there, and a KeepHead row cannot carry a meaningless import.
2. Same-module references (sunny-bee-153's note on `authored_is_suffix_of_canonical`): whether they mint occurrence records is decided by the r3 table, then written here as a rule.
3. `table_holds` answers true on an EMPTY table; the gate must demand at least one row.
