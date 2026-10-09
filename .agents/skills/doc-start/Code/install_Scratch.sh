#!/usr/bin/env bash
# Install the cleanup command without running it or replacing an existing implementation.
set -euo pipefail

fail() {
  printf 'Scratch command installation refused: %s\n' "$*" >&2
  exit 1
}

[[ "$#" -le 1 ]] || fail 'Usage: install_Scratch.sh [repo-root]'
INSTALL_SCRIPT_DIR="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)"
INSTALL_TEMPLATE="$INSTALL_SCRIPT_DIR/../Templates/clean_Scratch.sh"
cd -- "${1:-.}"

justfile_path=''
recipe_summary=''
for candidate in justfile Justfile .justfile; do
  if [[ -e "$candidate" || -L "$candidate" ]]; then
    [[ -z "$justfile_path" ]] || fail "Multiple Justfiles found: $justfile_path and $candidate."
    [[ -f "$candidate" && ! -L "$candidate" ]] || fail "$candidate must be a regular file."
    justfile_path="$candidate"
  fi
done

if [[ -n "$justfile_path" ]]; then
  command -v just >/dev/null 2>&1 || fail 'Install Just to inspect existing recipes before adding scratch-clean.'
  recipe_summary="$(just --justfile "$justfile_path" --summary)" || fail "Cannot parse $justfile_path."
  if [[ "$recipe_summary" =~ (^|[[:space:]])scratch-clean($|[[:space:]]) ]]; then
    printf '  exists:  scratch-clean recipe retained; verify its cleanup contract.\n'
    exit 0
  fi
else
  justfile_path='justfile'
fi

for directory in Code Code/Just Code/Just/Scripts; do
  [[ ! -L "$directory" ]] || fail "Helper directory must not be a symlink: $directory"
done
helper_path='Code/Just/Scripts/clean_Scratch.sh'
if [[ -e "$helper_path" || -L "$helper_path" ]]; then
  [[ -f "$helper_path" && ! -L "$helper_path" ]] || fail "$helper_path must be a regular file."
  cmp -s "$INSTALL_TEMPLATE" "$helper_path" || fail "Existing $helper_path differs from the template; review it before installation."
else
  mkdir -p Code/Just/Scripts
  cp -- "$INSTALL_TEMPLATE" "$helper_path"
fi

# A freshly scaffolded project's default must list commands, never run a destructive recipe.
if [[ -z "$recipe_summary" ]]; then
  printf '\ndefault:\n    @just --list\n' >> "$justfile_path"
fi
cat >> "$justfile_path" <<'EOF'

[doc('Delete all Scratch contents and recreate empty Agent_Tasks, Audit, Design, and Screenshots folders.')]
scratch-clean:
    bash Code/Just/Scripts/clean_Scratch.sh
EOF
printf '  installed: scratch-clean in %s (cleanup not run).\n' "$justfile_path"
