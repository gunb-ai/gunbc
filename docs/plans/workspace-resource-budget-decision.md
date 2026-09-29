# Decision needed: internally budgeted workspace admission

This is a proposal, not an implemented admission exception or an authorization
receipt. No capacity has been acquired under it.

## Observed blocker

`product.fabric.selection.DispatchInputs` requires availability, commitment,
start, transition, marginal operating and opportunity-cost observations. Its
unobserved arms refuse; owned capacity is deliberately not presumed free.
`workspace_prepare_convergence.WorkspacePreparationEvidence` currently requires
`WorkspaceOfferTerms`, which carries those dispatch observations. There is no
production producer of that commercial evidence for the dedicated workspace slot.

An operator-declared zero user tariff would be a billing policy. It would not
establish zero physical operating cost or zero opportunity cost. Populating the
latter with zeros would contradict the existing selector's contract.

## Proposed decision

For the first internal workspace, admit an explicit resource-budget policy in the
shared fabric path. This admits use of already committed dedicated capacity; it
does not claim a price-optimal placement or observed zero operating costs.

The policy must be explicit and request-bound, and carry the operator's authority
and policy revision. It must not be inferred merely because a supplier is local,
a price is absent, a priced offer failed, or only one candidate exists. Unknown
cost observations remain unknown in the result. Any zero user tariff is recorded
as an authored policy rather than as a measurement.

Retain the existing Work requirements, provider/use authorization, host eligibility,
resource-grant sizing, exclusive hold/CAS, reservation binding, installed lifecycle,
cleanup, lease and generation-fenced settlement. Do not create another allocator,
reservation store, VM controller or HTTP execution path.

Initial scope remains one authorized operator, one outstanding allocation including
cleanup, the existing two guest profiles, one accounted and exclusively withdrawn
28 GiB slot with controller cost charged separately, the pinned workspace image,
8 GiB disposable disk and a 60-minute lease. Requests still name no host. Only
actually commissioned eligible fleet capacity may satisfy the request; no paid
external purchases or unaccounted memory are admitted.

## Required controls before operational use

- Missing or mismatched resource-policy authority refuses; a pricing refusal
  cannot silently become resource-budget admission.
- Price-ranked selection continues to refuse missing cost evidence.
- Resource-budget selection retains unknown cost standing and makes no economic
  ranking claim, while enforcing all capacity, ownership and provider-use gates.
- Absent commissioning, uncertain hold/cleanup, competing purpose, stale evidence,
  foreign owner and mismatched generation still refuse.
- The same durable acquisition, retirement and same-slot reuse controls apply;
  no separate lifecycle is introduced.

The alternative is to keep the current price-ranked selector mandatory and first
commission attributable cost observations for this slot. Neither alternative is
established by the current focused tests or by server readiness.
