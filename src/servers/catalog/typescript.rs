use crate::servers::{Attempt, Find, Outcome, Requirement, Server};

pub static TYPESCRIPT: Server = Server {
    languages: &["typescript", "ts", "javascript", "js"],
    language: "TypeScript / JavaScript",
    name: "the TypeScript 7 native engine",
    used_for: "refac move and refac rename on TypeScript and JavaScript files",
    env_var: "",
    find: Find::Bundled { check },
    requirements: &[Requirement {
        name: "Bun",
        executable: "bun",
        why: "refac installs its locked TypeScript packages with it on first use",
        hint: "install Bun from https://bun.sh",
    }],
    install: &[
        "refac installs the engine itself on first use (bun install --frozen-lockfile in its scripts folder); only Bun has to be there",
        "install Bun: curl -fsSL https://bun.sh/install | bash",
    ],
};

/// The engine ships with the `scripts` folder next to refac; it is present
/// once the packages are installed.
fn check() -> Vec<Attempt> {
    let scripts = crate::drivers::resolve_resource_path("scripts/ts_refactor.ts");
    let Ok(script) = scripts else {
        return vec![Attempt {
            place: "the scripts folder next to refac".to_string(),
            outcome: Outcome::NotThere("scripts/ts_refactor.ts was not found".to_string()),
        }];
    };
    let packages = script
        .parent()
        .map(|dir| dir.join("node_modules/typescript-native"));
    match packages {
        Some(path) if path.exists() => vec![Attempt {
            place: path.display().to_string(),
            outcome: Outcome::Found(path),
        }],
        Some(path) => vec![Attempt {
            place: path.display().to_string(),
            outcome: Outcome::NotThere(
                "not installed yet; refac installs it on first use".to_string(),
            ),
        }],
        None => Vec::new(),
    }
}
