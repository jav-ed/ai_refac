#!/bin/bash
# Compare the exact active production release with local dist without submission.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
source "$SCRIPT_DIR/../Lib/runtime.sh"
source "$SCRIPT_DIR/../Lib/colors.sh"
source "$SCRIPT_DIR/../Lib/indexnow.sh"

ci_require_value CI_DEPLOY_HOST
ci_require_value CI_DEPLOYMENT_ROOT
ci_require_value CI_DEPLOY_DOMAIN
ci_indexnow_require_local_config
ci_indexnow_require_command jq
ci_indexnow_require_command rsync
ci_indexnow_require_command ssh

DEPLOYMENT_ROOT="${CI_DEPLOYMENT_ROOT%/}"
LANE_DIR="$DEPLOYMENT_ROOT/$CI_DEPLOY_DOMAIN/main"
ACTIVE_LINK="$LANE_DIR/dist"
SCRATCH_DIR="$CI_PROJECT_DIR/Scratch/IndexNow"
PREVIOUS_FIXTURE="$SCRATCH_DIR/Previous_Release"
REPORT_PATH="$SCRATCH_DIR/dry_Run_Report.json"
SSH=(ssh -o ClearAllForwardings=yes)

if [[ ! -d "$CI_PROJECT_DIR/dist" ]]; then
  echo "Refusing IndexNow dry run: local dist/ is missing. Run just Step_02_Build and just Step_04_Compress first." >&2
  exit 1
fi

ci_indexnow_verify_local_key "$CI_PROJECT_DIR/dist"
ACTIVE_TARGET="$("${SSH[@]}" "$CI_DEPLOY_HOST" "readlink '$ACTIVE_LINK'")"
PREVIOUS_ROOT="$(ci_indexnow_release_root "$LANE_DIR" "$ACTIVE_TARGET")"
INDEXNOW_MODE="$(ci_indexnow_remote_mode "$CI_DEPLOY_HOST" "$LANE_DIR")"

ci_header "IndexNow Dry Run"
ci_info "Mode:     $INDEXNOW_MODE"
ci_info "Current:  $CI_PROJECT_DIR/dist"
ci_info "Report:   $REPORT_PATH"

rm -rf "$PREVIOUS_FIXTURE"
mkdir -p "$PREVIOUS_FIXTURE"

if [[ "$INDEXNOW_MODE" == "bootstrap" ]]; then
  ci_info "Previous: empty sitemap (no valid bootstrap report exists)"
  ci_indexnow_create_empty_release "$PREVIOUS_FIXTURE" "$CI_INDEXNOW_SITEMAP_PATH"
else
  ci_info "Previous: $CI_DEPLOY_HOST:$PREVIOUS_ROOT"
  RSYNC_RSH="ssh -o ClearAllForwardings=yes" rsync -az --delete --prune-empty-dirs \
    --include='*/' \
    --include='*.html' \
    --include='*.xml' \
    --exclude='*' \
    "$CI_DEPLOY_HOST:$PREVIOUS_ROOT/" \
    "$PREVIOUS_FIXTURE/"
fi

"$CI_INDEXNOW_LOCAL_BIN" compare-releases \
  --site-url "$CI_INDEXNOW_SITE_URL" \
  --sitemap "$CI_INDEXNOW_SITEMAP_PATH" \
  --previous-root "$PREVIOUS_FIXTURE" \
  --current-root "$CI_PROJECT_DIR/dist" \
  --report "$REPORT_PATH"

jq '{
  added,
  changed,
  removed,
  unchanged,
  submission_count: (.submission_urls | length),
  submission_urls
}' "$REPORT_PATH"

ci_success "IndexNow dry run completed without contacting IndexNow"
