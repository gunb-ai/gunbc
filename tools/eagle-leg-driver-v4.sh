#!/usr/bin/sh
# eagle-leg-driver-v4 (crisp-eagle-244, run11+). v4 adds what run10 proved was missing: when
# TRACKED_DIRTY is nonzero the boot line NAMES the dirty tracked paths (comma-joined, cut from
# the porcelain path column) and tags OPTION=b_dirty_paths_named, so a leg that judges in-run
# spliced bytes carries its own doctrine-option-(b) evidence in its log instead of a stale
# OPTION=a label on dirty bytes (run10 claims-a printed OPTION=a_clean_checkout at
# TRACKED_DIRTY=1 -- the dirt was this run's fold-legs splice of src/v2/workflow/
# floor_preparation_split.dag; the splice receipt was the proof, and now the boot line is too).
# v3 (the version below) added: The leg-layer pin enforcement this layer lacked:
# a leg must refuse to exec a program whose on-disk sha256 is not the model's pinned sha -- the
# supply check upstream is a model-visible gate but its view of the binaries predates the leg; the
# leg judges the bytes it is about to run. v2 (sha 19df9ae3381aeb40bcf1549705b036528313a749e1ae4080ce0772349f3e082a)
# added the acquisition-identity block but no pin; the capture script's external pin guard is
# outside the model and must not be the only layer.
#
# argv[1] is the log path; the rest is program+arguments passed verbatim to exec, streams appended
# to that log. No shell string is built from leg data. The identity block prints, INSIDE the leg
# and from the leg's own process, what the leg judges: pid/owner, cgroup, HEAD/TREE/TRACKED_DIRTY
# (tracked dirt only, --untracked-files=no per the repo_self_build.dag:302-311 doctrine: untracked
# scratch reaches neither rustc nor the identity; at these legs it is zero by construction -- a
# clean checkout of the judged head), and sha256+bytes of every file operand.
LOG="$1"
shift
: > "$LOG"
PIN_CLAIM_BATCH="b395a556775931296f8f8aa8b9307f304d34442579f88d749189e273a4b0aee5"
{
  echo "[leg-boot $(date -u +%FT%TZ)] pid=$$ owner=$(id -un) ppid=$PPID driver=v4"
  echo "[leg-cgroup] $(cat /proc/self/cgroup 2>/dev/null | head -1)"
  TRACK_OUT=$(git status --porcelain --untracked-files=no 2>/dev/null || true)
  if [ -z "$TRACK_OUT" ]; then TRACK_DIRTY=0; TRACK_PATHS=none; OPTION_TAG=a_clean_checkout; else TRACK_DIRTY=$(printf '%s\n' "$TRACK_OUT" | grep -c '^[^?]' || true); TRACK_PATHS=$(printf '%s\n' "$TRACK_OUT" | cut -c4- | sort -u | tr '\n' ',' | sed 's/,$//'); OPTION_TAG=b_dirty_paths_named; fi
  echo "[leg-git] HEAD=$(git rev-parse HEAD 2>/dev/null) TREE=$(git rev-parse 'HEAD^{tree}' 2>/dev/null) TRACKED_DIRTY=$TRACK_DIRTY OPTION=$OPTION_TAG TRACKED_PATHS=$TRACK_PATHS"
  for a in "$@"; do
    if [ -f "$a" ]; then
      echo "[leg-binary] path=$a bytes=$(stat -c%s "$a") sha256=$(sha256sum "$a" | cut -d' ' -f1)"
      case "$a" in
        */claim_batch)
          if [ "$(sha256sum "$a" | cut -d' ' -f1)" != "$PIN_CLAIM_BATCH" ]; then
            echo "[leg-refuse] claim_batch sha256 is not the pinned release: $a"
            exit 97
          fi
          ;;
      esac
    fi
  done
  echo "[leg-argv] $*"
} >> "$LOG" 2>&1
exec >>"$LOG" 2>&1 "$@"
