#!/bin/bash
set -u
source <(sed -n '/^B()/p;/^regen()/,/^}/p;/^wit()/,/^}/p' tools/.phaseb_run.sh)
python3 - <<'P'
p='src/v1/04_infer.dag'; s=open(p).read()
a=s.index('fn nominal_value_at_host_text_diags'); o='  if value_type_carries_optional_layer(t: t, source_indices: scope.type_env.source_indices) {'; b=s.index(o,a)
s=s[:b]+'  if true { [] } else if value_type_carries_optional_layer(t: t, source_indices: scope.type_env.source_indices) {'+s[b+len(o):]
open(p,'w').write(s); print("MUTATION APPLIED")
P
git diff --stat -- src/v1/04_infer.dag
B; regen
wit MUTANT
