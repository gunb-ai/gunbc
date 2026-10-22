#!/bin/bash
# Watch #14715 merged + origin/main contains e86cbfeafae (the heal unblocker).
for i in $(seq 1 120); do
  ST=$(gh pr view 14715 --json state -q .state 2>/dev/null)
  if [ "$ST" = "MERGED" ]; then
    git fetch -q origin main
    if git merge-base --is-ancestor e86cbfeafae9e02456d7d61a0c1a1bc0237bfc3b origin/main 2>/dev/null; then
      echo "C-LANDED-AND-ON-MAIN $(git rev-parse origin/main | cut -c1-11)"; exit 0
    fi
  fi
  sleep 60
done
echo "TIMEOUT state=$ST"; exit 2
