#!/usr/bin/env bash
# Packs a release archive from an existing release build:
#
#   cargo build --release --locked && scripts/package_release.sh linux-x86_64
#
# The archive is dist/refac-<version>-<target>.tar.gz with one folder inside: the binary and the
# files it reads at run time (scripts/, pyrefly.toml), taken from what git tracks, so test files
# and node_modules never get in. refac finds scripts/ next to its executable, so the folder must
# stay together. The helper packages are installed on first use (Bun and network needed).
set -euo pipefail

target="${1:?usage: scripts/package_release.sh <target-name>, for example linux-x86_64}"
root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

binary="target/release/refac"
[[ -x "$binary" ]] || { echo "package_release: $binary does not exist. Run 'cargo build --release --locked' first." >&2; exit 1; }

version="$(sed -n 's/^version = "\(.*\)"$/\1/p' Cargo.toml | head -n 1)"
[[ -n "$version" ]] || { echo "package_release: no version line in Cargo.toml." >&2; exit 1; }
built="$("$binary" --version | awk '{print $2}')"
[[ "$built" == "$version" ]] || { echo "package_release: $binary is version $built, Cargo.toml says $version. Rebuild it." >&2; exit 1; }

name="refac-${version}-${target}"
stage="dist/${name}"
rm -rf "$stage" "dist/${name}.tar.gz"
mkdir -p "$stage"

cp "$binary" "$stage/refac"
git ls-files -z -- scripts pyrefly.toml LICENSE README.md \
    | grep -zv -e '^scripts/Tests/' -e '^scripts/install.sh$' -e '^scripts/package_release.sh$' \
    | xargs -0 cp --parents -t "$stage"

for needed in refac scripts/ts_refactor.ts scripts/python_refactor.py scripts/package.json scripts/bun.lock pyrefly.toml; do
    [[ -e "$stage/$needed" ]] || { echo "package_release: $needed is missing from the archive folder." >&2; exit 1; }
done

tar -C dist -czf "dist/${name}.tar.gz" "$name"
rm -rf "$stage"
echo "dist/${name}.tar.gz ($(du -h "dist/${name}.tar.gz" | cut -f1))"
