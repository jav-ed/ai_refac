# Release process

A release is a GitHub Release with one archive per tested target and a `SHA256SUMS` file. GitHub Actions builds it on free runners (the repository is public); nothing is built on a developer machine.

**Targets.** Linux x86_64 only: refac has not been run on any other system, so no other archive is built. To add one, run refac on that system first, then add a line to the `matrix` of `.github/workflows/release.yml`.

**Cutting a release.**

1. Raise `version` in `Cargo.toml` (and let `Cargo.lock` follow), commit, push, and wait until the `ci` workflow is green.
2. `git tag v<version>` and `git push origin v<version>`. The `release` workflow stops when the tag and the `Cargo.toml` version differ.
3. The workflow builds with `cargo build --release --locked`, packs with `scripts/package_release.sh`, starts the packed binary once to check its version, and publishes the release with generated notes.

**What the archive holds.** One folder `refac-<version>-<target>/` with the binary `refac`, `scripts/` (the TypeScript and Python helpers the binary starts; no tests, no `node_modules`), `pyrefly.toml`, `LICENSE`, and `README.md`. refac finds `scripts/` next to its executable, so the folder stays together. The helper packages are installed into `scripts/` on first use, which needs Bun, network, and a writable folder.

**Installing from an archive.**

```bash
mkdir -p ~/.local/share/refac ~/.local/bin
tar -xzf refac-<version>-linux-x86_64.tar.gz -C ~/.local/share/refac
ln -sfn ~/.local/share/refac/refac-<version>-linux-x86_64/refac ~/.local/bin/refac
refac --version
```

Check the download with `sha256sum -c SHA256SUMS`. An upgrade is the same commands with the new version, and the old folder can be deleted.

**Checked 2026-10-14** (locally, before the first workflow run): the archive from `scripts/package_release.sh` was extracted to another folder, the original checkout was hidden from the binary with a mount namespace, and `refac move --dry-run` on a TypeScript project still found `scripts/ts_refactor.ts` in the archive and planned the edit. The workflows themselves have not run on GitHub yet: read their first run.
