# Q2 classification — type-name-collision wall policy input (2026-06-23)

**Single living Q2 authority** for the #2 resolver-wall (PR #5652) flag-ANY-vs-flag-DIVERGENT
ruling. Upstream census: `docs/plans/dsl-v2-defork-audit.md` (parent's carrier-grounded 2026-06-22
pass). This doc **refreshes that census to post-#5640** and adds the two decision columns parent
needs — *reachability (live vs latent collision)* and *unify status* — measured read-only by
execution. No de-fork executed here (classification only).

Method (read-only, by execution): structural extraction of every top-level `type` decl from
`dsl/std/<b>.dag` vs `src/v2/std/<b>.dag`; shared **unqualified type names** compared with the
guard's own `structural_inequality` predicate (normalized body equality). Reachability = BFS of all
**351** `_test.dag` floor entries' whole-module import closures, testing co-occurrence of `std.<b>`
and `v2.std.<b>` in one closure (= the guard would fire).

## The 9 basenames

| basename | shared type-names | structural verdict | floor co-occurrence | category | unify status |
|---|---|---|---|---|---|
| **algebra** | 16 (13 differ, 3 byte-identical: `Lattice`/`Magma`/`Ordering`) | same concept, divergent **encoding** (dsl flat records vs v2 compositional `{monoid: Monoid<T>}` / coproduct `FreeMonoid`) | **LIVE — 75 entries** | grounding | cant-unify-yet, **path RULED** (operator 2026-06-22: v2 coproduct is authority, dsl record = projection-from-inhabitance; generalize #5428 FreeMonoid seam) |
| **nat** | 1 (`Nat` differs: `= CommutativeSemiring<Magnitude>` vs `= Zero \| Succ`) | same concept, divergent **model** | **LIVE — 4 entries** | grounding | cant-unify-yet, path = #5428 numeric tower; escalated (smart-ant-466), BLOCKED + LAST |
| **effects** | 3 (`EffectShape`/`KeySource`/`CreateCause`, all differ; `EffectShape` body re-modeled on a different axis) | divergent **axis** (dsl operation-kind vs v2 idempotency-class) | latent (0) | grounding | cant-unify-yet, path = pick axis authority (DESIGN §4 idempotency-into-variant); decidable |
| **float** | 3 (2 differ, 1 byte-identical: `Float = Float64`) | same concept, divergent encoding (alias vs `{body: FloatBody}`) | latent (0) | grounding | cant-unify-yet, path = #5428 algebraic-vs-bit layer |
| **integer** | 14 (11 differ only on `MachineWidth<8>` vs `<Word8>`; 3 byte-identical: `IntPlatform`/`UInt`/`UIntPlatform`) | same concept, near-identical (width-token representation diff) | latent (0) | grounding | cant-unify-yet, path = #5428 numeric tower (the diff is one token, mechanically reconcilable once the tower lands) |
| **verification** | **0** (was 1: `TestClaim`) | **RESOLVED by #5640** (dsl record renamed `TestClaim → AssertionClaim`) | none now | **resolved** | **DONE** |
| **logic** | **0** (dsl `Classical`/`classical_*` vs v2 `Bool`+bit-width grounding — different *names*) | divergent concept, **no name overlap** | none (0 shared) | grounding-or-rename (undecided) | **no type-name collision → guard never fires** |
| **coercion** | **0** (dsl cast vocab `CastRule`/`CastSyntax`/… vs v2 `coercion_fold`/`CoercionWitness`) | divergent concept, **no name overlap** | none (0 shared) | v1-exemption (basename rename, v1-coupled) | dissolve-on v1-delete; **0 type-name collisions → vacuous for this guard** |
| **node** | **0** (dsl `compiler_inductive_fields`/… v1-only authority vs v2 126-decl Node substrate) | divergent concept, **no name overlap** | none (0 shared) | v1-exemption (self-dissolves on v1-delete) | dissolve-on v1-delete; **0 type-name collisions → vacuous for this guard** |

## Findings that bear on the flag-ANY + {node,coercion} ruling

1. **The {node,coercion} exemption is VACUOUS for the type-name guard.** node, coercion (and logic,
   and post-#5640 verification) share **zero** type names — they share only the *basename*. The
   guard keys on a shared **unqualified type name** within one closure, so it **never fires** on
   them. Their de-fork is a *module-basename* rename (Route-C / v1-delete), a different surface than
   this guard. So the guard needs **no** {node,coercion} exemption to land green; carrying it would
   be a dead roster entry. (This is exactly the §5 "named/finite scaffold" discipline applied to the
   roster itself: don't list an exemption for a collision the mechanism cannot produce.)

2. **The guard's REAL land-green gate is the grounding collisions with shared type names**:
   `{algebra, nat}` are **LIVE today** — `std.<b>` and `v2.std.<b>` co-occur in **75** (algebra) and
   **4** (nat) floor closures, silently failing-open (benign now only because they shadow
   record-with-record, not the coproduct-variant-drop that broke verification under A1). A flag-ANY
   wall reds those 75+4 entries on landing. `{effects, float, integer}` are **latent** (no
   co-occurring closure today) but are re-arm risks — exactly how verification went from latent to
   LIVE under A1's closure expansion. So the wall **cannot land green until at least the LIVE
   groundings {algebra, nat} are de-forked**, and durably-green needs the latent three too.

3. **No genuinely-stuck THIRD mirror.** Every grounding has a unification *path* (algebra: authority
   ruled; integer/float/nat: #5428 numeric tower; effects: axis decision; verification: done). None
   is permanently un-unifiable. So in the "needs a milestone *later than the de-fork itself*" sense,
   the cant-unify-yet set is **{node, coercion}** (both wait on v1-delete) — the ruling's criterion
   **holds**. The catch is not a stuck mirror; it is finding (1)+(2): the exemption is on the wrong
   surface, and the true gate is the grounding de-fork sequence, fronted by the LIVE pair.

4. **verification is already de-forked (#5640)** — the 2026-06-22 census row is stale; refreshed
   above to 0-shared / resolved.

## Net for parent's ruling

- **flag-ANY is the right policy** (flag-DIVERGENT only excuses the byte-identical mirrors — still
  §3 violations — and fires on every grounding anyway, sparing nothing). Confirmed by the
  distribution: the byte-identical mirrors are a minority (3 algebra + 1 float + 3 integer = 7
  names) and all sit inside basenames that are *already* divergent on other names, so excusing them
  buys nothing.
- **Recommend dropping {node,coercion} from the GUARD exemption roster** (vacuous — 0 type-name
  collisions) and tracking them under the Route-C basename-rename instead. If parent prefers to keep
  a roster for cross-lane bookkeeping, mark it explicitly "non-firing / Route-C basename, not a
  type-name collision."
- **The guard's lands-green precondition = de-fork of the shared-type-name groundings**
  `{algebra, nat}` (LIVE) then `{effects, float, integer}` (latent). That sequence — not the
  {node,coercion} exemption — is what gates the wall. `verification` already cleared via #5640.
- Predicate validation (§5 bonus): the guard's `structural_inequality` ran on all 9 real basename
  pairs and classified correctly — byte-identical → mirror (excused under flag-DIVERGENT), any
  structural diff → divergent (fired). Receipt: this table.
