# Realization-decision census

**Question:** where do modules independently choose storage, caching, resource access, authorization or placement? This read-only census describes pin `1dd3eda5696a10e8cb49700f37d61b40c304d326`, surveyed 2026-10-24; it implements no fixes.

| Read | Purpose / identity |
|---|---|
| [classes.tsv](classes.tsv) | Error classes, walls, controls and residuals; `class_id` |
| [findings.tsv](findings.tsv) | One decision at one symbol; `finding_id` |
| [modules.tsv](modules.tsv) | Hand-read modules; `module` |
| [receipt.json](receipt.json) | Pin, script hashes, slice commits, populations and agreement |
| [method.md](method.md) | Sampling, classification, merge rulings and limitations |
| [merge_report.txt](merge_report.txt) | Reconciliation, including problem cells without instance rows |

**Interpretation:** None/Home/Passed mean no local decision, declared ownership, or a received value. Rederived/Literal/Forked mean a repeated decision, an unowned literal, or a duplicated mechanism. L0–L5 sorts sites; a module carries its worst instance per dimension. Receiving the existing value, declaring a missing owner, or adopting the existing library are possible remedies, not implemented designs. “No owning authority” remains a gap.

**Limits:** rung 0 means no wall observed, not proved unguarded. Targeted reads in slices 03/04/05/06/09 make None/Home/Passed weaker than identified problems; slice 06 often names mechanisms rather than quoting specimens. The no-signal sample bounds an unread remainder and does not clear it; use the receipt and method for populations and confidence. Module ordering is reading order, not severity. Parent seeds were unreachable, so leads were recomputed without the planned seed cross-check.

Derived counts and rankings belong to these rows and `scripts/readme_tables.py`; this index does not maintain another copy. The recorded script hashes and raw findings remain unchanged.
