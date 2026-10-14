#!/bin/bash

ci_indexnow_require_command() {
  local command_name="$1"

  if ! command -v "$command_name" >/dev/null 2>&1; then
    echo "Refusing IndexNow operation: required command '$command_name' is unavailable in PATH." >&2
    exit 1
  fi
}

# Validate the project-owned protocol values before deployment changes state.
ci_indexnow_require_common_config() {
  ci_require_value CI_INDEXNOW_SITE_URL
  ci_require_value CI_INDEXNOW_SITEMAP_PATH
  ci_require_value CI_INDEXNOW_KEY_FILE_NAME
  ci_require_value CI_INDEXNOW_ENDPOINT
  ci_require_value CI_INDEXNOW_TIMEOUT_SECONDS
  ci_require_value CI_INDEXNOW_REPORT_DIR_NAME

  if [[ ! "$CI_INDEXNOW_SITE_URL" =~ ^https://[^/]+/$ ]]; then
    echo "Refusing IndexNow operation: CI_INDEXNOW_SITE_URL must be one HTTPS origin root with a trailing slash, received '$CI_INDEXNOW_SITE_URL'." >&2
    exit 1
  fi
  if [[ ! "$CI_INDEXNOW_ENDPOINT" =~ ^https://[^/]+/indexnow$ ]]; then
    echo "Refusing IndexNow operation: CI_INDEXNOW_ENDPOINT must be an explicit HTTPS /indexnow endpoint, received '$CI_INDEXNOW_ENDPOINT'." >&2
    exit 1
  fi
  if [[ ! "$CI_INDEXNOW_SITEMAP_PATH" =~ ^[A-Za-z0-9._/-]+\.xml$ || "$CI_INDEXNOW_SITEMAP_PATH" == /* || "$CI_INDEXNOW_SITEMAP_PATH" == *".."* ]]; then
    echo "Refusing IndexNow operation: CI_INDEXNOW_SITEMAP_PATH must be a safe relative XML path, received '$CI_INDEXNOW_SITEMAP_PATH'." >&2
    exit 1
  fi
  if [[ ! "$CI_INDEXNOW_KEY_FILE_NAME" =~ ^[A-Za-z0-9-]{8,128}\.txt$ ]]; then
    echo "Refusing IndexNow operation: CI_INDEXNOW_KEY_FILE_NAME must be <8-128 letter/digit/dash key>.txt, received '$CI_INDEXNOW_KEY_FILE_NAME'." >&2
    exit 1
  fi
  if [[ ! "$CI_INDEXNOW_TIMEOUT_SECONDS" =~ ^[1-9][0-9]*$ ]]; then
    echo "Refusing IndexNow operation: CI_INDEXNOW_TIMEOUT_SECONDS must be a positive integer, received '$CI_INDEXNOW_TIMEOUT_SECONDS'." >&2
    exit 1
  fi
  if [[ ! "$CI_INDEXNOW_REPORT_DIR_NAME" =~ ^[A-Za-z0-9._-]+$ ]]; then
    echo "Refusing IndexNow operation: CI_INDEXNOW_REPORT_DIR_NAME must be one safe directory name, received '$CI_INDEXNOW_REPORT_DIR_NAME'." >&2
    exit 1
  fi
}

ci_indexnow_require_local_config() {
  ci_indexnow_require_common_config
  ci_require_value CI_INDEXNOW_LOCAL_BIN

  if [[ ! -x "$CI_INDEXNOW_LOCAL_BIN" ]]; then
    echo "Refusing IndexNow dry run: local binary is missing or not executable at $CI_INDEXNOW_LOCAL_BIN. Build the shared IndexNow repository first." >&2
    exit 1
  fi
}

ci_indexnow_require_remote_config() {
  ci_indexnow_require_common_config
  ci_require_value CI_INDEXNOW_REMOTE_BIN

  if [[ ! "$CI_INDEXNOW_REMOTE_BIN" =~ ^/[A-Za-z0-9._/-]+$ ]]; then
    echo "Refusing IndexNow submission: CI_INDEXNOW_REMOTE_BIN must be one safe absolute path, received '$CI_INDEXNOW_REMOTE_BIN'." >&2
    exit 1
  fi
}
