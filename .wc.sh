set -u
cp $0 /tmp/wc_self.sh
leg() { # name sha
  git fetch -q --depth=1 origin $2 && git checkout -q -f FETCH_HEAD || { echo "fetch/checkout failed $1"; return; }
  echo "== LEG $1 head=$(git rev-parse HEAD)"
  export CARGO_TARGET_DIR=$PWD/target
  cargo build --release -p v1-compiler --bin gunbc 2>&1 | tail -1
  mkdir -p /sys/fs/cgroup/gunbcrun; echo +memory > /sys/fs/cgroup/cgroup.subtree_control; echo 21474836480 > /sys/fs/cgroup/gunbcrun/memory.max
  ( echo $BASHPID > /sys/fs/cgroup/gunbcrun/cgroup.procs
    timeout 3000 target/release/gunbc compile --output-dir /tmp/wc_$1 --source-root dag --source-root src/v2 --target rust --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json > /tmp/raw_$1.txt 2>&1
    echo "exit=$?" )
  tail -3 /tmp/raw_$1.txt | cut -c1-300
  grep -E "error" /tmp/raw_$1.txt | cut -c1-400 | sort > /tmp/errs_$1.txt; wc -l /tmp/errs_$1.txt
  echo "== E-BEGIN $1"; gzip -c /tmp/errs_$1.txt | base64 -w0; echo; echo "== E-END $1"
  rmdir /sys/fs/cgroup/gunbcrun 2>/dev/null
}
leg before ea035ecb0ea17f26f1e38694ebc4d6dc200eb772
leg after 6fd7d3b098763b46a19d4b32302a99d7690592e3
