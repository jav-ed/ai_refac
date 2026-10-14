#!/bin/bash

ci_indexnow_bootstrap_report_path() {
  local lane_dir="$1"

  printf '%s/%s/bootstrap.json\n' "$lane_dir" "$CI_INDEXNOW_REPORT_DIR_NAME"
}

# A bootstrap report is operational state, not merely a file-existence marker.
# Validate that it proves a successful full-site submission before allowing
# incremental comparisons.
ci_indexnow_remote_mode() {
  local deploy_host="$1"
  local lane_dir="$2"
  local report_path
  local temp_report
  local lock_path
  local ssh_command=(ssh -o ClearAllForwardings=yes)

  ci_indexnow_require_remote_config
  ci_indexnow_require_command ssh
  report_path="$(ci_indexnow_bootstrap_report_path "$lane_dir")"
  temp_report="$report_path.tmp"
  lock_path="$report_path.lock"

  "${ssh_command[@]}" "$deploy_host" bash -s -- \
    "$report_path" "$temp_report" "$lock_path" <<'REMOTE_INDEXNOW_STATE'
set -euo pipefail

report_path="$1"
temp_report="$2"
lock_path="$3"

if [[ -e "$temp_report" || -e "$lock_path" ]]; then
  echo "Refusing IndexNow state check: an incomplete report or lock exists beside $report_path." >&2
  exit 1
fi
if [[ ! -e "$report_path" ]]; then
  printf '%s\n' 'bootstrap'
  exit 0
fi
if [[ ! -f "$report_path" ]]; then
  echo "Refusing IndexNow state check: bootstrap report is not a regular file at $report_path." >&2
  exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "Refusing IndexNow state check: jq is unavailable on the deployment host." >&2
  exit 1
fi
if ! jq -e '
  (.comparison.added | type) == "number" and
  .comparison.added > 0 and
  .comparison.changed == 0 and
  .comparison.removed == 0 and
  .comparison.unchanged == 0 and
  .submission.public_key_verified == true and
  .submission.submitted_url_count == .comparison.added and
  (.submission.batches | type) == "array"
' "$report_path" >/dev/null; then
  echo "Refusing IndexNow state check: bootstrap report is malformed or does not prove a full-site submission: $report_path." >&2
  exit 1
fi

printf '%s\n' 'incremental'
REMOTE_INDEXNOW_STATE
}
