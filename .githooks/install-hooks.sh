#!/usr/bin/env bash
# One-shot: point git at the repo-tracked hooks directory so
# .githooks/* runs on the relevant events.
#
# After this, `git push` will run .githooks/pre-push locally and
# fail fast on fmt drift, before CI runs.

set -e

git config core.hooksPath .githooks

chmod +x .githooks/pre-push

echo "[install-hooks] core.hooksPath set to .githooks"
echo "[install-hooks] .githooks/pre-push is executable"
echo "[install-hooks] Test: try 'git push --dry-run' — the hook should run."
