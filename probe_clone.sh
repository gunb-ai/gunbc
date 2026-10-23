#!/bin/bash
# Can a dispatch clone from origin at all? If the runner has no git credentials, the fresh-clone
# instrument is impossible and every number stays attributable to the SHARED workspace instead of a
# SHA -- which is the thing sharp-boar-42 says made my drift result wrong. Cheap to find out.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
D=/tmp/freshprobe; rm -rf $D; mkdir -p $D; cd $D
echo "git=$(git --version)"
timeout 300 git clone --depth 1 https://github.com/gunb-ai/gunbc.git repo > clone.log 2>&1; rc=$?
echo "clone rc=$rc"; tail -5 clone.log
if [ -d repo ]; then cd repo; echo "HEAD=$(git rev-parse HEAD)"; echo "branch=$(git rev-parse --abbrev-ref HEAD)"; echo "status_clean=$(git status --porcelain | wc -l) dirty lines"; du -sh .git; fi
echo "--- fetch a specific SHA (what the instrument needs):"
timeout 300 git fetch --depth 1 origin bd68035135c62c17dd5be2777ff9770955b58f50 > fetch.log 2>&1; echo "fetch rc=$?"; tail -3 fetch.log
git show FETCH_HEAD:carriers/README.md 2>&1 | head -3
git show FETCH_HEAD:tools/rebind_census.sh 2>&1 | head -2
echo PROBE_CLONE_DONE
