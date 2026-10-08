# Rust symbol rename

`refac rename` on an `.rs` file renames a function, method, trait method, struct field, enum variant, type, constant, or local with rust-analyzer, across the files of the Cargo package or workspace, including `tests/`, `examples/`, and `use` trees.

```bash
refac rename --project-path /path/to/cargo/root --file src/shapes.rs \
  --symbol area --new-name surface
```

`--project-path` must hold `Cargo.toml`. The server is started with that folder as its working directory, because rustup picks the toolchain from the directory: the project's `rust-toolchain.toml` decides which `rust-analyzer` runs. If it is missing, rustup's stand-in in `~/.cargo/bin` fails with "Unknown binary 'rust-analyzer'", which the locator reports as a broken install; `refac doctor rust` prints `rustup component add rust-analyzer` (run it inside the project when the project pins a toolchain).

## What rust-analyzer does by itself

- **Follows the type system.** A trait method rename renames every `impl` and every call; a re-exported struct is renamed through `pub use` and `use` trees; an enum variant through every `match`.
- **Refuses what does not build.** A local renamed to a name that would capture another binding is accepted by the server; the engine's proof refuses it (usages gained or lost).
- **Renames the field and the local together** when a shorthand initialiser ties them (`Rect { width, height }`): the edit at the parameter lies outside the field's references, so it is a related group that the proof checks separately.

## What refac adds

- **Macro bodies.** rust-analyzer does not rename inside a `macro_rules!` definition. When the old name is still written there, the report starts with `ATTENTION: ... is still written inside a macro_rules! definition (src/report.rs:5) ... The program does not build until you change it yourself.` The test edits the macro by hand and then runs `cargo check` to show the project builds again.
- **Refuses a module rename.** Renaming a `mod` name makes the server answer with file moves. The command stops with a pointer to `refac move-module`, which moves the module subtree semantically.
- **Reports what is still written**, as for every language.

## Limits

- Code behind `cfg` flags that are off for the server's configuration is not analysed; the leftover note lists the lines that still spell the old name.
- Procedural-macro output and generated code follow what rust-analyzer expands.
- Cost: the server loads the Cargo workspace for every call (see [Language servers](../../Setup/language_Servers.md) for measured times).

## Tests

`tests/rename/rust.rs` (eight scenarios, `#[ignore]`d, fixture `tests/fixtures/rust/rename_crate`, package `shop`): a trait method with impls, callers and tests (and the macro attention note), a field with shorthand initialisers, a struct through use trees and re-exports, an enum variant in every match, a name shared by two locals, a capture (refused), a module rename (refused, points to `move-module`), and a dry run. Each successful scenario ends with `cargo check --all-targets --offline`. The tests copy the repository's `rust-toolchain.toml` into the temporary project so rustup picks a toolchain that has rust-analyzer. Run with `cargo test --test rename rust:: -- --ignored`.
