#!/bin/bash
set -u
source <(sed -n '/^B()/p;/^regen()/,/^}/p' tools/.phaseb_run.sh)
python3 - <<'P'
p='src/v1/04_infer.dag'; s=open(p).read()
a=s.index('fn nominal_value_at_host_text_diags'); o='  if value_type_carries_optional_layer(t: t, source_indices: scope.type_env.source_indices) {'; b=s.index(o,a)
s=s[:b]+'  if true { [] } else if value_type_carries_optional_layer(t: t, source_indices: scope.type_env.source_indices) {'+s[b+len(o):]
open(p,'w').write(s); print("MUTATION APPLIED")
P
B; regen
one() { r=$(timeout 900 ./target/release/gunbc run --source-root dag --source-root src/v2 --entry dag/test/claim/$1.dag --function $2 2>&1 | grep -o 'returned `[a-z]*`' | head -1); echo "MUTANT $1 $2 :: ${r:-NO-RESULT}"; }
one nominal_host_text_argument_wall_witness control_a_list_concat_through_an_unsolved_accumulator_is_accepted
one nominal_host_text_argument_wall_witness two_records_at_concat_are_refused
for fn in $(grep -o '^test fn [a-z_0-9]*' dag/test/claim/nominal_cast_wall_witness.dag | awk '{print $3}'); do one nominal_cast_wall_witness $fn; done
