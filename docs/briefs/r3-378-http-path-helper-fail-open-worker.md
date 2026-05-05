---
status: draft (wait-window; awaits R3 host restoration before dispatch)
authority parent: R3 Substrate Manager (#1739)
roadmap row: ROADMAP.md "http_path.dag `None => ""` fabrications" (P1 — fabrication / fail-open boundaries)
---

# R3 :378 — `parse_segment_tokens` fail-open helper fix

## Context

The top-level `parse_path_template` in `dsl/std/http_path.dag:37-69`
already returns the typed `PathTemplateParseResult` coproduct
(`Ok(ParsedPathTemplate)` | `MalformedPathSegment { segment, reason }`).
The descent into `parse_segment_tokens` does **not** propagate that
discipline: lines 95, 99, 107, 111 each match an `Option` from
`first` / `skip(1) |> first` and substitute `""` on `None`. Each `None`
arm is structurally unreachable when the surrounding length / split
checks are correct, but the substitution silently fabricates a
zero-length string if invariants ever drift, and is the exact pattern
the row names.

The 2026-05-04 R3 wait-window audit verified all four sites against
HEAD; the parent-session #828 audit cited the entry-range only and
missed these. Row remains REAL, not partially-dissolved.

## Slice

1. Read the four `match … None => ""` sites in `parse_segment_tokens`
   (`dsl/std/http_path.dag:88-112`) and confirm each surrounding guard
   makes the `None` arm structurally unreachable. The
   `count(before_and_rest) != 2` and `count(name_and_suffix) != 2`
   guards already early-return `MalformedPathSegment`; under those
   guards, `first` and `skip(1) |> first` are both `Some`.
2. Replace each `None => ""` with `None => MalformedPathSegment {
   segment: seg, reason: "internal: <site-specific>" }`, threaded as a
   typed early-return — same shape as the already-used coproduct
   return at `:91` and `:103`.
3. Confirm `parse_segment_tokens`'s return type is the coproduct (it
   already is — see the `MalformedPathSegment` early returns at `:91`
   / `:103`). No signature change required.
4. Update / add the smallest cementing test: a programmatic feed of a
   single-segment input that drives one `None` arm, asserting the
   typed `MalformedPathSegment` propagates rather than producing
   `[LiteralToken { text: "" }]`.

## Acceptance

- `cargo test --workspace --exclude v2-compiler-tests` green.
- `cargo test -p v2-compiler-tests` green.
- `cargo clippy --all-targets -- -D warnings` clean.
- ROADMAP row "http_path.dag `None => ""` fabrications" flips Open →
  Retired with PR sha + reasons in the row.
- `docs/debt/r3-debt-paydown-ledger-2026-05-02.md` row 41 flips Open →
  Retired with PR sha and the dissolution shape ("parser returns
  typed coproduct on every internal `None` path").

## STOP-AND-ESCALATE

- If any `None` arm turns out to be structurally reachable (the
  upstream guard does not in fact prove it `Some`), STOP — that's an
  invariant gap, not a fabrication fix; surface to R3 Substrate
  Manager (#1739) before introducing any new diagnostic surface.
- If lifting the inner returns requires changing `parse_segment_tokens`'s
  return type or the caller's match shape, STOP and surface — that's
  a different scope than the row names.

## Authority audit receipt

1. **Substrate exists?** `PathTemplateParseResult` coproduct +
   `MalformedPathSegment` already declared and consumed at top level
   (`http_path.dag:37-69`); helper consumes the same shape at `:91`
   and `:103`. No new substrate.
2. **Existing brief?** None for this row. Audit ledger references
   only (`docs/debt/r3-debt-paydown-ledger-2026-05-02.md` row 41).
3. **Design-doc match?** No design-doc; ROADMAP row plus INVARIANTS.md
   C-8 fail-closed are the authority.
4. **Citations live?** `dsl/std/http_path.dag:95,99,107,111` verified
   at HEAD by the wait-window audit.
5. **Carrier dissolves the bridge?** Yes — `MalformedPathSegment`
   already exists and is the typed failure shape the row's dissolution
   sentence names.

## Provenance

Drafted during R3 host-block wait window (2026-05-05) per parent
session #828 instruction. Ratification pending host restoration and
parent dispatch slot allocation.
