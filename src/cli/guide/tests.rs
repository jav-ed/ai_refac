//! The guide texts are embedded files; these checks keep the list and the
//! texts honest.

use super::*;

#[test]
fn every_topic_has_a_text_that_opens_with_its_title() {
    for (name, answers, text) in TOPICS {
        assert!(!answers.is_empty(), "{name} has no summary");
        let first = text.lines().next().unwrap();
        assert!(
            first.starts_with(&name.to_uppercase()) || first.contains(':'),
            "{name}: the text should open with a title line, got `{first}`"
        );
        assert!(
            text.lines().count() > 20,
            "{name} is too short to be a guide"
        );
    }
}

#[test]
fn the_list_names_every_topic_and_the_other_two_depths() {
    let listing = list();
    for (name, _, _) in TOPICS {
        assert!(listing.contains(name), "{listing}");
    }
    assert!(
        listing.contains("-h") && listing.contains("--help"),
        "{listing}"
    );
}

#[test]
fn an_unknown_topic_is_an_error_that_lists_the_topics() {
    let error = execute_guide(GuideArgs {
        topic: Some("nonsense".to_string()),
    })
    .unwrap_err();
    let message = format!("{:#}", error.error);
    assert!(
        message.contains("`nonsense` is not a guide topic"),
        "{message}"
    );
    assert!(message.contains("languages"), "{message}");
}

#[test]
fn the_servers_guide_names_the_variable_of_every_language_server() {
    let (_, _, text) = TOPICS
        .iter()
        .find(|(name, _, _)| *name == "servers")
        .unwrap();
    for server in crate::servers::all() {
        if server.env_var.is_empty() {
            continue;
        }
        assert!(
            text.contains(server.env_var),
            "the servers guide lacks {}",
            server.env_var
        );
    }
}
