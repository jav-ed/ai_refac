//! A documentation tree of 3,000 Markdown files: one folder moves, every link
//! to or from it follows, and the whole run stays quick.

use crate::common;

use common::links::{edges, moved_edges};
use common::project::Project;
use std::time::{Duration, Instant};

const FOLDERS: usize = 100;
const FILES_PER_FOLDER: usize = 30;

fn tree_of_3000_files() -> Project {
    let project = Project::empty();
    project.write_bytes("img/logo.png", b"PNG");
    for folder in 0..FOLDERS {
        for file in 0..FILES_PER_FOLDER {
            let next = (file + 1) % FILES_PER_FOLDER;
            let neighbour_folder = (folder + 1) % FOLDERS;
            project.write(
                &format!("d{folder:03}/p{file:02}.md"),
                &format!(
                    "# Page {folder}.{file}\n\n\
                     Next: [next](p{next:02}.md), other folder: [there](../d{neighbour_folder:03}/p00.md#top).\n\
                     Logo: ![logo](../img/logo.png) and <img src=\"../img/logo.png\">.\n\n\
                     ```\n[code](p99.md)\n```\n\n\
                     [def]: p{next:02}.md\n"
                ),
            );
        }
    }
    project
}

#[test]
fn moving_a_folder_in_a_big_tree_keeps_every_link_and_stays_quick() {
    let project = tree_of_3000_files();
    let before = project.tree();
    assert_eq!(
        before.keys().filter(|path| path.ends_with(".md")).count(),
        FOLDERS * FILES_PER_FOLDER
    );

    let started = Instant::now();
    let output = project.move_ok(&[("d050", "archive/d050")]);
    let elapsed = started.elapsed();
    eprintln!("moving one folder in a 3,000 file tree took {elapsed:?}");

    assert!(output.contains("Checked 3000 Markdown files"), "{output}");
    // The folder and its neighbours change (d049 links into it, d050 links
    // out of it); the rest of the tree must be byte for byte as it was.
    let after = project.tree();
    for (path, bytes) in &before {
        let folder = path.split('/').next().unwrap();
        if !["d049", "d050", "d051"].contains(&folder) && folder != "img" {
            assert_eq!(after.get(path), Some(bytes), "{path} must not change");
        }
    }
    assert_eq!(
        edges(&after),
        moved_edges(&edges(&before), &|path| {
            match path.strip_prefix("d050") {
                Some(rest) if rest.is_empty() || rest.starts_with('/') => {
                    format!("archive/d050{rest}")
                }
                _ => path.to_string(),
            }
        })
    );
    assert!(
        elapsed < Duration::from_secs(60),
        "a folder move in a 3,000 file tree took {elapsed:?}"
    );
}
