//! `refac --help`: the whole tool on one page.

pub(in crate::cli) const ABOUT: &str =
    "Move or rename files and symbols and update every reference, in one command.";

pub(in crate::cli) const LONG_ABOUT: &str = r#"Move or rename files and symbols and update every reference, in one command.

COMMANDS
  move         Move or rename files and folders. Imports, module declarations, package lines
               and Markdown links that pointed at them are rewritten.
  move-module  Move a whole Rust module subtree: its files and every `crate::` path to it.
  rename       Rename a symbol (variable, parameter, function, type, field, method) and every
               reference to it. `--batch` runs several renames in one language-server session.
  doctor       Check the language servers refac starts, and print how to install a missing one.
  guide        In-depth documentation by topic (languages, safety, batching, output, servers).
  completions  Print shell completions.   man  Print the manual page.

WHAT EACH LANGUAGE SUPPORTS
  Language         Move file  Move folder  Rust module  Rename symbol  Done by
  TypeScript / JS  yes        yes          -            yes            Oxc parser + TypeScript native language server
  Kotlin / Android yes        yes          -            yes            JetBrains Kotlin language server + refac's Android XML layer
  Go               yes        no           -            yes            gopls
  Rust             yes        no           yes          yes            rust-analyzer
  Python           yes        no           -            yes            Rope (moves), basedpyright (rename)
  Dart             yes        no           -            yes            Dart SDK analysis server
  Markdown         yes        yes          -            -              built in (CommonMark); also follows what other languages moved
  The language of a file comes from its extension. A single `move` may mix languages.

HOW IT STAYS SAFE
  * Plan first. Nothing is written until the whole change is planned. A rename is also proven in
    memory: the server is shown the renamed text and must find exactly the same places again, so a
    name that clashes with or shadows something is refused with the places that would change.
  * Undo log. Every write goes through a journal; if a step fails, every file is put back. Messages
    say "Nothing was changed" or, in the rare failed undo, which files to inspect with git.
  * Nothing stays running. A language server is started for the command and stopped when it ends
    (for Kotlin that includes the Gradle daemon its build import starts). There is nothing to
    stop, and no memory held afterwards.
  * Loud failures. A missing server, an unsupported project layout or an ambiguous symbol stops
    the command with the reason and the fix; refac never guesses.

WHAT A COMMAND COSTS
  A server starts for the command and stops after it. Time to start / memory held while it runs:
  Go 2-8 s / 165 MB, Python 1-2 s / 160 MB, Dart under 1 s / 125 MB, Rust 5-35 s / 640 MB and more
  (grows with the project), Kotlin about 24 s / 1.3-1.8 GB. So put several moves into one `move`
  call and several renames of one project into one `rename --batch` call. `refac guide batching`;
  for Kotlin, which refuses a single change by default, `refac guide kotlin` says why and what to do.

READING THE OUTPUT
  Lines starting with `//` are refac's report. `// Note:` lines name places refac did NOT change
  and you may need to: names inside strings and comments, calls on untyped Python/Dart receivers,
  Rust `macro_rules!` bodies (marked ATTENTION: the build breaks until you edit them), ProGuard
  and build scripts. Each note ends with the `rg -w` command that finds the rest.
  With --json every command prints one JSON document on stdout; an error prints
  {"status":"error","error":"..."} on stderr instead.

EXIT CODES
  0  everything succeeded      1  something failed (the reason is on stderr; see "Nothing was changed")
  2  the command line itself was wrong (clap prints the usage)

ENVIRONMENT
  REFAC_PROJECT_PATH      default for --project-path
  REFAC_GOPLS, REFAC_RUST_ANALYZER, REFAC_PYTHON_SERVER, REFAC_DART, REFAC_KOTLIN_SERVER
                          where a language server is installed (`refac doctor <language>` explains)
  REFAC_LSP_TIMEOUT_SECS  how long a server may take to load or answer (default 300)
  REFAC_KOTLIN_TIMEOUT_SECS  same for the Kotlin Gradle import (default 600)
  REFAC_KOTLIN_CACHE      where the Kotlin server keeps its index between calls (a directory or `off`;
                          default ~/.cache/refac)
  REFAC_KOTLIN_MIRROR_VERSION  Kotlin version of the plain-JVM copy used for a Kotlin Multiplatform build,
                          when gradle/libs.versions.toml and the build scripts do not state one
  REFAC_KOTLIN_BATCH_ONLY  a single Kotlin move or rename is refused by default (each pays the 24 s
                          server start); --allow-single lets one through, =0 turns the refusal off
  REFAC_KOTLIN_GRADLE_IDLE_SECS  keep the Kotlin project's Gradle daemon up that many seconds (1 to
                          3600) after a command; off by default (it stops 10 s after the import)
  REFAC_LSP_TRACE=1       print every message exchanged with a server (debugging)
  RUST_LOG=debug          more of refac's own log on stderr"#;

pub(in crate::cli) const AFTER_LONG_HELP: &str = r#"EXAMPLES
  # Move a file; every import of it is rewritten
  refac move --project-path /my/project --source-path src/old/name.ts --target-path src/new/name.ts

  # Move several files in one call (paired 1:1, may mix languages)
  refac move --project-path /my/project \
    --source-path src/a.ts --source-path docs/a.md \
    --target-path src/x.ts --target-path docs/x.md

  # Rename a symbol and every reference (the extension picks the language)
  refac rename --project-path /my/module --file shape/shape.go --symbol Area --new-name Surface

  # Several renames of one project in ONE server session, all or nothing
  refac rename --project-path /my/crate --batch renames.json

  # Preview anything that renames: plan and prove it, write nothing
  refac rename --project-path /my/crate --file src/lib.rs --symbol old --new-name new --dry-run

  # Move a Rust module subtree
  refac move-module --project-path /my/workspace crate::engine::matching crate::domain::matching

  # A server is missing: what refac looked for and how to install it
  refac doctor            # every language at a glance
  refac doctor go         # install steps and a start check for one

THREE DEPTHS OF HELP
  refac <command> -h       a few lines: what the command is and its flags
  refac <command> --help   everything about one command: arguments, rules, output, examples
  refac guide [topic]      in-depth topics that cut across commands (languages, safety, batching,
                           output, servers), printed from inside the binary
  In the repository: README.md, and Project_Manag/Docs/doc_Start.md."#;
