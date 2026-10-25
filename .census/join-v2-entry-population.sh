set -u
export PATH=$HOME/.cargo/bin:$PATH
memwait() { until [ $(awk '/MemAvailable/ {print int($2/1048576)}' /proc/meminfo) -ge $1 ]; do sleep 60; done; }
W=~/wc-census; cd $W
build() { memwait 30; systemd-run --user --scope --quiet -p Slice=gunbc-user-work.slice -p MemoryMax=24G -p MemorySwapMax=0 env CARGO_TARGET_DIR=$W/target cargo build --release -p v1-compiler --bin gunbc --bin claim_executor > ~/after-build$1.log 2>&1; local rc=$?; echo "build$1 exit=$rc $(tail -1 ~/after-build$1.log | tr -d '\0')"; return $rc; }
echo "=== PART A: main-only leg (wall refusals on current main) start $(date +%T)"
git fetch -q origin main || { echo FETCH-FAIL; exit 2; }
M=$(git rev-parse origin/main); git checkout -q -f --detach $M && git clean -fdq -e target && echo "main at $M dirty=$(git status --short | wc -l); wall in main's infer mirror: $(grep -c 'at host text' src/v1/stage0/src/v1_compiler_infer.rs)"
build m || exit 3
memwait 64; echo "compile main start $(date +%T)"; rm -rf ~/after-main-out
env -u CURSOR_API_KEY timeout 7200 systemd-run --user --scope --quiet -p Slice=gunbc-user-work.slice -p MemoryMax=60G -p MemorySwapMax=0 $W/target/release/gunbc compile --output-dir ~/after-main-out --source-root dag --source-root src/v2 --target rust --repository gunbc --measured-root-demands tools/whole_corpus_compile_measured_root_demands.json > ~/after-main-compile.log 2>&1
echo "compile main exit=$? $(date +%T) $(grep -E 'blocking error' ~/after-main-compile.log | tail -1)"
grep 'at host text' ~/after-main-compile.log | sort -u > ~/after-main-wall.txt; echo "WALL REFUSALS ON MAIN $M: $(wc -l < ~/after-main-wall.txt)"; cut -c1-240 ~/after-main-wall.txt | head -30
echo "=== PART B: join census src/v2 per-file entries start $(date +%T)"
C=f66074a88b27c6b2b220eb5eb12973fbd93d00f9
git fetch -q origin census-local-join && git checkout -q -f --detach $C && git clean -fdq -e target && echo "at $(git rev-parse HEAD)"
build 0 || exit 3
rm -rf target/stage0-regen-candidate; memwait 44
env -u CURSOR_API_KEY timeout 3600 systemd-run --user --scope --quiet -p Slice=gunbc-user-work.slice -p MemoryMax=40G -p MemorySwapMax=0 $W/target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > ~/after-regen1.log 2>&1; echo "regen1 exit=$?"
n=0; while IFS= read -r f; do rel=${f#target/stage0-regen-candidate/src/}; if ! cmp -s "$f" "src/v1/stage0/src/$rel"; then cp "$f" "src/v1/stage0/src/$rel"; n=$((n+1)); fi; done < <(find target/stage0-regen-candidate/src -type f | sort); echo "installed $n; census text: $(grep -c JOINCENSUS src/v1/stage0/src/v1_compiler_infer.rs)"
build 1 || exit 3
i=0; : > ~/after-join-entry.txt
while IFS= read -r f; do i=$((i+1)); [ -n "$f" ] || continue
  memwait 30
  env -u CURSOR_API_KEY timeout 1200 systemd-run --user --scope --quiet -p Slice=gunbc-user-work.slice -p MemoryMax=30G -p MemorySwapMax=0 $W/target/release/gunbc compile --output-dir ~/after-je-$i --source-root dag --source-root src/v2 --entry "$f" --target rust > ~/after-je-$i.log 2>&1; rc=$?
  k=$(grep -E "^error\[$f:" ~/after-je-$i.log | grep -c JOINCENSUS)
  echo "ENTRY $f rc=$rc census_in_file=$k" | tee -a ~/after-join-entry.txt
  grep -E "^error\[$f:" ~/after-je-$i.log | grep JOINCENSUS | sed 's/^/SITE /' >> ~/after-join-entry.txt
  rm -rf ~/after-je-$i
done < ~/join-entry-files.txt
echo "ENTRY-DONE $(date +%T) sites: $(grep -c '^SITE ' ~/after-join-entry.txt)"
