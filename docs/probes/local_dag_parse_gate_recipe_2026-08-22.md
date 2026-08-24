# Reproducing CI's corpus-wide `.dag` parse gate locally, in about a minute

**Nothing local parses `.dag` before you push.** `.githooks/pre-push` runs `cargo fmt --all --check`
and nothing else. So a prose-only edit — a comment, an annotation, a `String` note on a carrier — has
no local check between it and CI, and it is not a low-risk edit class: the case that produced this
note was a comment edit that took CI down.

## The recipe

Drop a two-line entry module anywhere under `dag/`, compile it with both source roots, then delete it:

```
cat > dag/test/parsecheck.dag <<'EOF'
module test.parsecheck
import std.types { Bool }
fn parsecheck_holds() -> Bool {
  true
}
EOF
gunbc compile --output-dir /tmp/pc_out --source-root dag --source-root src/v2 --entry dag/test/parsecheck.dag
rm -f dag/test/parsecheck.dag
```

Exit 0 with `indexed NNNN modules from 2 source roots` means the tree parses. A parse defect *anywhere*
in the corpus refuses with the file and the byte span, whatever your entry was:

```
module index refused: 1 unparseable .dag source(s)
  dag/gunbc/ci_layer_roots.dag:22395-22396: expected expression, found Slash
```

## The indexing line is NOT the verdict — wait for the exit code

**This is the way to get a false green, and a false green here is worse than no gate**, because you
will have "run the check". Added 2026-08-23 after `neat-heron-312` came within a minute of reporting
the recipe as broken, and after this document's own earlier claim about *when* the check fires turned
out to be wrong.

The run prints `indexed 3884 modules from 2 source roots` and **succeeds** at that step. The
annotation-grain and parse diagnostics do not land there: they arrive after reconcile, at the very
end — measured at **128 seconds** on one box. Someone who starts the command, sees a clean indexing
line at 95 seconds, and interrupts will read a defect-free tree that is not defect-free.

So the verdict is `EXIT=0` plus `0 diagnostics` on the final line. Nothing earlier counts.

```
... ; echo "EXIT=$?"
```

**The mechanism claim in the next section is corrected by this.** An earlier revision of this
document (and a message this session sent to two other lanes) said annotation grain is checked
*during indexing*. It is not — indexing reads the sources, and the refusal is raised later. What
survives unchanged is the part that makes a trivial entry sufficient: the diagnostic is raised for
modules the entry never imports and never compiles.

## The gate has been run in both directions

A check that has never gone red is a decoration. `neat-heron-312` ran both arms on one tree with one
binary (2026-08-23):

| arm | result |
|---|---|
| in-body `//` planted in `dag/product/workload_simulation.dag` | `EXIT=1`, `1 hard diagnostic`, located to that file |
| reverted, same command, same binary | `EXIT=0`, `compiled: 6 files emitted, 0 diagnostics`, 128s |

The planted defect was in a module the trivial entry does not import. **Six files compiled, 3884
indexed, and the diagnostic came from one of the 3878 that were never compiled at all** — which is
the load-bearing evidence for the section below, and stronger than the reasoning that motivated it.

## A stale binary fails loudly and looks like a catastrophic finding

If your binary predates the `--entry` flag it refuses the flag outright. Falling back to a
whole-corpus compile *without* an entry selects a parser that rejects `//` entirely and returns
**~25964 errors on a clean tree** (`warm-tern-755`, 2026-08-23, confirmed against a stashed HEAD as
baseline noise).

That is the absorbing fallback with the failure wearing the costume of an answer: the degraded mode
is loud, and its output reads as a discovery rather than as a broken instrument. **Five-figure error
counts mean check your binary, not check the corpus.**

## Why a trivial entry suffices

The entry selects what gets **compiled**; it does not bound what gets **read**. Every `.dag` source
under every declared root is read and checked regardless of the entry closure, so the parse phase is
corpus-wide *by construction* and a two-line entry pays only that sweep, not a real compile — which is what makes it a minute rather than the eight
that a floor run costs.

That also fixes what the recipe does and does not tell you. It answers exactly one question — does
every `.dag` source in the tree parse — which is `required-ci`'s first phase. It is **not** compile-clean,
not regen, not the witness floor, and a green run here says nothing about any of them.

## Reading the coordinates

The local gate locates by **character offset**; the floor's message for the same class gives
`file:line:col`. So this tells you the file and makes you hunt within it — not a defect, but do not
assume your grep is broken when the number does not match a line.

`file:START-END` is a **byte** span, not a line range. `sed -n '1,NNNp'` on those numbers will point at
the wrong place. To see the offending text:

```
python3 -c "print(repr(open('<file>','rb').read()[<START>-120:<END>+120]))"
```

## The edit class that produced this

Pasting an executed diagnostic verbatim into a `.dag` string. The refusal that was being recorded read
`PoolRootContributesNothing { caller: "data_decl_type_facts", ... }`, and its inner `"` closed the
carrier's string early; the path that followed then parsed as an expression and hit `Slash`. Copying a
diagnostic exactly is the right instinct for evidence and the wrong one inside a quoted carrier —
restate the facts in prose (caller, counts, path, defect kind) rather than nesting quotes.

## The durable version, deliberately not built here

A `.dag` parse step belongs in the pre-push hook, which has a real modeled home in
`gunbc.githooks_pre_push_emit` rather than being hand-edited. That is throughput work and is deferred
by operator ruling (2026-08-22) behind a merge-constrained queue; this note is the interim recipe, not
a substitute for it. **This note deletes when that step lands** — at which point the check runs on every
push and nobody needs to remember a recipe.
