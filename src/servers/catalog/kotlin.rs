use crate::servers::{Find, Requirement, Server};

/// Where a person or an agent reads how to install and replace the server. The
/// error of an expired build points here, so it is a link and not a path: refac
/// may run from a release archive, without the repository.
pub const SETUP_GUIDE: &str =
    "https://github.com/jav-ed/ai_refac/blob/main/Project_Manag/Docs/Setup/kotlin_Server.md";

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
        "curl -LO https://download.jetbrains.com/language-server/kotlin-server/263.7084.0/kotlin-server-263.7084.0.tar.gz",
        "echo \"ae0c4b01e17ab73a4a5227a78ea99e6dcbe3e499600315b22759a4e4cddd11df  kotlin-server-263.7084.0.tar.gz\" | sha256sum -c -",
        "mkdir -p ~/.local/share/refac && tar -xzf kotlin-server-263.7084.0.tar.gz -C ~/.local/share/refac",
        "export REFAC_KOTLIN_SERVER=~/.local/share/refac/kotlin-server-263.7084.0   (add it to the shell profile; there is no automatic download)",
        "Android projects also need the Android SDK: export ANDROID_HOME=/path/to/android-sdk",
        "This server is published as early-access builds that stop starting about four weeks after their release. When it says \"has expired\", or the build above has expired too, read how to replace it: https://github.com/jav-ed/ai_refac/blob/main/Project_Manag/Docs/Setup/kotlin_Server.md",
    ],
};

#[cfg(test)]
mod tests {
    use super::*;

    const BUILD: &str = "263.7084.0";
    const URL: &str = "https://download.jetbrains.com/language-server/kotlin-server/263.7084.0/kotlin-server-263.7084.0.tar.gz";
    const SHA256: &str = "ae0c4b01e17ab73a4a5227a78ea99e6dcbe3e499600315b22759a4e4cddd11df";

    #[test]
    fn the_install_steps_name_the_verified_build_download() {
        let steps = KOTLIN.install.join("\n");
        assert!(steps.contains(URL), "{steps}");
        assert!(steps.contains(SHA256), "{steps}");
        assert!(steps.contains(BUILD), "{steps}");
    }

    #[test]
    fn the_install_steps_point_to_the_guide_the_expiry_error_names() {
        assert!(KOTLIN.install.join("\n").contains(SETUP_GUIDE));
    }
}
