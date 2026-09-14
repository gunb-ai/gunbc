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
RED on the unmodified generation-one relay seed; optional_literal_pattern_preserves_argument
passes semantically but its first-run CPU is 637ms, above the 500ms floor budget.
The arity control and regenerated-seed green controls remain owed.
The Optional representation census is PR comment 5658746422; its implementation is held
pending the separate gatekeeper/side-chat ruling.
