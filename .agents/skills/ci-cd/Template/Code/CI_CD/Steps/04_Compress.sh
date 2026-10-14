#!/bin/bash
# =============================================================================
# Woodpecker CI - Compress Static Assets
# =============================================================================
set -e

SCRIPT_DIR="$(dirname "$0")"
source "$SCRIPT_DIR/../Lib/runtime.sh"
source "$SCRIPT_DIR/../Lib/colors.sh"

cd "$CI_PROJECT_DIR"

ci_header "Compress Static Assets"
ci_info "Project: $(pwd)"
ci_info "Output: $(pwd)/dist"
echo ""

"$CI_COMPRESSOR_BIN" --dir dist --cache

echo ""
ci_success "Compression Complete"
