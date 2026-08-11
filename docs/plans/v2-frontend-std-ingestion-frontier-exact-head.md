# The v2 frontend's std-ingestion frontier — PARTIAL exact-head re-observation (10 of 15 measured)

**Status: measurement receipt, not a plan.** Produced 2026-08-11 at main `c03be069687`, with the
canonical v2 frontend fns driven from an ordinary `.dag` witness rather than a scratch host bin.

This does **not** replace [`v2-frontend-std-ingestion-frontier.md`](v2-frontend-std-ingestion-frontier.md).
That document is the historical receipt, revision-scoped to worktree HEAD `e588031201`, and it stays
unchanged as history. This one re-observes 10 of the same 15 members at an exact current head,
because the scheduling conclusion drawn from the historical receipt — *five bounded normalize
repairs, then three syntax gaps* — no longer describes the tree.

## How it was measured

The same canonical route as the historical receipt — `v2.compiler.tokenize` `lex_walk_artifact`
(rules `dag_lex_rules`) → `v2.compiler.parse` `parse_module` (grammar `dag_grammar`) →
`v2.compiler.normalize` `normalize` — driven per member through
`gunbc.tools.frontier_ingestion_probe` `classify_member`, which reads the member with
`filesystem_read` and walks the pipeline **once** per member.

Two measurement disciplines are load-bearing and both were violated by earlier drafts of this
work before being corrected:

**Stages are distinct states, never a zero-reasons collapse.** A first draft of the probe returned
an empty reason list for *both* "normalize accepted" and "lex/parse rejected". That is the
empty-observation narrow DESIGN names: ⊥-as-answer conflated with ⊥-as-ignorance. `classify_source`
now matches each stage's `Outcome` in sequence and returns a distinct state, so an unlexable member
can never read as an accepted one.

**Rejection reasons are read by containment, never by head.** `rejected_with_pending` PREPENDS the
pending accepted-carrier diagnostics to a later rejection, so the head of the list is routinely a
wrapper-retention carrier while the actual refusal sits further down. Every reason read here is
`reasons_contain` over the whole reason list. (The instrument collects head plus tail and does not deduplicate; an earlier draft of this sentence claimed a deduplication step that the code does not perform.)

## The rows

Each row below is one executed `classify_member` call at this head. `NOT RE-OBSERVED` is its own
state and is never filled in from the historical receipt.

| module | stage at `c03be069687` | historical (`e588031201`) | |
| --- | --- | --- | --- |
| `src/v2/test/claim/manual/cross_tree_constructor_binding_test.dag` | **PARSE** | accepted | **diverged** |
| `dag/std/primitives.dag` | ACCEPTED | accepted | |
| `src/v2/std/collection.dag` | ACCEPTED | accepted | |
| `src/v2/std/witness.dag` | ACCEPTED | accepted | |
| `dag/std/types.dag` | LEX | lex | |
| `dag/std/algebra.dag` | **LEX** | normalize | **diverged** |
| `src/v2/std/logic.dag` | NORM_RETAINED | normalize | |
| `src/v2/std/optional.dag` | NORM_RETAINED | normalize | |
| `src/v2/std/diagnostic.dag` | NORM_RETAINED | normalize | |
| `dag/std/occurrence_identity.dag` | NORM_RETAINED | normalize | |
| `src/v2/std/node.dag` | **NOT RE-OBSERVED** | parse | probe exceeded its 900s budget |
| `dag/std/content_hash.dag` | *pending* | parse | |
| `src/v2/std/live_tree.dag` | *pending* | normalize (graft) | |
| `dag/std/error_primitives.dag` | *pending* | normalize (graft) | |
| `src/v2/std/algebra.dag` | *pending* | **uncaptured** | never captured in either receipt |

`NOT RE-OBSERVED` is recorded for `src/v2/std/node.dag` because the interpreted probe did not
complete inside its budget, not because the member was found to be well-formed. A budget
interruption is an interruption plus a lower bound on cost — it is never a verdict about the
subject, and it may not be discharged by copying a value measured on a different tree.

**This receipt is PARTIAL and may not carry a scheduling conclusion on its unmeasured rows.**
Ten members produced an executed result; one is `NOT RE-OBSERVED` on a budget interruption; four
were never run to completion. The four `pending` rows and the `NOT RE-OBSERVED` row are **not**
evidence of any stage, and no claim below rests on them. Completing the batch is owed: the
interpreted instrument exceeds its budget on the larger members, so a batched or realized
instrument is the precondition for a genuinely complete 15/15 table.

## The population, restated

- **normalize retention contract violations: 4** — `logic`, `optional`, `diagnostic`,
  `occurrence_identity`. The historical receipt's five became four when `dag/std/algebra.dag`
  moved to lex.
- **lex walls: 2** — `types.dag` and `algebra.dag`, and they are **independent causes**
  (see below); one repair does not close the other.
- **the entry itself now rejects**, which the historical receipt's "behaviour differs by POSITION"
  section treated as the accepting side of its open question. That asymmetry cannot be
  re-derived from these rows as stated, and the open question is not closed by this receipt.

## What changed against the historical receipt

### `dag/std/algebra.dag` moved from normalize to lex, and its wall is attributed

The historical receipt lists this module as a normalize-stage rejection carrying the
wrapper-retained + not-well-formed pair. At this head it **does not lex**, with the file read
verified non-empty. It is therefore not a member of the normalize contract-violation population at
all.

Its lex wall is attributed, and the attribution is **not** a character-class gap:

**`dag_lex_rules()` registers no annotation rule for `//`.** `v2.extdeps.languages.dag`
`dag_line_comment_fidelity` is `FailClosed { feature: DagLineCommentFailClosed }`, and the
production rule list contains whitespace, string, int, ident, keyword and literal-operator rules
with no comment or annotation rule among them. A `//` line is therefore not lexed as a comment at
all: it tokenizes as two `dag_token_slash` tokens followed by whatever the payload happens to
tokenize as.

A comment consequently "lexes" **by accident**, exactly when every character in its payload has a
semantic lex rule, and fails at the first character that does not. Executed minimal pairs, all with
an identical surrounding module:

| comment payload | lexes | why |
| --- | --- | --- |
| `// a b` | accepted | `/` `/` ident ident |
| `// a - b` | accepted | `-` is `dag_token_minus` |
| `// a — b` (U+2014) | **rejected** | no rule for the em-dash |
| ``// a `s` b`` | **rejected** | no rule for the backtick |
| `// a @ b` | **rejected** | no rule for `@` |
| `data d: String = "dash —"` | accepted | em-dash inside a **string literal** is string content |

`dag/std/algebra.dag` fails on both counts: it carries backticks and an em-dash inside `//`
annotations near the end of the file.

Two consequences are worth stating explicitly, because both were nearly got wrong here:

- **The ASCII specimens are false controls for comment handling.** `// a b` passing does not show
  that comments work; it shows that its payload happens to be lexable as ordinary semantic tokens.
  Any control asserting only that lexing returned `Accepted` cannot distinguish the two.
- **The repair is not "support the em-dash".** The em-dash is merely the first byte sequence at
  which the accidental semantic route stops. Adding it — or widening a semantic character class —
  would leave backticks and `@` broken and would further entrench the absence of the channel.
  DESIGN §4c specifies the intended shape: annotations route through an annotation-specific
  lexical channel and may not produce ordinary semantic tokens or namespace bindings. The generic
  lexer already carries that architecture (`SemanticRule | AnnotationRule`, line-comment
  termination, annotation payload capture, `LineCommentTextChar`); the DAG language model simply
  does not wire it into its production rule set.

One earlier reading recorded during this work — a trailing ASCII hyphen in a comment refusing —
**did not reproduce** on recheck and is discarded as a scratch-file race. It is recorded here only
so that nobody re-derives it from the intermediate notes and treats it as a finding.

### `dag/std/types.dag` is unaffected by the annotation channel

Its `\x00` and `\x0d` sequences sit inside **string literals**, not comments, so the fixed-width
`\xNN` escape hypothesis for this member stands independently of the annotation-channel work. That
hypothesis is corroborated by source content and is **not yet executed** as a direct minimal pair.

## What the same change then repaired

The rows above are the observation taken **before** any repair in this change. Two repairs then
landed against them, and both are recorded here so the table is not read as still describing the
tree after them.

**The `//` annotation channel is now wired.** `dag_lex_rules` registers an `AnnotationRule` for
`//` over the generic lexer's existing `LineCommentTextChar` machinery, placed after the
string-literal rule so string content keeps winning. Executed at `LexArtifact` grain rather than on
`Accepted`, because an `Accepted`-only control cannot distinguish a recognised comment from an
accidentally-lexable one: a plain comment, an em-dash payload, and a backtick-plus-`@` payload each
capture **exactly one** annotation; no `dag_token_slash` reaches the semantic stream in either the
plain or the em-dash case; and the URL inside a string literal captures **zero** annotations.

`dag/std/algebra.dag` was **not** re-confirmed after this repair — the probe again exceeded its
budget, now doing strictly more work than before because it no longer stops at lex. Its post-repair
stage is therefore unmeasured, and the lex wall is claimed closed on the minimal pairs, not on that
member.

**Genuine body-lowering rejections now propagate.** `body_lower_finish_for_normalize` previously
matched every `Rejected` from `body_lower_after_children` and returned wrapper-retention instead,
binding the diagnostics to `_`. The legitimate not-applicable case was already explicit — the final
arm of `body_lower_production_emitted` — so retention had two sources, one produced and one inferred
from failure. The outer catch is deleted; retention now has exactly one producer.
`well_formed` is unchanged.

Measured effect, discriminating rather than blanket: `src/v2/std/logic.dag` moved
`NORM_RETAINED` → `NORM_OTHER` (its real refusal now surfaces as itself), while
`src/v2/std/optional.dag` and `src/v2/std/diagnostic.dag` stayed `NORM_RETAINED`, their retention
being genuine.

**Not repaired here:** the promotion boundary. `NormalizedTree` is still `= Node` and `resolve`
still consumes that alias, so *retained cannot reach resolve* remains true only by propagation,
not by construction.

### `src/v2/std/algebra.dag` remains uncaptured

The historical receipt ran its per-module reason probe over 10 of the 11 rejecting members, and
this is the one it did not cover. It stays visibly on the board here rather than being silently
absorbed into a neighbouring bucket.
