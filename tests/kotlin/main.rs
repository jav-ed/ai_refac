//! Kotlin and Android against the real JetBrains Kotlin language server (needs REFAC_KOTLIN_SERVER; Android also ANDROID_HOME).
//!
//! A Kotlin server and its Gradle import cost 20 to 47 seconds, so the tests that call the library
//! (`moves::`, `rename::`, `android::`, `dispatch::`, `multiplatform::`, and the move that
//! `dry_run::` compares a plan with) share ONE server per fixture inside a cargo command
//! (tests/common/pool.rs): the first test pays the start, every later one costs seconds.
//! Put the tests you want into ONE command, a single test or several:
//!   cargo test --test kotlin moves::a_file_moves_to_a_new_package_and_every_reference_follows -- --ignored
//!   cargo test --test kotlin -- --ignored --test-threads=1 <module::test> <module::test>
//!   cargo test --test kotlin moves:: -- --ignored --test-threads=1
//! The whole group takes about 7 minutes; the tests that start a server of their own (`server::`,
//! and the binary that plans a dry run on a copy) are the long ones.
//! Which tests cover which change: Project_Manag/Docs/Setup/kotlin_Server.md

#[allow(dead_code)]
#[path = "../common/mod.rs"]
mod common;

mod android;
mod dispatch;
mod dry_run;
mod moves;
mod multiplatform;
mod rename;
mod server;
