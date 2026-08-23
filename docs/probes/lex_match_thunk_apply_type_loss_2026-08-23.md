# LexMatchThunk `apply` receiver-type loss, 2026-08-23

## Question and bound

Why do the seven `Rc<LexMatchThunk>.apply` E0599 diagnostics in
`v2.compiler.tokenize` occur before rustc can expose the later tokenize region?
This receipt locates that one observation barrier. It does not claim a general
generic-instantiation repair.

The comparison instrument was `docs/probes/curated_cargo_probe_one.sh` with entry
`src/v2/compiler/01_tokenize.dag`. Every comparison rebuilt the seed binary from the
worktree before emission. **CLOSED_AT_REF:** `849d3aec6a59f5288781f1d576dbe68432cd3958`.
The unchanged control summary at that ref was:

```text
26 files emitted, 31 diagnostics, cargo refuse
E0599:8 E0308:4 E0277:4 E0560:1 E0425:1
```

Seven of the eight E0599 diagnostics are `no method named apply found for
Rc<LexMatchThunk>`; the eighth is the independent `partial_cmp` site in
`v2_std_node.rs`.

## Two refuted producers

Both arms below rebuilt the compiler from the changed worktree and reproduced the
summary above byte/count-identically, including all seven `apply` sites.

1. **Downstream reinference alone.** After first inferring the generic record-literal
   actual, direct-call inference derived its substitution and re-inferred the literal
   against the substituted formal. No emitted byte moved. Therefore the missing fact
   is not repaired merely by adding a second inference after the existing record
   generic unifier.
2. **Inferred type-variable id key plus reinference.** `unify_generics` recognizes an
   `InferredNode::TypeVariable` as bindable but keys it through the node's authored
   label. A trial keyed that arm by its type-variable id and retained the reinference
   above. No emitted byte moved. This is a real inconsistency worth a separate
   discriminating witness, but that representation is not the producer on this route;
   shipping it in the unmask change would be unrelated machinery.

Both trials were reverted.

Once the temporal root was observed, the two nulls became confirming evidence: both
attempts operated after the ordering had already denied the field lambdas their
expected types. A downstream key correction cannot help a binding consulted too late,
and a post-descent retry does not establish the evidence before the original descent.

A final bounded ordering trial pre-scanned only the record actual's non-lambda fields,
intending to bind `R` from `empty: LexMatchThunk` before the callable fields were
inferred. Its required discriminator was the lambda expectation itself, not the rustc
board. A worktree-built compiler still reported `expected_name=` and
`expected_params=0` for every outer algebra lambda. Thus the formal is less available
at that pre-descent seam than the later `subst=[R]` trace suggests: declaration-field
lookup there does not expose `empty: R` as bindable evidence. The ordering trial and
its instrumentation were reverted rather than widened into another compiler-wide
generic-inference theory.

## Root classification correction

The observation map initially suggested an **ordering defect**: the substitution
appeared to exist and merely be consulted after its consumers. The expectation-first
third arm refutes that reading. At the pre-descent seam, declaration-field lookup does
not represent `empty: R` as binding evidence at all; there is therefore no available
binding to consult earlier. The located root is a **representation gap, not an ordering
gap**. Its next owner is the declaration-field lookup/direct-call-fold boundary that
must make that evidence expressible, rather than another rearrangement of inference.

All three rebuilt-binary arms triangulate that boundary. Each assumed `R` was
recoverable at a different position—after descent, through the inferred-type-variable
key, or before descent—and each left the relevant observation byte/count-identical.
The `unify_generics` keying inconsistency in arm two remains a real, separate defect:
it needs its own discriminating red and must not be mistaken for this producer.

The discriminating rule is transferable: **measure the outer lambda expectation before
measuring the rustc mask**. An unchanged mask is ambiguous—it could mean a correct
upstream repair exposed a second blocker. An unchanged `expected_name=` /
`expected_params=0` directly proves the attempted producer did not establish the fact
its consumer requires, which closed the third arm without a fourth speculative repair.

## Located pipeline map

One observation-only compiler build printed the direct-call substitution, every
tokenize lambda's expected callable shape, and each `apply` receiver at the method
wall. The relevant sequence was:

```text
call arg=0 formal=LexPattern generics=[R] subst=[]
lambda params=[left_r,right_r] expected_name= expected_params=0
method recv= resolved= shape=Primitive()
...
call arg=1 formal=LexPatternFold generics=[R] subst=[R]
```

The same empty expected shape appeared on `choice`, `optional`, `one_or_more`, and
`delimited`; their seven `apply` receivers consequently arrived at the method wall as
`Primitive()` with no authored name. Controls in the same module showed ordinary
declared parameters arriving as `LexMatchThunk` / `Product(LexMatchThunk)`, and the
nested `apply: fn(s)` field lambdas receiving `expected_name=fn expected_params=1`.

The type therefore dies at a precise seam: **the generic substitution `R =
LexMatchThunk` becomes available only after the whole `LexPatternFold<R>` actual has
already been inferred.** Its function-valued fields are inferred during that first
pass, so the outer field lambdas (`sequence`, `choice`, and siblings) receive the
unsubstituted `R` formal as no established expected type. The later substitution does
not flow backward into their parameter bindings. The method wall and Rust emitter are
downstream observers, not producers: when a receiver nominal type exists, the same run
shows the wall resolving it and the emitter's existing callable-field path can lower
`(receiver.apply)(arg)`.

Coverage remains **PARTIAL**: mask `apply` remains 7, and the exact current hidden
population remains unknown. The historical 68-site tokenize roster is a different
unit at a different ref and is not added to this board.
