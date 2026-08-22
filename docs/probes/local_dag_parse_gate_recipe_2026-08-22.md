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

## Why a trivial entry suffices

The entry selects what gets **compiled**; it does not bound what gets **indexed**. Building the module
index reads every `.dag` source under every declared root before any entry closure is resolved, and it
refuses on any unparseable one. So the parse phase is corpus-wide *by construction* and a two-line
entry pays only the index, not a real compile — which is what makes it a minute rather than the eight
that a floor run costs.

That also fixes what the recipe does and does not tell you. It answers exactly one question — does
every `.dag` source in the tree parse — which is `required-ci`'s first phase. It is **not** compile-clean,
not regen, not the witness floor, and a green run here says nothing about any of them.

## Reading the coordinates

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
