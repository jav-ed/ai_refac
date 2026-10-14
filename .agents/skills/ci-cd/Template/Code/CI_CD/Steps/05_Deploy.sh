#!/bin/bash
# =============================================================================
# Woodpecker CI - Deploy static site
# =============================================================================
# rsync dist/ to the configured deploy host, then perform an atomic symlink swap.
#
# Structure on the deploy host:
#   <deployment-root>/zetunweb.com/
#   ├── main/
#   │   ├── releases/<date>_<hash>/      ← production deployments
#   │   └── dist -> releases/<date>_...  ← Caddy serves zetunweb.com
#   └── shared/                      ← persistent data across deploys
#
# Keeps the 5 most recent releases for rollback.
# =============================================================================
set -e

SCRIPT_DIR="$(dirname "$0")"
source "$SCRIPT_DIR/../Lib/runtime.sh"
source "$SCRIPT_DIR/../Lib/colors.sh"

cd "$CI_PROJECT_DIR"

ci_header "Deploy Static Site"
ci_info "Project: $(pwd)"

# --- SSH without audio forwarding (suppresses RemoteForward 4713 warning) ---
SSH="ssh -o ClearAllForwardings=yes"
export RSYNC_RSH="$SSH"

# --- Config ---
ci_require_value CI_DEPLOY_HOST
ci_require_value CI_DEPLOYMENT_ROOT
ci_require_value CI_DEPLOY_DOMAIN

DEPLOY_HOST="$CI_DEPLOY_HOST"
DEPLOYMENT_ROOT="${CI_DEPLOYMENT_ROOT%/}"
DEPLOY_DOMAIN="$CI_DEPLOY_DOMAIN"
KEEP_RELEASES="${CI_DEPLOY_KEEP_RELEASES:-5}"

# --- Sanity checks ---
# Refuse broad or non-runtime roots before deriving the domain-specific site path.
case "$DEPLOYMENT_ROOT" in
  ""|"/")
    echo "Refusing deploy: CI_DEPLOYMENT_ROOT is too broad: ${DEPLOYMENT_ROOT}" >&2
    exit 1
    ;;
esac

if [[ "$DEPLOYMENT_ROOT" != *"/Web_Layer/20_Static_Websites/"* ]]; then
  echo "Refusing deploy: CI_DEPLOYMENT_ROOT must point into Web_Layer/20_Static_Websites: ${DEPLOYMENT_ROOT}" >&2
  exit 1
fi

case "$DEPLOY_DOMAIN" in
  ""|"."|".."|*/*)
    echo "Refusing deploy: CI_DEPLOY_DOMAIN must be one domain folder name, got: ${DEPLOY_DOMAIN}" >&2
    exit 1
    ;;
esac

if [[ "$DEPLOY_DOMAIN" == *"'"* ]]; then
  echo "Refusing deploy: CI_DEPLOY_DOMAIN must not contain single quotes: ${DEPLOY_DOMAIN}" >&2
  exit 1
fi

# --- Branch-specific paths ---
BRANCH="${CI_COMMIT_BRANCH:-main}"

case "$BRANCH" in
  main)
    DEPLOY_ENV_DIR="main"
    ;;
  test)
    if [[ "${CI_DEPLOY_ENABLE_TEST:-0}" != "1" ]]; then
      echo "Refusing deploy: test branch deploy is not enabled for this project." >&2
      exit 1
    fi
    DEPLOY_ENV_DIR="test"
    ;;
  *)
    echo "Refusing deploy: branch must be main, got '${BRANCH}'." >&2
    exit 1
    ;;
esac

SITE_DIR="${DEPLOYMENT_ROOT}/${DEPLOY_DOMAIN}"
LANE_DIR="${SITE_DIR}/${DEPLOY_ENV_DIR}"
RELEASES_DIR="${LANE_DIR}/releases"
ACTIVE_LINK="${LANE_DIR}/dist"

# --- Release name: date + short git hash ---
COMMIT_SHA="${CI_COMMIT_SHA:-$(git rev-parse HEAD)}"
SHORT_SHA="${COMMIT_SHA:0:8}"
PIPELINE_NUMBER="${CI_PIPELINE_NUMBER:-local}"
RELEASE_NAME="$(date +%Y-%m-%d)_p${PIPELINE_NUMBER}_${SHORT_SHA}"
RELEASE_DIR="${RELEASES_DIR}/${RELEASE_NAME}"
ACTIVE_TARGET="releases/${RELEASE_NAME}"

ci_info "Site:    ${DEPLOY_DOMAIN}"
ci_info "Lane:    ${DEPLOY_ENV_DIR}"
ci_info "Release: ${RELEASE_NAME}"
ci_info "Target:  ${DEPLOY_HOST}:${RELEASE_DIR}"
ci_info "Active:  ${DEPLOY_HOST}:${ACTIVE_LINK} -> ${ACTIVE_TARGET}"
echo ""

if $SSH "$DEPLOY_HOST" "test -e '$RELEASE_DIR'"; then
  echo "Refusing deploy: release directory already exists: ${DEPLOY_HOST}:${RELEASE_DIR}" >&2
  exit 1
fi

if $SSH "$DEPLOY_HOST" "test -e '$ACTIVE_LINK' && ! test -L '$ACTIVE_LINK'"; then
  echo "Refusing deploy: active dist path exists but is not a symlink: ${DEPLOY_HOST}:${ACTIVE_LINK}" >&2
  exit 1
fi

# --- Create release directory ---
$SSH "$DEPLOY_HOST" "mkdir -p '$RELEASES_DIR' && mkdir '$RELEASE_DIR'"

# --- rsync dist/ to release directory ---
ci_info "Syncing dist/ ..."
rsync -az --delete dist/ "${DEPLOY_HOST}:${RELEASE_DIR}/"

# --- Atomic symlink swap ---
ci_info "Swapping symlink ..."
$SSH "$DEPLOY_HOST" "ln -sfn '$ACTIVE_TARGET' '$ACTIVE_LINK'"

# --- Cleanup old releases (keep N most recent) ---
ci_info "Cleaning old releases (keeping ${KEEP_RELEASES}) ..."
$SSH "$DEPLOY_HOST" "cd '$RELEASES_DIR' && ls -1dt */ | tail -n +$((KEEP_RELEASES + 1)) | xargs -r rm -rf"

ci_success "Deployed ${RELEASE_NAME}"
