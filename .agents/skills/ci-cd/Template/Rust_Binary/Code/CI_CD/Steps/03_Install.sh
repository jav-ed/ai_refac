#!/bin/bash
# Atomically install the verified release binary on the configured remote host.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../Lib/runtime.sh"

ci_require_command scp
ci_require_command ssh
ci_require_value CI_BINARY_NAME
ci_require_value CI_INSTALL_HOST
ci_require_value CI_INSTALL_PATH

if [[ ! "$CI_INSTALL_HOST" =~ ^[A-Za-z0-9._-]+$ ]]; then
  echo "CI_INSTALL_HOST must be one reviewed SSH alias containing only letters, digits, dots, underscores, or dashes; received '$CI_INSTALL_HOST'." >&2
  exit 1
fi
if [[ ! "$CI_INSTALL_PATH" =~ ^/[A-Za-z0-9._/-]+$ ]]; then
  echo "CI_INSTALL_PATH must be one absolute path without spaces or shell metacharacters; received '$CI_INSTALL_PATH'." >&2
  exit 1
fi
if [[ "${CI_INSTALL_PATH##*/}" != "$CI_BINARY_NAME" ]]; then
  echo "Install-path basename '${CI_INSTALL_PATH##*/}' does not match CI_BINARY_NAME '$CI_BINARY_NAME'. Refusing to replace an ambiguously named remote file." >&2
  exit 1
fi

BINARY="$CI_PROJECT_DIR/target/release/$CI_BINARY_NAME"
if [[ ! -x "$BINARY" ]]; then
  echo "Release binary is missing at $BINARY. Run Code/CI_CD/Steps/02_Build.sh successfully before installation." >&2
  exit 1
fi

if [[ -n "${CI_PIPELINE_NUMBER:-}" ]]; then
  INSTALL_RUN_ID="pipeline-${CI_PIPELINE_NUMBER}"
else
  INSTALL_RUN_ID="manual-$$"
fi

TEMP_PATH="${CI_INSTALL_PATH}.tmp-${INSTALL_RUN_ID}"
INSTALL_DIR="${CI_INSTALL_PATH%/*}"
SSH=(ssh -o ClearAllForwardings=yes)

ci_heading "Install Rust binary"
echo "Source: $BINARY"
echo "Target: $CI_INSTALL_HOST:$CI_INSTALL_PATH"

if ! "${SSH[@]}" "$CI_INSTALL_HOST" "test -d '$INSTALL_DIR' && test -w '$INSTALL_DIR'"; then
  echo "Remote installation directory is missing or not writable: $CI_INSTALL_HOST:$INSTALL_DIR. The pipeline will not create or guess a destination." >&2
  exit 1
fi

scp -q -o ClearAllForwardings=yes "$BINARY" "${CI_INSTALL_HOST}:${TEMP_PATH}"
"${SSH[@]}" "$CI_INSTALL_HOST" "chmod 0755 '$TEMP_PATH' && mv '$TEMP_PATH' '$CI_INSTALL_PATH'"
"${SSH[@]}" "$CI_INSTALL_HOST" "'$CI_INSTALL_PATH' --version && '$CI_INSTALL_PATH' --help >/dev/null"

echo "Binary was installed atomically and verified at $CI_INSTALL_HOST:$CI_INSTALL_PATH."
