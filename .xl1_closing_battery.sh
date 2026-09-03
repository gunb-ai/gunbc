# XL-1 closing battery. Untracked; never git add. Each part is one remote dispatch under the 45-min runner cap:
#   for P in af hk ln prod p5 live; do ctrl-build --remote -- bash -lc "$(cat .xl1_closing_battery.sh)" x $P > battery_$P.out 2>&1 & done
# Runner rule (XL-N, measured by A1-R): parts that only run claim_batch fit the DEFAULT VM (~7.1 GiB) with budget 7000000000;
#   any part running gunbc compile / gunbc run over the whole tree (live; the gunbc-compile control) needs the 20GB runner:
#   CTRL_BUILD_RUNNER_EXEC_PROPERTIES=$'EstimatedMemory=20GB\nEstimatedCPU=8' CTRL_BUILD_FORWARD_ENV=GUNBC_MEMORY_BUDGET_BYTES GUNBC_MEMORY_BUDGET_BYTES=16106127360 ctrl-build --remote -- ... x live
#   and verify the 'forwarding env:' line names GUNBC_MEMORY_BUDGET_BYTES. Inside the script the export honours an already-set value.
# Bind afterwards: source_tree = git rev-parse HEAD^{tree} of the tree the battery ran on (squash sha ok iff tree identical).
set -e
PART=$1
export GUNBC_MEMORY_BUDGET_BYTES=${GUNBC_MEMORY_BUDGET_BYTES:-${XL1_MEMORY_BUDGET_BYTES:-7000000000}}  # #9726 governor needs a bound; ctrl-build does not forward GUNBC_*. BuildBuddy VM is ~7.86 GB so the budget must sit UNDER it (7 GB) or the VM kills the run before the governor engages; on srv1 pass XL1_MEMORY_BUDGET_BYTES=10737418240 (main_wet peak 7.7 GiB)
cargo build --release -p v1-compiler --bin claim_batch --bin gunbc 2>&1 | tail -1
echo "PRODUCER=claim_batch (sole producer; gunbc compile exceeds the 7.5 GB runner VM on this main). COMPILE CONTROL = resolve-refusal lines (^error / ^claim_batch: .*refus) in the same claim_batch run, reported per arm as resolve_refusals=N"
G=./target/release/gunbc; CB=./target/release/claim_batch
M1=src/v2/compiler/effect_demand.dag; T1=src/v2/test/claim/effect_demand/effect_demand_census_test.dag
M2=src/v2/compiler/effect_demand_floor_join.dag; T2=src/v2/test/claim/effect_demand/effect_demand_floor_join_test.dag
comp(){ echo "n/a"; }
refusals(){ $CB --source-root dag --source-root src/v2 --entry $1 --functions $(grep -oE "^test fn [a-z0-9_]+" $1 | sed "s/^test fn //" | head -1 | paste -sd,) 2>&1 | grep -cE "^error|^claim_batch: .*refus" || true; }
runall(){ local T=$1; local A=$(grep -oE '^test fn [a-z0-9_]+' $T | sed 's/^test fn //' | paste -sd,); $CB --source-root dag --source-root src/v2 --entry $T --functions $A 2>&1 | grep -E '^(PASS|FAIL) |^claim_batch:|^error' | sed -E 's/(effect_demand|floor_join)_witness_//'; }
mut(){ python3 - "$1" "$2" "$3" <<'PY'
import sys
p,old,new=sys.argv[1],sys.argv[2],sys.argv[3]
s=open(p).read()
assert s.count(old)>=1, "MUTATION TARGET MISSING: "+old[:80]
open(p,"w").write(s.replace(old,new,1)); print("   applied")
PY
}
arm(){ echo "=== MUT $1: $2 (resolve_refusals=$(refusals $4))"; runall $4 | grep -E "^FAIL|^claim_batch: [0-9]+ (FAIL|fail)|^error" || echo "(no FAIL lines)"; cp $3.bak $3; }
cp $M1 $M1.bak; cp $M2 $M2.bak
echo "TREE=$(git rev-parse HEAD^{tree}) HEAD=$(git rev-parse HEAD)"
case $PART in
af)
echo "##### DEMAND+SEAM BASELINE + ARMS A-F over $T1 ($(grep -c '^test fn' $T1) witnesses)"
echo "=== BASELINE (resolve_refusals=$(refusals $T1))"; runall $T1
mut $M1 'if row.primitive.slug == primitive.slug { Cons' 'if true { Cons'; arm A "realization slug comparison always true" $M1 $T1
mut $M1 '    SeamUnrostered => SeamUnrealized,' '    SeamUnrostered => SeamRealizedByBridge { bridge: "smuggled" },'; arm B "unrostered resolves a realization" $M1 $T1
mut $M1 'evidence: SeamEvidenceDerivedNotExercised { reason: ^declaration_not_emitted_in_the_measured_closure }' 'evidence: SeamRefusalObservedInClosure { closure_entry: "v2.compiler.compile" }'; arm C "a realized seam carries refusal evidence" $M1 $T1
mut $M1 'bridge: "rc_empty_map"' 'bridge: "not_the_bridge"'; arm D "registry row rebound to a wrong bridge" $M1 $T1
mut $M1 '  SeamRuntimeBinding { primitive: primitive_identity_slug(name: "symbol_lexeme"), bridge: "symbol_lexeme" },' '  SeamRuntimeBinding { primitive: primitive_identity_slug(name: "empty_map"), bridge: "a_second_claimant" },
  SeamRuntimeBinding { primitive: primitive_identity_slug(name: "symbol_lexeme"), bridge: "symbol_lexeme" },'; arm E "one primitive claimed by two bridges" $M1 $T1
mut $M1 '      Cons { head: second, tail: rest } => SeamRuntimeBindingAmbiguous { primitive: primitive, bridges: matches }' '      Cons { head: second, tail: rest } => SeamRealizedByBridge { bridge: head }'; arm F "ambiguity collapses to last-match-wins" $M1 $T1
echo "=== RESTORE CONTROL M1"; cmp -s $M1 $M1.bak && echo "bytes identical to pre-mutation"
;;
hk)
echo "##### DEMAND ARMS H-K over $T1"
mut $M1 '  (demand.operation.slug == realization.operation.slug)
    && execution_mode_eq(left: demand.mode, right: realization.mode)' '  (demand.operation.slug == realization.operation.slug)'; arm H "realization join ignores execution mode" $M1 $T1
mut $M1 '  effect_demand_strings_contained(left: left, right: right)
    && effect_demand_strings_contained(left: right, right: left)
    && (length(xs: left) == length(xs: right))' '  (length(xs: left) == length(xs: right))'; arm I "membership agreement falls back to count equality" $M1 $T1
mut $M1 '  if !effect_demand_receipt_roots_resolved(receipt: left) {' '  if false {'; mut $M1 '  else if !effect_demand_receipt_roots_resolved(receipt: right) {' '  else if false {'; arm J "gate stops requiring resolved roots" $M1 $T1
mut $M1 '  else if !effect_demand_producers_are_distinct(left: left, right: right) {' '  else if false {'; arm K "gate stops requiring distinct producers" $M1 $T1
echo "=== RESTORE CONTROL M1"; cmp -s $M1 $M1.bak && echo "bytes identical to pre-mutation"
;;
ln)
echo "##### DEMAND ARMS L-N + PASS COUNT over $T1"
mut $M1 '  else if !effect_demand_population_subject_is_complete(population: population) {' '  else if false {'; arm L "gate stops requiring a complete subject" $M1 $T1
mut $M1 '  else if length(xs: left.unresolved_indirect_edge_identities) != 0 {' '  else if false {'; mut $M1 '  else if length(xs: right.unresolved_indirect_edge_identities) != 0 {' '  else if false {'; arm M "unresolved indirect edges ignored" $M1 $T1
mut $M1 '  else if floor_discovery_tree_identity(tree: left.source_tree) != floor_discovery_tree_identity(tree: right.source_tree) {' '  else if false {'; arm N "tree mismatch ignored (join key blinded)" $M1 $T1
echo "=== RESTORE CONTROL M1"; cmp -s $M1 $M1.bak && echo "bytes identical to pre-mutation"; echo "PASS count: $(runall $T1 | grep -cE '^PASS')"
;;
prod)
echo "##### PRODUCER RECEIPT over $T2 ($(grep -c '^test fn' $T2) witnesses)"
echo "=== BASELINE (resolve_refusals=$(refusals $T2))"; runall $T2
mut $M2 '    predicate: fn(path) { contains(xs: seam_paths, item: strip_leading_dot_slash(s: path), eq: floor_join_string_eq) }' '    predicate: fn(path) { true }'; arm 1 "seam reach always true" $M2 $T2
mut $M2 '    Absent => floor_join_refuse(state: state, cause: EntryPathUndeclared { entry_path: row.entry, function: row.function }),' '    Absent => state,'; arm 2 "undeclared entry skipped instead of refused" $M2 $T2
mut $M2 '                disposition: required_floor_site_disposition(module_path: owning_module, identity: identity)' '                disposition: Planned'; arm 3 "standing forced Planned" $M2 $T2
mut $M2 '  serialize_content_hash(hash: content_hash_of_value(value: floor_join_digest_text(rows: rows) as NonEmptyStr))' '  serialize_content_hash(hash: content_hash_of_value(value: "constant" as NonEmptyStr))'; arm 4 "digest constant" $M2 $T2
echo "=== RESTORE CONTROL M2"; cmp -s $M2 $M2.bak && echo "bytes identical to pre-mutation"
;;
p5)
echo "##### PRODUCER ARM 5 + RESTORE over $T2"
mut $M2 '                identity_ref: DeclarationRef { module_path: owning_module, decl_name: row.function, field: WholeDeclaration },' '                identity_ref: DeclarationRef { module_path: owning_module, decl_name: "renamed", field: WholeDeclaration },'; arm 5 "declaration ref decl_name diverges from the dotted identity" $M2 $T2
echo "=== RESTORE CONTROL M2"; cmp -s $M2 $M2.bak && echo "bytes identical to pre-mutation"; echo "PASS count: $(runall $T2 | grep -cE '^PASS')"
;;
live)
echo "##### LIVE DIGEST (bounded)"
S0=$(date +%s); timeout 2400 $G run --source-root dag --source-root src/v2 --entry $M2 --function effect_demand_floor_join_digest_live 2>&1 | grep -vE "^advisory|^\s+\||^\s+[0-9]+ \||^✓|^$" | tail -6 || echo "(live digest exit $?)"; echo "live elapsed_s=$(( $(date +%s) - S0 ))"
;;
esac
