set -u
G=/sys/fs/cgroup/kq$$; mkdir -p $G && echo 48G > $G/memory.max && echo $$ > $G/cgroup.procs
H=${1:?head}; BR=${2:?branch}
git fetch -q origin $BR; d=/tmp/q; git worktree add -q --detach $d $H; cd $d
echo "TREE $(git rev-parse HEAD) PARENTS $(git log -1 --format=%P)"
cargo build --release -p v1-compiler --bin claim_batch 2>&1 | grep -aE "^error" | head -3
B=target/release/claim_batch; echo "BIN $(sha256sum $B | cut -c1-16)"
R="--wet --source-root dag --source-root src/v2"
cb(){ $B $R --entry $1 --functions "$2" > /tmp/cb.out 2>&1; echo "EXIT $?"; grep -aE '^(PASS|FAIL|UNOBS)' /tmp/cb.out | cut -c1-400; }
shift 2
for f in "$@"; do FNS=$(grep -aoE '^test fn [a-z0-9_]+' $f | sed 's/test fn //' | paste -sd,); echo "== $f ($(echo $FNS | tr , '\n' | wc -l) claims)"; cb $f "$FNS"; done
D=dag/gunbc/design_ruling.dag; W=dag/test/claim/documentary_refs_witness.dag; C=documentary_refs_every_ruling_standing_agrees_with_the_document,documentary_refs_agreeing_standings_are_admitted
flip(){ python3 -I - "$D" "$1" "$2" <<'P'
import sys; p,r,to=sys.argv[1:]; s=open(p).read()
i=s.index(f"data {r}: DesignRuling = DesignRuling {{\n"); j=s.index("\n}\n",i); seg=s[i:j]
a=seg.index("  standing:"); b=seg.index("\n",a) if "\n" in seg[a:] else len(seg)
new='  standing: CitedInDesign {},' if to=='C' else '  standing: LedgerOnly { reason: "red" },'
print("FLIP", r, repr(seg[a:b][:60]), "->", new); s=s[:i]+seg[:a]+new+seg[b:]+s[j:]; open(p,'w').write(s)
P
}
echo "== RED 1: where_refinements_retire_for_checked_constructors -> LedgerOnly (cited row)"; flip where_refinements_retire_for_checked_constructors L; cb $W $C; git checkout -q -- $D
echo "== RED 2: approval_enrolment_is_google_sign_in -> CitedInDesign (uncited row)"; flip approval_enrolment_is_google_sign_in C; cb $W $C; git checkout -q -- $D
