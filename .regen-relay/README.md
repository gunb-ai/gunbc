Transaction input only; this relay branch is not for merge.

Base: cb9529347fe93ec21b73e6697cc25e1c6aba0446 (PR #11277).
Apply payload-correction.patch to that base in the regeneration transaction.
The patch is the gatekeeper-approved variant_owner_application payload correction
and kernel Optional declaration correction (messages msg_5002be21 and msg_57ded911).
It makes no Optional representation migration or comparator changes.

The production branch must receive source plus the adjudicated generated mirrors
atomically. This patch relay commits the WIP without publishing another source-only
production head. It is not a fixed-point receipt or an installed generated candidate.

Validation so far: frontend parses the changed inference source (import-only diagnostics
in the single-file parse probe); generic_variant_pattern_preserves_argument remains
RED on the unmodified generation-one relay seed (223ms CPU), specifically a pattern
binding T and refusing its children field. The authored zero-parameter Optional/none
arity control compiles with no diagnostic on that seed, so its new refusal is discriminating.
Existing none_present_branch_binds covers the literal application call site. A redundant
annotated-literal control was measured and omitted; its annotation already supplies Item,
so it did not discriminate the payload repair. The permanent inferred Optional pattern
control and new generic pattern control retain their unannotated discriminating form.
The regenerated-seed green controls and floor ceiling receipts remain owed.
The Optional representation census is PR comment 5658746422; its implementation is held
pending the separate gatekeeper/side-chat ruling.
