use crate::common;

use std::{fs, path::Path, process::Output};

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, content).unwrap();
}

fn run(project: &Path, source: &str, target: &str) -> Output {
    common::run_cli(&[
        "move-module",
        "--project-path",
        project.to_str().unwrap(),
        source,
        target,
    ])
}

fn run_json(project: &Path, source: &str, target: &str) -> Output {
    common::run_cli(&[
        "move-module",
        "--json",
        "--project-path",
        project.to_str().unwrap(),
        source,
        target,
    ])
}

mod basic;
mod declarations;
mod imports;
mod paths;
