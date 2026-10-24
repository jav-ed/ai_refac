//! Symbol rename in projects with `.astro` and `.mdx` files.

use crate::common::project::{Project, assert_same_tree};

const RENAME_TSCONFIG: &str = r#"{
  "compilerOptions": {
    "module": "esnext",
    "moduleResolution": "bundler",
    "target": "es2022",
    "strict": true,
    "paths": { "@lib/*": ["./src/lib/*"] }
  },
  "include": ["src/**/*"]
}"#;

fn rename_project() -> Project {
    let project = Project::empty();
    project.write("tsconfig.json", RENAME_TSCONFIG);
    project.write(
        "src/lib/format.ts",
        "export const formatDate = (date: Date) => date.toISOString();\nexport const other = 1;\n",
    );
    project.write(
        "src/lib/use.ts",
        "import { formatDate } from \"./format\";\nexport const today = formatDate(new Date());\n",
    );
    project
}

fn rename_symbol(
    project: &Project,
    file: &str,
    symbol: &str,
    new_name: &str,
) -> std::process::Output {
    crate::common::run_cli(&[
        "rename",
        "--project-path",
        project.path().to_str().unwrap(),
        "--file",
        file,
        "--symbol",
        symbol,
        "--new-name",
        new_name,
    ])
}

/// The language server cannot read inside `.astro` and `.mdx` files. A rename
/// used to rewrite the `.ts` files, leave these on the old name, and exit 0.
#[test]
fn a_rename_that_component_files_would_not_follow_is_refused_and_changes_nothing() {
    let project = rename_project();
    project.write(
        "src/pages/index.astro",
        "---\nimport { formatDate } from \"@lib/format\";\n---\n<p>{formatDate(new Date())}</p>\n",
    );
    project.write(
        "src/content/post.mdx",
        "import { formatDate } from \"../lib/format\";\n\n{formatDate(new Date())}\n",
    );
    let before = project.tree();

    let output = rename_symbol(&project, "src/lib/format.ts", "formatDate", "formatDay");

    assert!(!output.status.success());
    let error = crate::common::stderr_text(&output);
    assert!(
        error.contains("`formatDate` is imported by 2 component file(s)"),
        "{error}"
    );
    assert!(error.contains("src/pages/index.astro"), "{error}");
    assert!(error.contains("src/content/post.mdx"), "{error}");
    assert_same_tree(&before, &project.tree());
}

/// Component files that never write the name do not stand in the way.
#[test]
fn a_rename_is_not_refused_for_components_that_do_not_use_the_name() {
    let project = rename_project();
    project.write(
        "src/pages/index.astro",
        "---\nimport { other } from \"@lib/format\";\n---\n<p>{other}</p>\n",
    );

    let output = rename_symbol(&project, "src/lib/format.ts", "formatDate", "formatDay");

    assert!(
        output.status.success(),
        "{}",
        crate::common::stderr_text(&output)
    );
    assert!(project.read("src/lib/use.ts").contains("formatDay"));
}

/// A project the TypeScript engine reads `.astro` files in: its tsconfig lists
/// the Astro content mapper, which is installed from the helper's own
/// `node_modules` (it is a dev dependency of `scripts/` for this reason).
fn mapped_project() -> Project {
    let project = Project::empty();
    project.write(
        "tsconfig.json",
        r#"{
  "compilerOptions": {
    "module": "esnext",
    "moduleResolution": "bundler",
    "target": "es2022",
    "strict": true
  },
  "include": ["src/**/*"],
  "contentMappers": [{ "package": "@astrojs/ts-content-mapper", "extensions": [".astro"] }]
}"#,
    );
    let mapper = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("scripts/node_modules/@astrojs/ts-content-mapper");
    assert!(
        mapper.is_dir(),
        "{} is missing: run `bun install` in scripts/",
        mapper.display()
    );
    let link = project
        .path()
        .join("node_modules/@astrojs/ts-content-mapper");
    std::fs::create_dir_all(link.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&mapper, &link).unwrap();
    project.write(
        "src/lib/track.ts",
        "export function track(event: string): void {\n  console.log(\"track\", event);\n}\n",
    );
    project.write(
        "src/components/Card.astro",
        "---\nimport { track } from \"../lib/track\";\ntrack(\"render\");\n---\n<button data-track=\"open\">Open</button>\n<script>\n  import { track } from \"../lib/track\";\n  document.querySelector(\"button\")?.addEventListener(\"click\", () => track(\"click\"));\n</script>\n",
    );
    project
}

fn rename_with_astro(project: &Project, extra: &[&str]) -> std::process::Output {
    let mut args = vec![
        "rename",
        "--project-path",
        project.path().to_str().unwrap(),
        "--file",
        "src/lib/track.ts",
        "--symbol",
        "track",
        "--new-name",
        "trackEvent",
    ];
    args.extend_from_slice(extra);
    crate::common::run_cli(&args)
}

/// With `--astro` the engine reads the `.astro` file through the content
/// mapper, so the frontmatter and the bundled script follow the rename, and
/// the same-looking text elsewhere (an attribute, a log string) does not.
#[test]
fn a_rename_with_astro_follows_the_symbol_into_the_astro_files() {
    let project = mapped_project();

    let output = rename_with_astro(&project, &["--astro"]);

    assert!(
        output.status.success(),
        "{}",
        crate::common::stderr_text(&output)
    );
    assert_eq!(
        project.read("src/components/Card.astro"),
        "---\nimport { trackEvent } from \"../lib/track\";\ntrackEvent(\"render\");\n---\n<button data-track=\"open\">Open</button>\n<script>\n  import { trackEvent } from \"../lib/track\";\n  document.querySelector(\"button\")?.addEventListener(\"click\", () => trackEvent(\"click\"));\n</script>\n"
    );
    assert!(
        project
            .read("src/lib/track.ts")
            .contains("function trackEvent")
    );
}

/// Running a project's content mapper is code from `node_modules`: never the
/// default, and the refusal names the way to allow it.
#[test]
fn a_project_with_content_mappers_is_refused_without_astro_and_changes_nothing() {
    let project = mapped_project();
    let before = project.tree();

    let output = rename_with_astro(&project, &[]);

    assert!(!output.status.success());
    let error = crate::common::stderr_text(&output);
    assert!(error.contains("contentMappers"), "{error}");
    assert!(error.contains("--astro"), "{error}");
    assert_same_tree(&before, &project.tree());
}

/// `--astro` with no mapper listed would rename in `.ts` files and leave every
/// `.astro` file behind; it stops instead.
#[test]
fn astro_without_a_content_mapper_in_the_tsconfig_is_refused() {
    let project = mapped_project();
    project.write("tsconfig.json", RENAME_TSCONFIG);
    let before = project.tree();

    let output = rename_with_astro(&project, &["--astro"]);

    assert!(!output.status.success());
    let error = crate::common::stderr_text(&output);
    assert!(error.contains("lists no .astro file"), "{error}");
    assert!(error.contains("contentMappers"), "{error}");
    assert_same_tree(&before, &project.tree());
}

/// The mapper reads `.astro` only. A `.mdx` file that uses the name is still
/// out of reach, so `--astro` does not remove that refusal.
#[test]
fn an_mdx_user_still_stops_a_rename_with_astro() {
    let project = mapped_project();
    project.write(
        "src/content/post.mdx",
        "import { track } from \"../lib/track\";\n\n{track(\"post\")}\n",
    );
    let before = project.tree();

    let output = rename_with_astro(&project, &["--astro"]);

    assert!(!output.status.success());
    let error = crate::common::stderr_text(&output);
    assert!(error.contains("src/content/post.mdx"), "{error}");
    assert!(!error.contains("Card.astro"), "{error}");
    assert_same_tree(&before, &project.tree());
}

/// A symbol declared in the frontmatter of an `.astro` file is renamed where
/// it is used in the template too, and only with `--astro`.
#[test]
fn a_symbol_declared_in_an_astro_file_is_renamed_with_astro() {
    let project = mapped_project();
    project.write(
        "src/components/Label.astro",
        "---\nconst label = \"Open\";\n---\n<button aria-label={label}>{label}</button>\n",
    );
    let rename = |extra: &[&str]| {
        let mut args = vec![
            "rename",
            "--project-path",
            project.path().to_str().unwrap(),
            "--file",
            "src/components/Label.astro",
            "--symbol",
            "label",
            "--new-name",
            "caption",
        ];
        args.extend_from_slice(extra);
        crate::common::run_cli(&args)
    };

    let refused = rename(&[]);
    assert!(!refused.status.success());
    assert!(
        crate::common::stderr_text(&refused).contains("--astro"),
        "{}",
        crate::common::stderr_text(&refused)
    );

    // `aria-label` is a second candidate named `label` (an attribute), so the
    // engine asks which one is meant: line 2 is the declaration.
    let ambiguous = rename(&["--astro"]);
    assert!(
        crate::common::stderr_text(&ambiguous).contains("several different symbols"),
        "{}",
        crate::common::stderr_text(&ambiguous)
    );

    let done = rename(&["--astro", "--line", "2"]);
    assert!(
        done.status.success(),
        "{}",
        crate::common::stderr_text(&done)
    );
    assert_eq!(
        project.read("src/components/Label.astro"),
        "---\nconst caption = \"Open\";\n---\n<button aria-label={caption}>{caption}</button>\n"
    );
}
