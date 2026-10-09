#!/usr/bin/env bash
# Installed at Code/Just/Scripts/clean_Scratch.sh; all Scratch contents are disposable.
set -euo pipefail

fail() {
  printf 'Scratch reset refused: %s\n' "$*" >&2
  exit 1
}

[[ "$#" -eq 0 ]] || fail 'Use just scratch-clean; no paths or options are accepted.'
CLEANUP_SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
[[ "$CLEANUP_SCRIPT_DIR" == */Code/Just/Scripts ]] || fail 'Install this helper at Code/Just/Scripts/clean_Scratch.sh before invoking it.'
CLEANUP_PROJECT_ROOT="$(cd -- "$CLEANUP_SCRIPT_DIR/../../.." && pwd -P)"
CLEANUP_SCRATCH_DIR="$CLEANUP_PROJECT_ROOT/Scratch"

if [[ -L "$CLEANUP_SCRATCH_DIR" || ( -e "$CLEANUP_SCRATCH_DIR" && ! -d "$CLEANUP_SCRATCH_DIR" ) ]]; then
  fail "$CLEANUP_SCRATCH_DIR must be a real directory, not a symlink or file."
fi

mkdir -p -- "$CLEANUP_SCRATCH_DIR"
# Include hidden entries. rm unlinks symlinks and hard links without rewriting their targets.
shopt -s dotglob nullglob
for scratch_entry in "$CLEANUP_SCRATCH_DIR"/*; do
  rm -rf -- "$scratch_entry" || fail "Could not remove $scratch_entry."
done
for scratch_folder in Agent_Tasks Audit Design Screenshots; do
  mkdir -- "$CLEANUP_SCRATCH_DIR/$scratch_folder"
done
printf 'Scratch reset. Empty folders: Agent_Tasks, Audit, Design, Screenshots.\n'
