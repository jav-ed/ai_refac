#!/bin/bash
# =============================================================================
# Generate OG Images
# =============================================================================
set -e

SCRIPT_DIR="$(dirname "$0")"
source "$SCRIPT_DIR/../Lib/runtime.sh"
source "$SCRIPT_DIR/../Lib/colors.sh"

cd "$CI_PROJECT_DIR"

ci_header "Generate OG Images"
ci_info "Project: $(pwd)"
echo ""

cd "$CI_OG_IMAGE_DIR"
bun src/main.js --project_Root "$CI_PROJECT_DIR"

echo ""
ci_success "OG Image Generation Complete"
