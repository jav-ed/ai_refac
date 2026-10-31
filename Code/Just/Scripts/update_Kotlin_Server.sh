#!/usr/bin/env bash
# Installed at Code/Just/Scripts/update_Kotlin_Server.sh. Installs one build of the JetBrains
# Kotlin language server, checks it, and with --clean deletes the older builds and their caches.
# JetBrains publishes only early-access builds and each stops starting about four weeks after its
# release; the steps around this script are in Project_Manag/Docs/Setup/kotlin_Server.md.
set -euo pipefail

USAGE='Use: just kotlin-server-update <build, for example 263.7084.0> [--clean]'

fail() {
  printf 'Kotlin server update stopped: %s\n' "$*" >&2
  exit 1
}

[[ "$#" -ge 1 && "$#" -le 2 ]] || fail "$USAGE"
BUILD="$1"
[[ "$BUILD" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail "'$BUILD' is not a build number like 263.7084.0. $USAGE"
CLEAN=no
if [[ "$#" -eq 2 ]]; then
  [[ "$2" == "--clean" ]] || fail "$USAGE"
  CLEAN=yes
fi

# The new build goes next to the one in use; with none configured, where the install steps put it.
if [[ -n "${REFAC_KOTLIN_SERVER:-}" ]]; then
  PARENT="$(dirname -- "${REFAC_KOTLIN_SERVER%/}")"
else
  PARENT="$HOME/.local/share/refac"
fi
mkdir -p -- "$PARENT"
PARENT="$(cd -- "$PARENT" && pwd -P)"
DEST="$PARENT/kotlin-server-$BUILD"

# Where refac keeps the warm index of each build (src/drivers/kotlin/server/cache.rs).
CACHE=""
if [[ "${REFAC_KOTLIN_CACHE:-}" != "off" ]]; then
  CACHE="${REFAC_KOTLIN_CACHE:-${XDG_CACHE_HOME:-$HOME/.cache}/refac}"
fi

if [[ -f "$DEST/build.txt" && "$(tr -d '\n' < "$DEST/build.txt")" == "ILS-$BUILD" ]]; then
  printf 'Build %s is already installed at %s; nothing to download.\n' "$BUILD" "$DEST"
else
  [[ ! -e "$DEST" ]] || fail "$DEST exists but is not build $BUILD (its build.txt differs). Remove it by hand and run this again."
  URL="https://download.jetbrains.com/language-server/kotlin-server/$BUILD/kotlin-server-$BUILD.tar.gz"
  WORK="$(mktemp -d "$PARENT/.kotlin-server-update.XXXXXX")"
  trap 'rm -rf -- "$WORK"' EXIT
  printf 'Downloading %s\n' "$URL"
  curl -fL --retry 2 --progress-bar -o "$WORK/server.tar.gz" "$URL" || fail "the download failed. Is $BUILD a published build? Builds are listed at https://github.com/Kotlin/kotlin-lsp/releases"
  curl -fsSL --retry 2 -o "$WORK/server.sha256" "$URL.sha256" || fail "the checksum file of $BUILD could not be downloaded."
  # The published file names the checksum; compare it with the archive before unpacking anything.
  EXPECTED="$(awk '{print $1}' "$WORK/server.sha256")"
  ACTUAL="$(sha256sum "$WORK/server.tar.gz" | awk '{print $1}')"
  [[ "$EXPECTED" == "$ACTUAL" ]] || fail "checksum mismatch: published $EXPECTED, downloaded $ACTUAL. Nothing was installed."
  printf 'Checksum ok: %s\n' "$ACTUAL"
  mkdir -- "$WORK/unpacked"
  tar -xzf "$WORK/server.tar.gz" -C "$WORK/unpacked"
  [[ -f "$WORK/unpacked/kotlin-server-$BUILD/build.txt" ]] || fail "the archive does not hold kotlin-server-$BUILD/build.txt."
  [[ "$(tr -d '\n' < "$WORK/unpacked/kotlin-server-$BUILD/build.txt")" == "ILS-$BUILD" ]] || fail "the archive's build.txt does not say ILS-$BUILD."
  mv -- "$WORK/unpacked/kotlin-server-$BUILD" "$DEST"
  printf 'Installed %s\n' "$DEST"
  printf 'For src/servers/catalog/kotlin.rs: sha256 %s, %s bytes.\n' "$ACTUAL" "$(stat -c %s "$WORK/server.tar.gz")"
fi

# The license date is what decides how long the build is useful.
printf '\nLicense of the new build:\n'
LICENSE="$("$DEST/bin/intellij-server" license status 2>&1)" || fail "the new build does not start: $LICENSE"
printf '%s\n' "$LICENSE"
if grep -qi 'expired' <<<"$LICENSE"; then
  fail "build $BUILD has expired already. Install a newer one from https://github.com/Kotlin/kotlin-lsp/releases"
fi

# Older builds: the server folders next to the new one, and refac's warm index of each build.
OLD=()
for folder in "$PARENT"/kotlin-server-*; do
  [[ -d "$folder" && ! -L "$folder" && "$folder" != "$DEST" && -f "$folder/build.txt" ]] || continue
  [[ "$(head -c 4 "$folder/build.txt")" == "ILS-" ]] || continue
  OLD+=("$folder")
done
if [[ -n "$CACHE" ]]; then
  for folder in "$CACHE"/kotlin-[0-9]*; do
    [[ -d "$folder" && ! -L "$folder" && "$folder" != "$CACHE/kotlin-$BUILD" ]] || continue
    OLD+=("$folder")
  done
fi

if [[ "${#OLD[@]}" -eq 0 ]]; then
  printf '\nNo older Kotlin server builds or caches found.\n'
elif [[ "$CLEAN" == yes ]]; then
  printf '\nDeleting older builds and caches:\n'
  for folder in "${OLD[@]}"; do
    printf '  %s (%s)\n' "$folder" "$(du -sh -- "$folder" | cut -f1)"
    rm -rf -- "$folder"
  done
else
  printf '\nOlder builds and caches (run again with --clean to delete them):\n'
  for folder in "${OLD[@]}"; do
    printf '  %s (%s)\n' "$folder" "$(du -sh -- "$folder" | cut -f1)"
  done
fi

printf '\nNext steps (details: Project_Manag/Docs/Setup/kotlin_Server.md, "Replacing the build"):\n'
printf '  1. export REFAC_KOTLIN_SERVER=%s   (in the shell profile)\n' "$DEST"
printf '  2. refac doctor kotlin   must say [ready]\n'
printf '  3. change the pinned build, URL and checksum in src/servers/catalog/kotlin.rs, then run the real-server tests once\n'
if [[ -n "${REFAC_KOTLIN_SERVER:-}" && "${REFAC_KOTLIN_SERVER%/}" != "$DEST" ]]; then
  printf '\nREFAC_KOTLIN_SERVER still names %s' "${REFAC_KOTLIN_SERVER%/}"
  [[ -d "${REFAC_KOTLIN_SERVER%/}" ]] || printf ', which does not exist any more'
  printf '. Change it before the next Kotlin command.\n'
fi
