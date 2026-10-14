#!/bin/bash
# Executed on the deployment host through stdin by submission.sh.
set -euo pipefail

mode="$1"
indexnow_bin="$2"
site_url="$3"
sitemap_path="$4"
key_file_name="$5"
endpoint="$6"
timeout_seconds="$7"
active_link="$8"
current_target="$9"
previous_root="${10}"
current_root="${11}"
lane_dir="${12}"
report_path="${13}"
temp_report="$report_path.tmp"
lock_path="$report_path.lock"
bootstrap_root=""

if [[ "$mode" != "bootstrap" && "$mode" != "incremental" ]]; then
  echo "Refusing IndexNow submission: unsupported mode '$mode'." >&2
  exit 1
fi
if [[ ! -x "$indexnow_bin" ]]; then
  echo "Refusing IndexNow submission: remote binary is missing or not executable at $indexnow_bin." >&2
  exit 1
fi
if ! command -v jq >/dev/null 2>&1; then
  echo "Refusing IndexNow submission: jq is unavailable on the deployment host." >&2
  exit 1
fi
if [[ "$(readlink "$active_link")" != "$current_target" ]]; then
  echo "Refusing IndexNow submission: active release changed; expected $current_target." >&2
  exit 1
fi
if [[ ! -d "$current_root" ]]; then
  echo "Refusing IndexNow submission: current release root is missing at $current_root." >&2
  exit 1
fi
if [[ "$mode" == "incremental" && ! -d "$previous_root" ]]; then
  echo "Refusing IndexNow submission: previous release root is missing at $previous_root." >&2
  exit 1
fi

mkdir -p "$(dirname "$report_path")"
if [[ -e "$report_path" || -e "$temp_report" ]]; then
  echo "Refusing IndexNow submission: report path already exists or is incomplete: $report_path." >&2
  exit 1
fi

# mkdir is the atomic boundary shared by automatic deployment and the manual
# recovery command. Only one process may cross it and contact IndexNow.
if ! mkdir "$lock_path" 2>/dev/null; then
  echo "Refusing IndexNow submission: another process owns the report lock at $lock_path." >&2
  exit 1
fi

ci_indexnow_remote_cleanup() {
  rm -f "$temp_report"
  if [[ -n "$bootstrap_root" ]]; then
    rm -rf -- "$bootstrap_root"
  fi
  rmdir "$lock_path" 2>/dev/null || true
}
trap ci_indexnow_remote_cleanup EXIT

if [[ "$mode" == "bootstrap" ]]; then
  bootstrap_root="$(mktemp -d "$lane_dir/.indexnow-bootstrap.XXXXXX")"
  mkdir -p "$bootstrap_root/$(dirname "$sitemap_path")"
  printf '%s\n' \
    '<?xml version="1.0" encoding="UTF-8"?>' \
    '<urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9"></urlset>' \
    > "$bootstrap_root/$sitemap_path"
  previous_root="$bootstrap_root"
fi

"$indexnow_bin" submit-releases \
  --site-url "$site_url" \
  --sitemap "$sitemap_path" \
  --previous-root "$previous_root" \
  --current-root "$current_root" \
  --key-file "$current_root/$key_file_name" \
  --endpoint "$endpoint" \
  --timeout-seconds "$timeout_seconds" \
  --report "$temp_report"

# Do not finalize evidence for a release that stopped being active while the
# network request was running. The request remains visible as a CI failure.
if [[ "$(readlink "$active_link")" != "$current_target" ]]; then
  echo "Refusing IndexNow report: active release changed during submission." >&2
  exit 1
fi

if ! jq -e '
  .submission.submitted_url_count == (
    .comparison.added + .comparison.changed + .comparison.removed
  ) and
  (.submission.batches | type) == "array" and
  (
    (
      .submission.submitted_url_count == 0 and
      .submission.public_key_verified == false and
      (.submission.batches | length) == 0
    ) or
    (
      .submission.submitted_url_count > 0 and
      .submission.public_key_verified == true and
      (.submission.batches | length) > 0
    )
  )
' "$temp_report" >/dev/null; then
  echo "Refusing IndexNow report: submission report is malformed or contradictory." >&2
  exit 1
fi

if [[ "$mode" == "bootstrap" ]] && ! jq -e '
  .comparison.added > 0 and
  .comparison.changed == 0 and
  .comparison.removed == 0 and
  .comparison.unchanged == 0 and
  .submission.submitted_url_count == .comparison.added
' "$temp_report" >/dev/null; then
  echo "Refusing IndexNow bootstrap report: report does not prove a full-site submission." >&2
  exit 1
fi

mv "$temp_report" "$report_path"
