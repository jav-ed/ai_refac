# Quality recipes call the same scripts as Woodpecker to prevent contract drift.

[doc('Run formatting checks, all tests, and Clippy with warnings denied.')]
[group('quality')]
quality:
    Code/CI_CD/Steps/01_Quality.sh

[doc('Run the complete local CI gate: quality checks followed by a release build.')]
[group('quality')]
check:
    just quality
    just build
