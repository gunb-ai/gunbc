# One obligation, one authority, one refusal per position

## Why not seven checks

Seven grammar positions can accept a value that does not inhabit the type declared at them.
The tempting repair is seven local predicates at seven seams. That is seven representations of
one rule (DESIGN sections 2 and 3), and they drift: the corpus already carries the receipt —
`kernel_value_declared_type_mismatch` bails unless the ACTUAL is a kernel type,
`structured_application_site_type_mismatch` bails unless the actual EXPR is an `ExprRecordLit`,
and `declared_type_conformance_diags` judges only when BOTH sides are ground kernel types. Three
predicates, three different scopes, one question, and an actual that is a CALL falls between all
three.

## The shape

Each value-bearing grammar position produces one obligation; one relation decides it; the
position supplies only its own name and span for the diagnostic.

    type DeclaredTypePosition =
      | RecordLiteralField | DataInitializer | DeclaredReturn | DirectCallArgument
      | ParameterDefault | LetAnnotation | ListElement | MapValue
      | GenericTypeArgument | VariantPayload | CallableReturn | CastTarget

    type DeclaredTypeObligation {
      position: DeclaredTypePosition
      declared: Node
      produced: Node
      span: SourceSpan
      module_name: String
    }

    type InhabitanceVerdict =
      | Inhabits
      | RefusedPayloadAtParent { arm_name: String }
      | RefusedKernelAtStructured
      | Undecidable { reason: UndecidableReason }

    type UndecidableReason =
      | GenericFormal | OptionalCarrier | FormalUnresolved | ProducedIdentityErased

`declared_type_inhabitance(obligation, scope) -> InhabitanceVerdict` is the single authority. It
CONSUMES the transparent-alias identity relation (gunbc#8873's `transparent_alias_identity_agrees`
over the `SymbolIndex`) and `application_type_names_compatible` rather than reimplementing name
identity — the fact it needs does not survive `resolve_item_types`, which is that lane's finding
and not a wheel to re-invent.

Five distinctions it must draw, because each has a live specimen in the corpus:

1. exact coproduct member at a coproduct position -> `Inhabits`
2. transparent alias of the declared type -> `Inhabits` (`Hash` for `Fnv1a64Structural`)
3. a value of one arm's PAYLOAD type at the PARENT position -> `RefusedPayloadAtParent`
4. a plain kernel value at a declared structured type -> `RefusedKernelAtStructured`
5. a generic coproduct, the `Optional` cardinality carrier, an unresolved formal, or a produced
   side whose type identity was erased upstream -> `Undecidable`, typed and COUNTED

## `Undecidable` is not the rejected counted advisory

Review 45647 rejected a counted advisory over `DeclaredTypeConformanceUnjudged` — correctly: it
covered programs that were PROVABLY wrong, and a number beside a wrong program is fabricated
success one level up. `Undecidable` here covers only pairs the relation cannot decide from the
declaration (a type variable in a payload position is not decidably a member or a non-member).
Refusing there would be a fabricated refusal. Counting there keeps the deficit's frequency
observable rather than zero by construction, which is the same rule read the other way.

## Evidence per position, and the one people skip

Per reachable value-bearing position, five arms:

- positive: the correctly wrapped member is ACCEPTED
- negative A: a plain kernel value REFUSES
- negative B: an arm payload at the parent position REFUSES
- reachability: an undefined name at the same position REFUSES
- **discriminator**: with THAT position's obligation producer disabled, exactly that position's
  controls go red and no other position's do

The discriminator is what proves a control is wired to the position it names. Without it, a
control can pass because some neighbouring judgment refuses the same program for another reason.

## Sequencing

The 14-position enumeration is the coverage authority: a new grammar type position cannot be
added without an explicit disposition row. Positions land one at a time, each with its own
measured corpus arm, because turning a wall on IS the census (gunbc#8876 turned one on and found
eight live defects).

The direct-call argument position is LAST and is blocked: deleting
`module_skips_direct_call_arg_check` is necessary for `v2.*` direct-call checking and INSUFFICIENT
for the other positions, and a correction to that effect is open. Closing it first would report
the class closed while six positions stay silent — a real fix producing a false completion.

The cast position stays separately classified throughout: `expr as T` is an authored assertion
with its own semantics, not an implicit inhabitance claim.
