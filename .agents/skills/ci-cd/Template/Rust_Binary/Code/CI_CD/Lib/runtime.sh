#!/bin/bash

CI_RUNTIME_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CI_PROJECT_DIR="$(cd "$CI_RUNTIME_DIR/../../.." && pwd)"
CI_CONFIG_FILE="$CI_PROJECT_DIR/Code/CI_CD/Config/env.env"

if [[ ! -f "$CI_CONFIG_FILE" ]]; then
  echo "CI configuration is missing at $CI_CONFIG_FILE. Restore the tracked configuration before running any CI command; no defaults will be guessed." >&2
  exit 1
fi

source "$CI_CONFIG_FILE"

if [[ "${CI_CONFIG_LOADED:-}" != "1" ]]; then
  echo "CI configuration did not set CI_CONFIG_LOADED=1. Refusing to run with a partial or unrelated environment file." >&2
  exit 1
fi

ci_require_command() {
  local command_name="$1"

  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Required command '$command_name' is unavailable. Install it on the Woodpecker agent or correct PATH before rerunning this step." >&2
    exit 1
  fi
}

ci_require_value() {
  local variable_name="$1"
  local variable_value="${!variable_name:-}"

  if [[ -z "$variable_value" || "$variable_value" == __SET_*__ ]]; then
    echo "Required CI variable '$variable_name' is empty or still contains a template placeholder. Set it explicitly in $CI_CONFIG_FILE; this pipeline does not invent project values." >&2
    exit 1
  fi
}

ci_heading() {
  echo ""
  echo "== $1 =="
}

# Establish the same Rust environment in interactive shells and Woodpecker.
ci_prepare_rust() {
  ci_require_value CI_RUST_TOOLCHAIN
  ci_require_value CI_RUSTUP_BIN
  ci_require_value CI_RUSTUP_HOME
  ci_require_value CI_CARGO_HOME

  if [[ ! -x "$CI_RUSTUP_BIN" ]]; then
    echo "Configured Rustup executable is missing or not executable at $CI_RUSTUP_BIN. Install Rustup there or correct CI_RUSTUP_BIN in $CI_CONFIG_FILE." >&2
    exit 1
  fi
  if [[ ! -d "$CI_RUSTUP_HOME" ]]; then
    echo "Configured Rustup home does not exist at $CI_RUSTUP_HOME. Install the pinned '$CI_RUST_TOOLCHAIN' toolchain there before running CI." >&2
    exit 1
  fi
  if [[ ! -d "$CI_CARGO_HOME" ]]; then
    echo "Configured Cargo home does not exist at $CI_CARGO_HOME. Create the reviewed directory or correct CI_CARGO_HOME in $CI_CONFIG_FILE." >&2
    exit 1
  fi

  export RUSTUP_HOME="$CI_RUSTUP_HOME"
  export CARGO_HOME="$CI_CARGO_HOME"

  if ! "$CI_RUSTUP_BIN" run "$CI_RUST_TOOLCHAIN" rustc --version; then
    echo "Pinned Rust toolchain '$CI_RUST_TOOLCHAIN' is unavailable under $CI_RUSTUP_HOME. Install that exact toolchain; no default will be substituted." >&2
    exit 1
  fi
  "$CI_RUSTUP_BIN" run "$CI_RUST_TOOLCHAIN" cargo --version
}

ci_rust() {
  "$CI_RUSTUP_BIN" run "$CI_RUST_TOOLCHAIN" "$@"
}
