cd /home/briansrls/.worktrees/gunbc/jolly-bee-729
for f in compiler/infer_optional_at_required_witness compiler/infer_application_argument_inhabitance_witness body_let_annotation; do n=$(grep -oE "^test fn [a-z0-9_]+" src/v2/test/claim/$f.dag | awk '{print $3}' | paste -sd,); /tmp/jb/target/release/claim_batch --source-root dag --source-root src/v2 --entry src/v2/test/claim/$f.dag --functions "$n" > /tmp/jb/base_$(basename $f).out 2>&1; echo rc=$? >> /tmp/jb/base_$(basename $f).out; done
cp /tmp/jb/inh_fixed.dag src/v2/std/inhabitance.dag; echo finished > /tmp/jb/base.done
