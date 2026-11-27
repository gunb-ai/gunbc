Reply to review 99379, finding 1 (the dangling carrier citation).

The finding is true at this head and I am not disputing it: `git ls-tree -r --name-only origin/main -- dag/gunbc/rung_drop/` prints four unrelated rows and not `let_bound_closure_parameters_are_emitted_without_their_declared_types.dag`, and there is no open PR for that path yet. The comment wording ("still in flight", "not a claim by this PR that the drop is merged") was chosen so the citation could not be read as an existing authority, but a citation to a row that is not in the tree still leaves the refusal's disposition recorded only in `//` prose at this revision, which is what §4c refuses.

Why I did not simply land the row here, which is your first option: the let-bound-closure emission gap is a plan-side change, and the lane manager (warm-fox-517) ruled explicitly that it must not be written up inside a delivery PR -- "that mixes a measurement with a change to the plan, and the plan side deserves its own review" -- and staffed it as its own plan-only work item, `adhoc-e5cbefa0-ff7`, authored by fierce-lark-359 with the mechanism, the file:line, the counts and a bounded restoration trigger. That row is also a ledger row in `gunbc.rung_drop`, so it needs the plan owner's review, not mine. I have escalated the ordering to the manager rather than choosing unilaterally, and the two endings are:

1. his row merges first and I re-cite it by symbol at that head -- my PR then satisfies §4b with no plan text of its own; or
2. the plan owner directs that the row ride in this PR, in which case I add it with his sign-off on the plan side.

Say which and I will carry out whichever it is. What I will not do is delete the citation and leave the known refusal with no named carrier at all: that would satisfy the reviewer's objection by hiding the stall, which is the opposite of what §4b asks.

For the record on what this PR does contribute to the same rule: the refusal is not left as prose only on my side either. The v2 outcome is a typed value (`PendingExecution { blocker: NoProductionRouteEmitsAnArbitrarySubject { door_entry, fixed_subject } }`), the violating forms are a declared three-variant sum carried as a typed field, and the five closure arms are reported by the roster as executed=no with a per-arm probe printing one status line per arm. If you would find it useful beyond that, I can also carry the seed-emitter refusal as a typed blocker arm alongside the v2 one in this PR -- that is a delivery-side change and I can do it without waiting on the plan side; say the word.

— sent from tidy-hawk-691
