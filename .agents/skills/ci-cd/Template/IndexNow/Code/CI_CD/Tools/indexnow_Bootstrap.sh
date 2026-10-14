#!/bin/bash
# Recover the one-time full-site submission after a key-bearing release is active.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../Lib/runtime.sh"
source "$SCRIPT_DIR/../Lib/colors.sh"
source "$SCRIPT_DIR/../Lib/indexnow.sh"

ci_require_value CI_DEPLOY_HOST
ci_require_value CI_DEPLOYMENT_ROOT
ci_require_value CI_DEPLOY_DOMAIN
ci_indexnow_require_remote_config
ci_indexnow_require_command ssh

DEPLOYMENT_ROOT="${CI_DEPLOYMENT_ROOT%/}"
LANE_DIR="$DEPLOYMENT_ROOT/$CI_DEPLOY_DOMAIN/main"
ACTIVE_LINK="$LANE_DIR/dist"
SSH=(ssh -o ClearAllForwardings=yes)

INDEXNOW_MODE="$(ci_indexnow_remote_mode "$CI_DEPLOY_HOST" "$LANE_DIR")"
if [[ "$INDEXNOW_MODE" != "bootstrap" ]]; then
  echo "Refusing IndexNow bootstrap: a valid bootstrap report already exists." >&2
  exit 1
fi

ACTIVE_TARGET="$("${SSH[@]}" "$CI_DEPLOY_HOST" "readlink '$ACTIVE_LINK'")"
CURRENT_ROOT="$(ci_indexnow_release_root "$LANE_DIR" "$ACTIVE_TARGET")"

ci_header "IndexNow Full-Site Bootstrap"
ci_info "Active: $CI_DEPLOY_HOST:$CURRENT_ROOT"
ci_info "Report: $CI_DEPLOY_HOST:$(ci_indexnow_bootstrap_report_path "$LANE_DIR")"

ci_indexnow_bootstrap_remote "$CI_DEPLOY_HOST" "$LANE_DIR" "$ACTIVE_LINK" "$ACTIVE_TARGET"

ci_success "IndexNow full-site bootstrap completed"
