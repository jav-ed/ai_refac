#!/bin/bash
# =============================================================================
# CI Colors & Helpers - Shared Module
# =============================================================================
# Source this file in CI scripts: source "$(dirname "$0")/colors.sh"
# =============================================================================

export C_RESET='\033[0m'
export C_BRIGHT='\033[1m'

export C_RED='\033[1;91m'
export C_GREEN='\033[1;92m'
export C_YELLOW='\033[1;93m'
export C_BLUE='\033[1;94m'
export C_CYAN='\033[1;96m'

ci_header() {
    echo ""
    echo -e "${C_CYAN}────────────────────────────────────────${C_RESET}"
    echo -e "${C_CYAN}${C_BRIGHT}  $1${C_RESET}"
    echo -e "${C_CYAN}────────────────────────────────────────${C_RESET}"
    echo ""
}

ci_success() {
    echo ""
    echo -e "${C_GREEN}────────────────────────────────────────${C_RESET}"
    echo -e "${C_GREEN}${C_BRIGHT}  ✓ $1${C_RESET}"
    echo -e "${C_GREEN}────────────────────────────────────────${C_RESET}"
    echo ""
}

ci_warn()  { echo -e "${C_YELLOW}⚠ $1${C_RESET}"; }
ci_error() { echo -e "${C_RED}✗ $1${C_RESET}"; }
ci_info()  { echo -e "${C_BLUE}→ $1${C_RESET}"; }
