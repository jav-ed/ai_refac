use super::*;
use std::fs;
use tempfile::tempdir;

fn pair(source: &str) -> (String, String) {
    (source.to_string(), format!("moved/{source}"))
}

fn write(root: &Path, file: &str, content: &str) {
    let path = root.join(file);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

#[test]
fn a_file_is_used_when_code_names_it_and_not_otherwise() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "src/pages/index.astro",
        "---\nimport art from \"../assets/hero.svg\";\n---\n",
    );
    write(dir.path(), "assets/hero.svg", "");
    write(dir.path(), "assets/unused.svg", "");

    let used = used_by_code(
        &[pair("assets/hero.svg"), pair("assets/unused.svg")],
        Some(dir.path()),
    )
    .unwrap();

    assert_eq!(used, [true, false]);
}

#[test]
fn a_folder_is_used_when_code_names_any_file_in_it() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "src/lib/photo.ts",
        "import a from \"../../gallery/a.png\";\n",
    );
    write(dir.path(), "gallery/a.png", "");
    write(dir.path(), "gallery/b.png", "");
    write(dir.path(), "other/c.png", "");

    let used = used_by_code(&[pair("gallery"), pair("other")], Some(dir.path())).unwrap();

    assert_eq!(used, [true, false]);
}

/// The relative paths in an `.mdx` page's own imports change with its folder,
/// whether or not any code names the page.
#[test]
fn an_mdx_page_with_imports_is_used_but_one_with_prose_only_is_not() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "content/with.mdx",
        "import Card from \"../Card.astro\";\n\n# Title\n",
    );
    write(
        dir.path(),
        "content/exported.mdx",
        "export const meta = {};\n\n# Title\n",
    );
    write(
        dir.path(),
        "content/plain.mdx",
        "# Title\n\nimportant words, exports too.\n",
    );

    let used = used_by_code(
        &[
            pair("content/with.mdx"),
            pair("content/exported.mdx"),
            pair("content/plain.mdx"),
        ],
        Some(dir.path()),
    )
    .unwrap();

    assert_eq!(used, [true, true, false]);
}

/// Dependencies and build output name everything; they must not decide.
#[test]
fn dependency_and_ignored_folders_are_not_searched() {
    let dir = tempdir().unwrap();
    write(dir.path(), ".gitignore", "dist/\n");
    write(
        dir.path(),
        "node_modules/pkg/index.js",
        "require(\"hero.svg\");\n",
    );
    write(dir.path(), "dist/bundle.js", "\"hero.svg\"\n");
    write(dir.path(), "assets/hero.svg", "");

    let used = used_by_code(&[pair("assets/hero.svg")], Some(dir.path())).unwrap();

    assert_eq!(used, [false]);
}

#[test]
fn only_the_pairs_that_code_uses_are_handed_to_the_engine() {
    let dir = tempdir().unwrap();
    write(dir.path(), "src/a.ts", "import \"../hero.svg\";\n");
    write(dir.path(), "hero.svg", "");
    write(dir.path(), "notes.md", "");
    let batches = HashMap::from([(
        "markdown".to_string(),
        vec![pair("hero.svg"), pair("notes.md")],
    )]);

    let used = used_pairs(&batches, Some(dir.path())).unwrap();

    assert_eq!(used, [pair("hero.svg")]);
}

#[test]
fn a_request_without_a_markdown_batch_has_nothing_for_the_engine() {
    let batches = HashMap::from([("python".to_string(), vec![pair("a.py")])]);

    assert!(used_pairs(&batches, None).unwrap().is_empty());
}

/// The Markdown driver already fixes `](img/hero.png)`; only a quoted path is
/// code, and a longer name that ends alike is another file.
#[test]
fn a_markdown_link_or_a_longer_name_is_not_a_use() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "content/post.mdx",
        "# Post\n\n![hero](../img/hero.png)\n",
    );
    write(dir.path(), "src/a.ts", "import \"../img/superhero.png\";\n");
    write(dir.path(), "img/hero.png", "");

    let used = used_by_code(&[pair("img/hero.png")], Some(dir.path())).unwrap();

    assert_eq!(used, [false]);
}

#[test]
fn a_query_or_fragment_after_the_name_is_still_a_use() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "src/a.ts",
        "import url from \"../img/hero.png?url\";\n",
    );
    write(
        dir.path(),
        "src/b.ts",
        "import \"../img/other.png#frag\";\n",
    );
    write(dir.path(), "img/hero.png", "");
    write(dir.path(), "img/other.png", "");

    let used = used_by_code(
        &[pair("img/hero.png"), pair("img/other.png")],
        Some(dir.path()),
    )
    .unwrap();

    assert_eq!(used, [true, true]);
}

/// A stylesheet writes its paths without quotes, in `url(...)`; a moved
/// stylesheet with `url()` or `@import` has paths of its own to fix.
#[test]
fn a_stylesheet_url_is_a_use_and_a_moved_stylesheet_with_urls_is_used() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "src/theme.css",
        ".a { background: url(../img/hero.png); }\n",
    );
    write(dir.path(), "src/other.css", ".b { color: red; }\n");
    write(dir.path(), "img/hero.png", "");
    write(dir.path(), "img/plain.png", "");
    write(
        dir.path(),
        "img/own.css",
        ".c { background: url(./x.png); }\n",
    );
    write(dir.path(), "img/none.css", ".d { color: blue; }\n");

    let used = used_by_code(
        &[
            pair("img/hero.png"),
            pair("img/plain.png"),
            pair("img/own.css"),
            pair("img/none.css"),
        ],
        Some(dir.path()),
    )
    .unwrap();

    assert_eq!(used, [true, false, true, false]);
}

/// A content collection names its folder, not the files in it
/// (`base: "./src/Content/blog"`).
#[test]
fn a_folder_is_used_when_code_names_the_folder_itself() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "src/content.config.ts",
        "export const blog = { base: \"./src/Content/blog\" };\nexport const deep = \"./src/Content/news/2024\";\n",
    );
    write(dir.path(), "src/Content/blog/post.mdx", "# Post\n");
    write(dir.path(), "src/Content/news/a.mdx", "# A\n");
    write(dir.path(), "src/Content/legal/b.mdx", "# B\n");
    write(dir.path(), "src/Content/other/c.mdx", "# C\n");
    // Another folder whose name only ends alike.
    write(dir.path(), "src/Content/superblog/d.mdx", "# D\n");

    let used = used_by_code(
        &[
            pair("src/Content/blog"),
            pair("src/Content/news"),
            pair("src/Content/legal"),
            pair("src/Content/superblog"),
        ],
        Some(dir.path()),
    )
    .unwrap();

    assert_eq!(used, [true, true, false, false]);
}

/// A page's frontmatter names pictures the way a content collection reads them
/// (`image: ../assets/hero.png`), unquoted and ending the line.
#[test]
fn a_picture_named_by_a_frontmatter_value_is_a_use() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "content/post.mdx",
        "---\nseo:\n  image: ../assets/hero.png\n  other: \"../assets/quoted.png\"\n---\n\nText.\n",
    );
    write(
        dir.path(),
        "content/note.md",
        "---\nhero: ../assets/note.png\n---\n",
    );
    write(dir.path(), "assets/hero.png", "");
    write(dir.path(), "assets/quoted.png", "");
    write(dir.path(), "assets/note.png", "");
    write(dir.path(), "assets/unused.png", "");
    // Prose that mentions a name is not a path, and a longer name is another file.
    write(
        dir.path(),
        "content/prose.md",
        "Look at superunused.png in prose.\n",
    );

    let used = used_by_code(
        &[
            pair("assets/hero.png"),
            pair("assets/quoted.png"),
            pair("assets/note.png"),
            pair("assets/unused.png"),
        ],
        Some(dir.path()),
    )
    .unwrap();

    assert_eq!(used, [true, true, true, false]);
}

#[test]
fn a_moved_page_with_a_relative_frontmatter_path_is_used() {
    let dir = tempdir().unwrap();
    write(
        dir.path(),
        "content/with.mdx",
        "\u{feff}---\n- ../assets/a.png\n---\n",
    );
    write(
        dir.path(),
        "content/listed.md",
        "---\ngallery:\n  - './a.png'\n---\n",
    );
    write(
        dir.path(),
        "content/absolute.md",
        "---\nimage: /assets/a.png\ncanonical: https://example.com/../a\n---\n",
    );
    write(dir.path(), "content/rule.md", "---\nnot closed: ../a.png\n");
    write(
        dir.path(),
        "content/body.md",
        "# Title\n\nimage: ../a.png\n",
    );

    let used = used_by_code(
        &[
            pair("content/with.mdx"),
            pair("content/listed.md"),
            pair("content/absolute.md"),
            pair("content/rule.md"),
            pair("content/body.md"),
        ],
        Some(dir.path()),
    )
    .unwrap();

    assert_eq!(used, [true, true, false, false, false]);
}
