### Heuristics indicate lost structure

Heuristics are a code smell in compiler and runtime logic. String
matching, score-based classification, best-effort guessing, "close
enough" defaults, and inference from naming conventions usually mean
the pipeline has already thrown away information that should have been
structural.

**The principle:** do not tune the heuristic first. Trace the pipeline
upstream until you find where the needed fact stopped being explicit,
then restore that structure as close to the source as practical.

**The test:** if a code path has to guess from strings, partial shapes,
error text, or naming patterns, the real bug is upstream information
loss. The preferred fix is to carry the missing fact in the type/IR/API
boundary instead of improving the guess.

**The fix:** push structure earlier in the pipeline so the downstream
stage can make an exact decision.

**Structural prevention:** Graph rendering. The emitter walks the
typed graph and invokes the language renderer for each structural
pattern. The emitter never produces strings — it matches patterns
(product, coproduct, sequence, etc.) and the renderer converts them
to target text using `LanguageSpec`. The emitter cannot produce
`"Rc<Vec<...>>"` because it doesn't produce strings at all. The
escape hatch is string concatenation in the emitter; the fix is an
emit stage that walks the graph and delegates to the renderer. This
is the highest-leverage single change — it structurally prevents
~60% of all recurring violations (hardcoded target syntax, Rc
wrapping, container patterns, type name dispatch, method rendering).

