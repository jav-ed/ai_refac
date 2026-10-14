#!/bin/bash
# =============================================================================
# Build Astro Site
# =============================================================================
set -e

SCRIPT_DIR="$(dirname "$0")"
source "$SCRIPT_DIR/../Lib/runtime.sh"
source "$SCRIPT_DIR/../Lib/colors.sh"

cd "$CI_PROJECT_DIR"

ci_header "Build Astro Site"
ci_info "Project: $(pwd)"
echo ""

bun run build

echo ""
ci_success "Build Complete"
