#!/usr/bin/env bash
# One command to install refac from this checkout and prove it is current:
# builds the release binary (a few seconds when nothing changed), links it as
# ~/.local/bin/refac, and checks that the `refac` found on the PATH is that
# build and was built from the commit that is checked out.
#
#   scripts/install.sh
#
# Run it after every pull, checkout or source change. It stops with a message
# and exit code 1 when the installed command is not this checkout's build.
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$root"

cargo build --release
binary="$root/target/release/refac"

mkdir -p "$HOME/.local/bin"
ln -sfn "$binary" "$HOME/.local/bin/refac"

found="$(command -v refac || true)"
if [ -z "$found" ]; then
    echo "refac is linked at ~/.local/bin/refac but that folder is not on the PATH: add it to the shell profile" >&2
    exit 1
fi
if [ "$(readlink -f "$found")" != "$(readlink -f "$binary")" ]; then
    echo "the refac on the PATH is $found, not this checkout's build $binary: remove or replace it" >&2
    exit 1
fi

version="$(refac --version)"
commit="$(git rev-parse --short HEAD 2>/dev/null || true)"
if [ -n "$commit" ] && [[ "$version" != *"($commit"* ]]; then
    echo "the installed refac says '$version' but the checkout is at $commit: the build did not pick up this commit" >&2
    exit 1
fi
echo "installed and current: $version"
echo "  $found -> $binary"
