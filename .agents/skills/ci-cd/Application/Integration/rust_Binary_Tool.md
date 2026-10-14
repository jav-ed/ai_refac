# Rust Binary Tool Integration

Use this path for a Rust repository whose `main` push should quality-check, compile, and atomically install one executable on a remote host. The workflow was proven with IndexNow through a real Forgejo webhook, Woodpecker local-backend run, and g12 installation.

## Contents

- [Ownership contract](#ownership-contract)
- [Copy the template](#copy-the-rust-template)
- [Configure variables](#configure-explicit-variables)
- [Test locally](#test-locally)
- [Activate and prove the trigger](#activate-and-prove-the-trigger)
- [Failure policy](#failure-policy)

## Ownership Contract

- `.woodpecker.yml` owns only the real checkout path and ordered step calls.
- `Code/CI_CD/Config/env.env` owns project-specific toolchain, binary, and installation values.
- `Code/CI_CD/Lib/runtime.sh` validates and establishes the non-interactive Rust environment.
- `Code/CI_CD/Steps/` owns quality, build, and remote mutation.
- The root `justfile` and `Code/Just/` expose documented local commands but never become a Woodpecker dependency.

## Copy The Rust Template

From the new Rust repository root:

```bash
TEMPLATE="<path-to-ci-cd-skill>/Template/Rust_Binary"

cp "$TEMPLATE/.woodpecker.yml" .
cp "$TEMPLATE/justfile" .
mkdir -p Code
cp -r "$TEMPLATE/Code/CI_CD" Code/
cp -r "$TEMPLATE/Code/Just" Code/
chmod +x Code/CI_CD/Steps/*.sh
```

Every `__SET_*__` placeholder is intentionally invalid. Do not weaken the runtime checks or add defaults to make an incomplete copy run.

## Configure Explicit Variables

Set the real checkout in `.woodpecker.yml`:

```yaml
CI_REPO_PATH: /absolute/path/to/repository
```

Set every project value in `Code/CI_CD/Config/env.env`:

```bash
CI_RUST_TOOLCHAIN="1.89.0"
CI_RUSTUP_BIN="/absolute/path/to/rustup"
CI_RUSTUP_HOME="/absolute/path/to/.rustup"
CI_CARGO_HOME="/absolute/path/to/.cargo"
CI_BINARY_NAME="example-tool"
CI_INSTALL_HOST="ssh-alias"
CI_INSTALL_PATH="/absolute/remote/path/example-tool"
```

Use an exact installed toolchain. `stable` is not reproducible, and downloading an unreviewed toolchain during deployment hides host drift. The install-path basename must equal `CI_BINARY_NAME`.

Set the reviewed Forgejo URL in `Code/Just/forgejo.just`. `just publish` refuses a dirty tree, a non-`main` branch, or a different origin and never commits automatically.

## Test Locally

First inspect the command surface:

```bash
just --list
```

Then reproduce the local-backend boundary instead of testing only through an interactive shell:

```bash
env -i \
  HOME=/tmp/project-woodpecker-home \
  PATH=/usr/bin:/bin \
  Code/CI_CD/Steps/01_Quality.sh

env -i \
  HOME=/tmp/project-woodpecker-home \
  PATH=/usr/bin:/bin \
  Code/CI_CD/Steps/02_Build.sh
```

Both commands must print the configured Rust and Cargo versions. A Rustup “no default is configured” error means the scripts still depend on interactive state.

Run the complete manual remote path only after confirming the destination:

```bash
just deploy
just binary-status
```

`just deploy` runs quality, build, atomic installation, and installed-binary verification. `just install` intentionally installs an already-built artifact and is the narrower mutation command.

## Activate And Prove The Trigger

Commit the integrated files, push the repository, then use [Trigger checks](../Trigger/repo_Trigger_Checks.md). A newly created Forgejo repository commonly has no Woodpecker repo state or webhook until activation.

After activation, make one meaningful committed `main` push and inspect the actual pipeline result. Do not treat a successful Git push as proof of CI success. The first pipeline must show all three steps succeeding and the remote binary version check completing.

Woodpecker step log data may be base64 encoded in the API response. Decode it before diagnosing an immediate step failure. The tested failure signature was an implicit Rustup default missing under the local-backend HOME; the explicit variables above removed that hidden dependency.

## Failure Policy

- Missing or placeholder variables fail before build or installation.
- Missing configured directories and toolchains fail without substitution.
- Quality failure skips build and installation.
- Build failure leaves the installed binary unchanged.
- Installation uses a temporary remote path followed by an atomic `mv`.
- Forgejo publication and direct remote installation remain separate explicit commands.
