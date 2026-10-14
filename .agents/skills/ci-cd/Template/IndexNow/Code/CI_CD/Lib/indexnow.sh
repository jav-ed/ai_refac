#!/bin/bash

# This stable entrypoint keeps deploy and operator scripts independent from the
# feature's internal file split. Add new IndexNow responsibilities below it.
CI_INDEXNOW_LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/IndexNow" && pwd)"

source "$CI_INDEXNOW_LIB_DIR/config.sh"
source "$CI_INDEXNOW_LIB_DIR/release.sh"
source "$CI_INDEXNOW_LIB_DIR/fixture.sh"
source "$CI_INDEXNOW_LIB_DIR/state.sh"
source "$CI_INDEXNOW_LIB_DIR/submission.sh"
