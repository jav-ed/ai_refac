#!/bin/bash

# CI scripts are the real execution layer. Just may call them as local
# shortcuts, but Woodpecker runs them directly, so this file owns repo/env setup.
CI_RUNTIME_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
export CI_PROJECT_DIR="$(cd "$CI_RUNTIME_DIR/../../.." && pwd)"

CI_ENV_FILE="$CI_PROJECT_DIR/Code/CI_CD/Config/env.env"

if [[ ! -f "$CI_ENV_FILE" ]]; then
  echo "Refusing to continue: CI env file is missing: ${CI_ENV_FILE}" >&2
  exit 1
fi

# shellcheck source=/dev/null
source "$CI_ENV_FILE"

: "${CI_CONFIG_LOADED:?Code/CI_CD/Config/env.env was not loaded.}"

ci_require_value() {
  local name="$1"
  local value="${!name:-}"

  if [[ -z "$value" || "$value" == __REQUIRED_* ]]; then
    echo "Refusing to continue: ${name} is not configured." >&2
    echo "Set the real value in Code/CI_CD/Config/env.env. Do not guess deploy values." >&2
    exit 1
  fi
}
