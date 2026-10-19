set -e
cd /root/workspace/repo-root
echo "HEAD=$(git rev-parse HEAD)"
echo "COMMIT-EXISTS=$(git cat-file -t a36e08620e63649dee42a9c49ba749ab6e373862 2>&1)"
echo "DIFF-SIZE=$(git diff 65b25985167c5abbaf5a6df2b26218fe618a13f3 a36e08620e63649dee42a9c49ba749ab6e373862 -- src/v1/stage0 | wc -c)"
echo "REMOTE:"; git remote -v | head -2
echo "CRED-HELPER=$(git config --get credential.helper || echo none)"
echo "NETRC=$([ -f ~/.git-credentials ] && echo yes || echo no)"
echo "PUSH-TRY:"
set +e
git push -q origin a36e08620e63649dee42a9c49ba749ab6e373862:refs/heads/session/bright-swift-151-mirrors 2>&1 | head -4
echo "PUSH-RC=$?"
set -e
echo "ZZ-DONE"
