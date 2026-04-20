### Explicit boundary contracts

Each stage of the pipeline (parse → typecheck → lower → resolve →
execute) passes a complex IR type to the next stage. The receiving
stage's preconditions must be structural — encoded in the type of the
boundary, not checked by a validation pass after the fact.

**The principle:** make illegal states unrepresentable. When a
downstream stage needs a guarantee (e.g., "all type references are
resolved"), the upstream stage must produce an output type that
*cannot* represent the unresolved case. The compiler enforces the
contract; no runtime validation walk is needed.

**The test:** if you find yourself wanting to add a validation pass
at a boundary, instead refactor the upstream stage's output type so
the invalid state is impossible to construct.

Examples (current state and target):
- After lowering (done): transport nodes are a distinct `LoweredOp::Transport`
  variant with required `ServiceCallMetadata` and `TransportObligation`.
  Transport obligations are structurally excluded from `LoweredOp::Callable`.
- After lowering (target): ports embed `ResolvedType` instead of `TypeId(String)`.
  `ResolvedType` is defined in `gunbc-ir` but not yet wired into ports;
  the migration is additive (`resolved_type` alongside `type_id`).
- After typecheck (target): the output type embeds resolved type structure,
  not a string TypeId that might not resolve.
- After resolve: the output DAG is parameterized by a trait that
  requires `Executable`, so non-executable nodes are unrepresentable.

When a boundary today uses a type that *can* represent invalid states,
that is the root cause — not the absence of a validation function.
Every fabrication fallback in FC-7 existed because the producing
stage's output type was too permissive, and the consuming stage
compensated with a fallback instead of failing.

A boundary fact table is only valid when both of these hold:

1. Every entry is an exact derivation from upstream structure. If the
   table collapses distinct bindings, guesses a classification, or drops
   witnesses needed downstream, it is a lossy representation and is
   already an invariant violation.
2. A downstream stage actually consumes the table as the authority for a
   decision. If no consumer reads it, the table is speculative metadata
   or a parallel representation waiting to diverge.

Unused or lossy fact tables are not harmless scaffolding. Unused tables
violate "No parallel implementations" / "Single-authority metadata."
Lossy tables violate "Explicit boundary contracts" / "Heuristics
indicate lost structure." The default action is to delete the table
until a concrete consumer exists, or tighten it until the missing
distinctions are structurally preserved.

New semantic boundaries must land end-to-end. A new normalize/pass/fact
layer is not accepted just because it computes plausible metadata; at
least one downstream consumer in the same change must read it as the
authority for a real compilation decision. Otherwise the layer is still
speculative metadata and should stay out of the pipeline until the
consumer exists.

