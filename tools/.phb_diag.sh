#!/bin/bash
set -u
source <(sed -n '/^B()/p;/^regen()/,/^}/p;/^wit()/,/^}/p' tools/.phaseb_run.sh)
B; regen
wit HEAD
python3 - <<'P'
p='src/v1/04_infer.dag'; s=open(p).read()
a=s.index('fn nominal_value_at_host_text_diags'); o='  if value_type_is_declared_nominal(t: t, scope: scope) {'; b=s.index(o,a)
s=s[:b]+'  if false && value_type_is_declared_nominal(t: t, scope: scope) {'+s[b+len(o):]
open(p,'w').write(s); print("MUTATION APPLIED")
P
git diff --stat -- src/v1/04_infer.dag
regen
wit MUTANT
