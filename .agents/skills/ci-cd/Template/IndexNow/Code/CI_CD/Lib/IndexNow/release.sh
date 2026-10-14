#!/bin/bash

ci_indexnow_release_root() {
  local lane_dir="$1"
  local release_target="$2"

  if [[ ! "$release_target" =~ ^releases/[A-Za-z0-9._-]+$ ]]; then
    echo "Refusing IndexNow operation: release target must match releases/<release-id>, received '$release_target'." >&2
    return 1
  fi

  printf '%s/%s\n' "$lane_dir" "$release_target"
}

ci_indexnow_verify_local_key() {
  local release_root="$1"
  local key_file="$release_root/$CI_INDEXNOW_KEY_FILE_NAME"
  local key_value="${CI_INDEXNOW_KEY_FILE_NAME%.txt}"

  ci_indexnow_require_command cmp

  if [[ ! -f "$key_file" ]]; then
    echo "Refusing IndexNow operation: current release does not contain the configured root ownership file at $key_file." >&2
    exit 1
  fi
  if ! cmp -s "$key_file" <(printf '%s' "$key_value") && ! cmp -s "$key_file" <(printf '%s\n' "$key_value"); then
    echo "Refusing IndexNow operation: $key_file must contain exactly the key encoded by its filename, with at most one final newline." >&2
    exit 1
  fi
}
