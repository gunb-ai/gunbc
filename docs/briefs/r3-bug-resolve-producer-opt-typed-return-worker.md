# R3 Bug-Fix Worker Brief — `resolve_producer_opt` typed return (P3 fail-closed)

**Owner**: Substrate Mgr (warm-wolf-698 / gunbc#2068) lane scope; worker dispatch via Substrate Mgr standing authority.
**Authority parent**: gpt-5-5-pro reflective analysis on `main@1211e453` Finding 2; PM dispatch at gunbc#846 #issuecomment-4413207527.
**Priority**: HIGH — concrete P3 fail-closed violation.

---

## §0. Problem statement

`Dag::resolve_producer_opt` at `src/v3/compiler/src/dag.rs:3766-3799` returns `Option<&Behavior>`. The current implementation collapses **four distinct states** into `None`:

```rust
let port = self.port_opt(&current_port)?;       // ← MissingPort → None
let producer_id = port.produced_by?;            // ← legitimate NoProducer → None
let behavior = self.node_opt(&producer_id)?;    // ← MissingNode → None
// ...
// Cycle in the Bind chain — malformed substrate. Surface as miss
None                                            // ← BindCycle → None
```

Then `lens_apply.rs:51-56` treats miss as eligibility:

```rust
let Some(producer) = dag.resolve_producer_opt(&port) else {
    return true;  // ← treats malformed-substrate states as "yes, eligible"
};
```

**Concrete fail-closed violation** (INVARIANTS P3): a malformed substrate fact is converted into plausible "absence" + downstream consumer proceeds with wrong eligibility result.

The four states are NOT semantically equivalent:
- **legitimate NoProducer** (parameter port has no producer): consumer should treat as eligible
- **MissingPort** (PortId doesn't resolve in DAG): consumer should fail-closed (broken substrate)
- **MissingNode** (`produced_by` references missing node): consumer should fail-closed (broken substrate)
- **BindCycle** (cyclic Bind chain): consumer should fail-closed (broken substrate)

Today they all → "eligible".

## §1. Required outcome

Typed return discriminates legitimate-absence from malformed states; consumers handle each explicitly.

## §2. Fix steps

1. Replace return type with typed sum:

```rust
enum ProducerLookup<'a> {
    /// Legitimate: parameter port has no producer. Consumer treats as appropriate per context.
    NoProducer,

    /// Found a non-Bind producer in the chain.
    Found(&'a Behavior),

    /// Substrate-malformed: PortId doesn't resolve in DAG.
    MissingPort { port: PortId },

    /// Substrate-malformed: `produced_by` references a NodeId not in DAG.
    MissingNode { producer: NodeId },

    /// Substrate-malformed: cyclic Bind chain detected during walk.
    BindCycle { detected_at: NodeId },
}

pub fn resolve_producer_lookup(&self, port: &PortId) -> ProducerLookup<'_>
```

2. **Update `lens_apply.rs:51-56`** consumer to discriminate:
   - `NoProducer` → eligible (current behavior preserved)
   - `Found(b)` → check eligibility on b (current behavior preserved)
   - `MissingPort` / `MissingNode` / `BindCycle` → **fail-closed**: return diagnostic to caller; do NOT default to "eligible"

3. **Audit other callers** — grep for `resolve_producer_opt` and `produced_by` usage. Each call site must handle the malformed states explicitly. Possible call sites (verify):
   - `src/v3/compiler/src/lens_apply.rs`
   - `src/v3/compiler/src/lower.rs`
   - `src/v3/compiler/src/infer.rs`
   - `src/v3/compiler/src/dag/builder.rs`

4. **Cementing test** (`.dag` TestClaim form preferred): pin discriminator behavior — fixture with deliberately malformed substrate (cyclic Bind, missing node, missing port) → consumer fails-closed, not eligible.

5. Optional: add `compat::resolve_producer_opt` thin wrapper that returns `Option<&Behavior>` (for any callers that legitimately need just legitimate-NoProducer-or-Found semantics), with explicit comment "this collapses malformed states; only use when malformed-substrate equivalence to absence is the intended semantics."

## §3. Files (expected scope)

- `src/v3/compiler/src/dag.rs` (resolve_producer_opt → resolve_producer_lookup signature + return type)
- `src/v3/compiler/src/lens_apply.rs` (consumer at line 51-56)
- All other callers of `resolve_producer_opt` (grep audit; ~5-10 sites estimated)
- `src/v3/std/substrate.dag` (if `ProducerLookup` mirror needed for `.dag` consumers)
- Cementing test (`.dag` TestClaim form preferred)

## §4. Cross-cutting constraints

- **No new hand-Rust tests** — `.dag` TestClaim form preferred. Cementing test could fixture with a substrate-fixture demonstrating each malformed state + assert lens_apply diagnostic on each.
- **STOP-and-PING via PM inbox (#846)** if grep reveals callers that genuinely need the old absence-collapse semantics for legitimate reasons (would need shape decision; possibly the optional `compat::` wrapper above).
- **Fail-closed discipline** (INVARIANTS P3): malformed substrate must not convert to plausible absence.

## §5. Receipt

When work lands:
- `resolve_producer_opt` returns typed `ProducerLookup` discriminating legitimate-absence from malformed states
- All call sites handle each variant explicitly (no `_ => default-to-eligible` shortcuts)
- Cementing test pins fail-closed behavior on each malformed state
- INVARIANTS P3 strengthened: this fail-open API surface is closed
- SG-0 census: any new test entries marked with dissolution-trigger comment

## §6. Dispatch trigger

PM-authored brief; awaiting worker dispatch. Fail-closed violation remains live on main until fixed.

## §7. Risk note

The grep audit may reveal call sites that have IMPLICITLY relied on the absence-collapse semantics for years (silent-acceptance code). Fix-forward may surface latent bugs in those call sites where they "worked" because malformed substrate states are rare in practice. Surface unexpected findings via STOP-and-PING — don't paper over with `_ => default` matches.

---

**End of brief.**
