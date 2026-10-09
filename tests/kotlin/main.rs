//! Kotlin and Android against the real JetBrains Kotlin language server (needs REFAC_KOTLIN_SERVER; Android also ANDROID_HOME).
//!
//! Every test starts its own server and Gradle import (1 to 2 minutes). Run one test, or a few
//! named tests in ONE command, one at a time; do not run the whole group as a routine:
//!   cargo test --test kotlin dispatch::a_kotlin_rename_is_routed_by_the_file_extension -- --ignored
//!   cargo test --test kotlin -- --ignored --test-threads=1 <module::test> <module::test>
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
