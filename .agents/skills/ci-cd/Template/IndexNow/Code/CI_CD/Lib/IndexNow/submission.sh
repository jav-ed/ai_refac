#!/bin/bash

ci_indexnow_print_report() {
  local deploy_host="$1"
  local report_path="$2"
  local ssh_command=(ssh -o ClearAllForwardings=yes)

  "${ssh_command[@]}" "$deploy_host" "cat '$report_path'" | jq '{
    added: .comparison.added,
    changed: .comparison.changed,
    removed: .comparison.removed,
    unchanged: .comparison.unchanged,
    submitted: .submission.submitted_url_count,
    batches: (.submission.batches | length),
    public_key_verified: .submission.public_key_verified
  }'
}

ci_indexnow_run_remote_submission() {
  local mode="$1"
  local deploy_host="$2"
  local lane_dir="$3"
  local active_link="$4"
  local previous_root="$5"
  local current_root="$6"
  local current_target="$7"
  local report_path="$8"
  local remote_worker="$CI_INDEXNOW_LIB_DIR/remote_Submission.sh"
  local ssh_command=(ssh -o ClearAllForwardings=yes)

  ci_indexnow_require_remote_config
  ci_indexnow_require_command jq
  ci_indexnow_require_command ssh

  if [[ ! -f "$remote_worker" ]]; then
    echo "Refusing IndexNow submission: remote worker is missing at $remote_worker." >&2
    exit 1
  fi

  "${ssh_command[@]}" "$deploy_host" bash -s -- \
    "$mode" "$CI_INDEXNOW_REMOTE_BIN" "$CI_INDEXNOW_SITE_URL" \
    "$CI_INDEXNOW_SITEMAP_PATH" "$CI_INDEXNOW_KEY_FILE_NAME" \
    "$CI_INDEXNOW_ENDPOINT" "$CI_INDEXNOW_TIMEOUT_SECONDS" "$active_link" \
    "$current_target" "$previous_root" "$current_root" "$lane_dir" \
    "$report_path" "${CI_INDEXNOW_EXTENSIONLESS_URLS:-false}" < "$remote_worker"

  ci_indexnow_print_report "$deploy_host" "$report_path"
  ci_info "IndexNow report: ${deploy_host}:${report_path}"
}

ci_indexnow_submit_remote() {
  local deploy_host="$1"
  local lane_dir="$2"
  local active_link="$3"
  local previous_target="$4"
  local current_target="$5"
  local release_name="$6"
  local previous_root
  local current_root
  local report_path="$lane_dir/$CI_INDEXNOW_REPORT_DIR_NAME/$release_name.json"

  previous_root="$(ci_indexnow_release_root "$lane_dir" "$previous_target")"
  current_root="$(ci_indexnow_release_root "$lane_dir" "$current_target")"
  ci_info "Submitting changed URLs through IndexNow ..."
  ci_indexnow_run_remote_submission \
    incremental "$deploy_host" "$lane_dir" "$active_link" "$previous_root" \
    "$current_root" "$current_target" "$report_path"
}

ci_indexnow_bootstrap_remote() {
  local deploy_host="$1"
  local lane_dir="$2"
  local active_link="$3"
  local current_target="$4"
  local current_root
  local report_path

  current_root="$(ci_indexnow_release_root "$lane_dir" "$current_target")"
  report_path="$(ci_indexnow_bootstrap_report_path "$lane_dir")"
  ci_info "Submitting the complete active sitemap through IndexNow ..."
  # SSH reconstructs a remote command string and drops an empty positional
  # argument. Pass the current root in slot 10; the bootstrap worker replaces
  # it with the generated empty-sitemap fixture before invoking IndexNow.
  ci_indexnow_run_remote_submission \
    bootstrap "$deploy_host" "$lane_dir" "$active_link" "$current_root" "$current_root" \
    "$current_target" "$report_path"
}

ci_indexnow_notify_remote() {
  local deploy_host="$1"
  local lane_dir="$2"
  local active_link="$3"
  local previous_target="$4"
  local current_target="$5"
  local release_name="$6"
  local mode

  mode="$(ci_indexnow_remote_mode "$deploy_host" "$lane_dir")"
  ci_info "IndexNow mode: $mode"

  if [[ "$mode" == "bootstrap" ]]; then
    ci_indexnow_bootstrap_remote "$deploy_host" "$lane_dir" "$active_link" "$current_target"
    return
  fi

  ci_indexnow_submit_remote \
    "$deploy_host" "$lane_dir" "$active_link" "$previous_target" \
    "$current_target" "$release_name"
}
