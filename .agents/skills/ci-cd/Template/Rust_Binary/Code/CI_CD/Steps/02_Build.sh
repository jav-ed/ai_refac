#!/bin/bash
# Build the locked optimized binary and verify its operator-facing CLI.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../Lib/runtime.sh"

ci_prepare_rust
ci_require_value CI_BINARY_NAME
cd "$CI_PROJECT_DIR"

ci_heading "Build Rust release binary"
ci_rust cargo build --locked --release

BINARY="$CI_PROJECT_DIR/target/release/$CI_BINARY_NAME"
if [[ ! -x "$BINARY" ]]; then
  echo "Cargo reported success but the configured executable is missing at $BINARY. Confirm CI_BINARY_NAME matches the Cargo binary target." >&2
  exit 1
fi

"$BINARY" --version
"$BINARY" --help >/dev/null
echo "Release binary built and its version/help commands succeeded: $BINARY"
