//! Kotlin and Android against the real JetBrains Kotlin language server (needs REFAC_KOTLIN_SERVER; Android also ANDROID_HOME).

#[allow(dead_code)]
#[path = "../common/mod.rs"]
mod common;

mod android;
mod dispatch;
mod dry_run;
mod moves;
mod rename;
mod server;
