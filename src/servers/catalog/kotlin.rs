use crate::servers::{Find, Requirement, Server};

/// The install steps name the build the protocol of
/// `drivers/kotlin/server.rs` was verified against.
pub static KOTLIN: Server = Server {
    languages: &["kotlin", "kt", "android"],
    language: "Kotlin",
    name: "the JetBrains Kotlin language server",
    used_for: "refac move and refac rename on .kt files (Kotlin, Android)",
    env_var: "REFAC_KOTLIN_SERVER",
    find: Find::Folder {
        executable: "bin/intellij-server",
        build_file: "build.txt",
    },
    requirements: &[Requirement {
        name: "JDK 17 or newer",
        executable: "java",
        why: "the server imports the Gradle build of the project with it",
        hint: "install a JDK, for example from https://adoptium.net",
    }],
    install: &[
        "curl -LO https://download.jetbrains.com/language-server/kotlin-server/263.6379.0/kotlin-server-263.6379.0.tar.gz",
        "echo \"ab8ca4455dc2fc5fe1a24db2bccc46c104254d2c465155c4251ee65df8f3f7cc  kotlin-server-263.6379.0.tar.gz\" | sha256sum -c -",
        "mkdir -p ~/.local/share/refac && tar -xzf kotlin-server-263.6379.0.tar.gz -C ~/.local/share/refac",
        "export REFAC_KOTLIN_SERVER=~/.local/share/refac/kotlin-server-263.6379.0   (add it to the shell profile; there is no automatic download)",
        "Android projects also need the Android SDK: export ANDROID_HOME=/path/to/android-sdk",
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    const BUILD: &str = "263.6379.0";
    const URL: &str = "https://download.jetbrains.com/language-server/kotlin-server/263.6379.0/kotlin-server-263.6379.0.tar.gz";
    const SHA256: &str = "ab8ca4455dc2fc5fe1a24db2bccc46c104254d2c465155c4251ee65df8f3f7cc";

    #[test]
    fn the_install_steps_name_the_verified_build_download() {
        let steps = KOTLIN.install.join("\n");
        assert!(steps.contains(URL), "{steps}");
        assert!(steps.contains(SHA256), "{steps}");
        assert!(steps.contains(BUILD), "{steps}");
    }
}
