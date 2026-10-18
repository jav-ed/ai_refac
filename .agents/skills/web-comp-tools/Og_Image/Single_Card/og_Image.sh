#!/usr/bin/env bash
# Renders an HTML social card to a 1200 by 630 PNG with playwright-cli.
#
#   og_Image.sh [CARD_HTML] [OUT_PNG]       defaults: Code/Brand/og-card.html and public/og.png
#
# Copy this file into the site, for example to Code/Just/Scripts/og_Image.sh, and run it from there
# (it works from the repository root, three folders above itself). The PNG is a committed file: run
# the script after changing the card, look at the result, then commit it.
# playwright-cli blocks file: URLs, so the repository is served on localhost for the render.
# Variables: OG_FONT is the CSS font to wait for (default "700 40px Geist": use the card's own font),
# OG_PORT the localhost port (default 4175), PW_BROWSER another installed browser.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/../../.."
card="${1:-Code/Brand/og-card.html}"
out="${2:-public/og.png}"
font="${OG_FONT:-700 40px Geist}"
port="${OG_PORT:-4175}"

command -v playwright-cli >/dev/null || { echo "og_Image: playwright-cli is not installed (see the playwright-cli skill)." >&2; exit 1; }
command -v identify >/dev/null || { echo "og_Image: ImageMagick (identify) is not installed." >&2; exit 1; }
command -v python3 >/dev/null || { echo "og_Image: python3 is not installed (it serves the card on localhost)." >&2; exit 1; }
[[ -f "$card" ]] || { echo "og_Image: the card $card does not exist." >&2; exit 1; }
curl -s -o /dev/null --max-time 2 "http://127.0.0.1:$port/" && { echo "og_Image: something already answers on port $port. Set OG_PORT to a free port." >&2; exit 1; }

# The server is tracked by its PID, never by `pkill -f`: that pattern also matches this shell.
python3 -m http.server "$port" --bind 127.0.0.1 >/dev/null 2>&1 &
server_pid=$!
cleanup() { playwright-cli close >/dev/null 2>&1 || true; kill "$server_pid" 2>/dev/null || true; }
trap cleanup EXIT
for _ in $(seq 50); do curl -s -o /dev/null "http://127.0.0.1:$port/" && break; sleep 0.1; done
curl -s -o /dev/null "http://127.0.0.1:$port/" || { echo "og_Image: the local server did not start on port $port." >&2; exit 1; }

playwright-cli open ${PW_BROWSER:+--browser="$PW_BROWSER"} "http://127.0.0.1:$port/$card" >/dev/null
playwright-cli resize 1200 630 >/dev/null
# The web font loads after the page: the shot waits until the face is usable.
# playwright-cli prints the value on the line after "### Result".
fonts="$(playwright-cli eval "document.fonts.ready.then(() => document.fonts.check('$font'))" | awk '/^### Result/ {getline; print; exit}')"
[[ "$fonts" == true ]] || { echo "og_Image: the font '$font' did not load (got: $fonts). Check the font path in $card." >&2; exit 1; }
case "$out" in /*) target="$out" ;; *) target="$PWD/$out" ;; esac
mkdir -p "$(dirname "$target")"
playwright-cli screenshot --filename="$target" >/dev/null

size="$(identify -format '%wx%h' "$target")"
[[ "$size" == "1200x630" ]] || { echo "og_Image: $out is $size, expected 1200x630." >&2; exit 1; }
echo "wrote $out ($size)"
