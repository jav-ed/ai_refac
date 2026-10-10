//! Kotlin and Android against the real JetBrains Kotlin language server (needs REFAC_KOTLIN_SERVER; Android also ANDROID_HOME).
//!
//! DO NOT run these tests casually, and never after every edit. A Kotlin server and its Gradle
//! import cost 20 to 47 seconds before the first request and about 2 GB of memory, and the whole
//! group takes 7 minutes. So by default only the QUICK SET runs, and it fits into 30 seconds:
//!   cargo test --test kotlin quick:: -- --ignored --test-threads=1
//! Every other test is BLOCKED unless the command says REFAC_KOTLIN_TESTS=all, which is a decision
//! (the change needs that test), made once per change, for the few tests that cover it, in ONE command:
//!   REFAC_KOTLIN_TESTS=all cargo test --test kotlin -- --ignored --test-threads=1 <module::test> <module::test>
//!   REFAC_KOTLIN_TESTS=all cargo test --test kotlin moves:: -- --ignored --test-threads=1
//! The tests that call the library (`moves::`, `rename::`, `android::`, `dispatch::`,
//! `multiplatform::`, and the move that `dry_run::` compares a plan with) share ONE server per
//! fixture inside a cargo command (tests/common/pool.rs): the first test pays the start, every later
//! one costs seconds. The tests that start a server of their own (`server::`, and the binary that
//! plans a dry run on a copy) are the long ones.
//! Which tests cover which change: Project_Manag/Docs/Setup/kotlin_Server.md

#[allow(dead_code)]
#[path = "../common/mod.rs"]
mod common;

mod android;
mod dispatch;
mod dry_run;
mod moves;
mod multiplatform;
mod quick;
mod rename;
mod server;
