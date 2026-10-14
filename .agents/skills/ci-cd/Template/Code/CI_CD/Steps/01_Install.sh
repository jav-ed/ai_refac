#!/bin/bash
# =============================================================================
# Woodpecker CI - Install Dependencies
# =============================================================================
set -e

SCRIPT_DIR="$(dirname "$0")"
source "$SCRIPT_DIR/../Lib/runtime.sh"
source "$SCRIPT_DIR/../Lib/colors.sh"

cd "$CI_PROJECT_DIR"

ci_header "Install Dependencies"
ci_info "Project: $(pwd)"
ci_info "Bun: $(bun --version)"
echo ""

bun install

echo ""
ci_success "Install Complete"
