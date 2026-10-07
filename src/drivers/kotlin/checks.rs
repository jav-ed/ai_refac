//! Checks on the server's work that are cheap to make: a moved file's
//! `package` line follows its new directory, and a file renamed in place takes
//! the class that carried its name along. The server is trusted for the
//! reference updates themselves.

use super::declarations::{declared_package, top_level_types};
use super::project::{package_of_dir, source_root};
use anyhow::{Result, bail};
use std::path::Path;

pub enum Check {
    Verified,
    /// The file did not follow its directory before the move, so there is
    /// nothing to compare against; the message is reported to the user.
    Unverified(String),
}

/// Compare the package of a file after its move with the package its new
/// directory stands for. `before` and `after` are the file's text on both sides.
pub fn check_moved_file(from: &Path, to: &Path, before: &str, after: &str) -> Result<Check> {
    check_class_follows_name(from, to, before, after)?;
    let package_of = |path: &Path| {
        let root = source_root(path)?;
        package_of_dir(&root, path.parent()?)
    };
    let declared_before = declared_package(before);
    if declared_before != package_of(from) {
        return Ok(Check::Unverified(format!(
            "{} declares package {} that does not follow its directory; the server's result was not checked",
            from.display(),
            declared_before.as_deref().unwrap_or("(default)")
        )));
    }
    let expected = package_of(to);
    let declared_after = declared_package(after);
    if declared_after != expected {
        bail!(
            "The server left package {} in {}, but its new directory stands for {}",
            declared_after.as_deref().unwrap_or("(default)"),
            to.display(),
            expected.as_deref().unwrap_or("(default)")
        );
    }
    Ok(Check::Verified)
}

/// A file renamed in place takes along the type that carried its name
/// (`Greeter.kt` holding `class Greeter`). A server that renamed only the
/// file would leave file and type apart without saying so.
fn check_class_follows_name(from: &Path, to: &Path, before: &str, after: &str) -> Result<()> {
    if from.parent() != to.parent() {
        return Ok(());
    }
    let (Some(old), Some(new)) = (stem(from), stem(to)) else {
        return Ok(());
    };
    if !top_level_types(before).contains(&old) {
        return Ok(());
    }
    let declared = top_level_types(after);
    if declared.contains(&new) && !declared.contains(&old) {
        return Ok(());
    }
    bail!(
        "{} was renamed to {}, but the server did not rename the type {old} to {new} with it",
        from.display(),
        to.display()
    )
}

fn stem(path: &Path) -> Option<String> {
    Some(path.file_stem()?.to_str()?.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const FROM: &str = "/p/src/main/kotlin/com/example/util/Helper.kt";
    const TO: &str = "/p/src/main/kotlin/com/example/common/Helper.kt";

    #[test]
    fn accepts_a_package_that_followed_the_move() {
        let check = check_moved_file(
            Path::new(FROM),
            Path::new(TO),
            "package com.example.util\n",
            "package com.example.common\n",
        )
        .unwrap();
        assert!(matches!(check, Check::Verified));
    }

    #[test]
    fn refuses_a_package_the_server_left_behind() {
        let error = check_moved_file(
            Path::new(FROM),
            Path::new(TO),
            "package com.example.util\n",
            "package com.example.util\n",
        )
        .err()
        .unwrap()
        .to_string();
        assert!(error.contains("com.example.common"), "{error}");
    }

    #[test]
    fn a_file_that_never_followed_its_directory_is_reported_not_judged() {
        let check = check_moved_file(
            Path::new(FROM),
            Path::new(TO),
            "package legacy.name\n",
            "package legacy.name\n",
        )
        .unwrap();
        assert!(matches!(check, Check::Unverified(_)));
    }

    #[test]
    fn a_rename_in_place_keeps_the_package() {
        let to = "/p/src/main/kotlin/com/example/util/Utilities.kt";
        let check = check_moved_file(
            Path::new(FROM),
            Path::new(to),
            "package com.example.util\n",
            "package com.example.util\n",
        )
        .unwrap();
        assert!(matches!(check, Check::Verified));
    }

    #[test]
    fn a_renamed_file_must_take_its_type_along() {
        let from = Path::new("/p/src/main/kotlin/com/example/app/Greeter.kt");
        let to = Path::new("/p/src/main/kotlin/com/example/app/Welcomer.kt");
        let before = "package com.example.app\n\nclass Greeter\n";
        let renamed = "package com.example.app\n\nclass Welcomer\n";
        assert!(check_moved_file(from, to, before, renamed).is_ok());
        let error = check_moved_file(from, to, before, before)
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("Greeter"), "{error}");
        // A file whose name matches no type has nothing that has to follow.
        let functions = "package com.example.app\n\nfun greet() {}\n";
        assert!(check_moved_file(from, to, functions, functions).is_ok());
    }
}
