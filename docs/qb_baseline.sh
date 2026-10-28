set -u
git fetch --depth=1 origin 507e413106929bed5984edcf330704bfbb145ef1 2>&1 | tail -1
git worktree add --detach /tmp/qbbase 507e413106929bed5984edcf330704bfbb145ef1 2>&1 | tail -1
test -d /tmp/qbbase/src || { echo "NO BASE TREE"; exit 3; }
echo "base head: $(git -C /tmp/qbbase rev-parse HEAD)"
bash docs/qb_census.sh base /tmp/qbbase
