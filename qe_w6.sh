cd "$(git rev-parse --show-toplevel)"
export CARGO_TARGET_DIR=$PWD/target
cp dag/test/claim/optional_leaf_instance_join_witness.dag /tmp/w_orig2.dag
bash qe_w4.sh > /tmp/qe_w6_inner.log 2>&1
grep -a '^rc=' /tmp/qe_w6_inner.log
