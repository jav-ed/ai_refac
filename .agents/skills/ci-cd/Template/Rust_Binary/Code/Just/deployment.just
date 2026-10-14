# Remote mutation stays explicit: install uses the existing artifact, while
# deploy proves and rebuilds the current source first.

[doc('Atomically install the existing release binary remotely and verify it.')]
[group('deployment')]
install:
    Code/CI_CD/Steps/03_Install.sh

[doc('Quality-check, compile, atomically install, and verify the current source.')]
[group('deployment')]
deploy:
    just quality
    just build
    just install

[doc('Print the configured local and installed binary versions.')]
[group('deployment')]
binary-status:
    @source Code/CI_CD/Config/env.env; test -x "target/release/$CI_BINARY_NAME" || { echo "Local release binary is missing at target/release/$CI_BINARY_NAME. Run just build first." >&2; exit 1; }; "target/release/$CI_BINARY_NAME" --version
    @source Code/CI_CD/Config/env.env; ssh -o ClearAllForwardings=yes "$CI_INSTALL_HOST" "'$CI_INSTALL_PATH' --version"
