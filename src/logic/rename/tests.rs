//! What a batch refuses before any language server is started.

use super::*;

fn request(file: &str) -> RenameRequest {
    RenameRequest {
        project_path: ".".into(),
        file: file.into(),
        symbol: "old".to_string(),
        new_name: "new".to_string(),
        line: None,
        column: None,
        dry_run: true,
    }
}

async fn refusal(requests: Vec<RenameRequest>) -> String {
    format!("{:#}", handle_rename_batch(requests).await.unwrap_err())
}

#[tokio::test]
async fn an_empty_batch_is_refused() {
    assert!(refusal(Vec::new()).await.contains("at least one rename"));
}

#[tokio::test]
async fn a_batch_renames_in_one_language() {
    let message = refusal(vec![request("a.go"), request("b.rs")]).await;

    assert!(
        message.contains("rename 2 is a Rust file (b.rs)"),
        "{message}"
    );
    assert!(message.contains("rename 1 a Go file (a.go)"), "{message}");
    assert!(message.contains("one batch per language"), "{message}");
}

#[tokio::test]
async fn typescript_takes_one_rename_per_command() {
    let message = refusal(vec![request("a.ts"), request("b.ts")]).await;

    assert!(
        message.contains("Kotlin, Go, Rust, Python and Dart"),
        "{message}"
    );
}

#[tokio::test]
async fn an_unknown_extension_is_named() {
    let message = refusal(vec![request("a.rb")]).await;

    assert!(message.contains("got a.rb"), "{message}");
}

#[test]
fn the_extension_picks_the_backend() {
    let backend = |file: &str| Backend::of(Path::new(file)).unwrap();

    assert_eq!(backend("src/a.tsx"), Backend::TypeScript);
    assert_eq!(backend("A.kt"), Backend::Kotlin);
    assert_eq!(backend("a.go"), Backend::Go);
    assert_eq!(backend("a.rs"), Backend::Rust);
    assert_eq!(backend("a.py"), Backend::Python);
    assert_eq!(backend("a.dart"), Backend::Dart);
}
