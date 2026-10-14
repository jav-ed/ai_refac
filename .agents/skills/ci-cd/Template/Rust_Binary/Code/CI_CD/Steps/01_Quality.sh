#!/bin/bash
# Run every non-mutating Rust quality gate before a release build.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../Lib/runtime.sh"

ci_prepare_rust
cd "$CI_PROJECT_DIR"

ci_heading "Rust quality gate"
echo "Repository: $CI_PROJECT_DIR"
echo "Checks: rustfmt, all tests, Clippy with warnings denied"

ci_rust cargo fmt --all -- --check
ci_rust cargo test --locked
ci_rust cargo clippy --locked --all-targets --all-features -- -D warnings

echo "Quality gate passed. No source file, test, format, or Clippy failure was ignored."
