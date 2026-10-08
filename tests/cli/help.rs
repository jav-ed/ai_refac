//! The help a person or an agent reads: `-h` is the summary, `--help` is the whole
//! description with examples, and a bare `refac` shows the usage instead of failing
//! with a one-line error.

use crate::common::{run_cli, stderr_text, stdout_text};

const COMMANDS: &[&str] = &[
    "move",
    "move-module",
    "rename",
    "doctor",
    "guide",
    "completions",
    "man",
];

#[test]
fn every_command_has_a_full_help_with_a_summary_and_a_short_one_without_the_details() {
    for command in COMMANDS {
        let long = run_cli(&[command, "--help"]);
        assert!(
            long.status.success(),
            "{command} --help: {}",
            stderr_text(&long)
        );
        let long = stdout_text(&long);
        assert!(long.contains("Usage:"), "{command}: {long}");

        let short = run_cli(&[command, "-h"]);
        assert!(
            short.status.success(),
            "{command} -h: {}",
            stderr_text(&short)
        );
        let short = stdout_text(&short);
        assert!(short.contains("Usage:"), "{command}: {short}");
        assert!(
            short.len() < long.len(),
            "{command}: -h ({} bytes) must be shorter than --help ({} bytes)",
            short.len(),
            long.len()
        );
    }
}

#[test]
fn the_commands_that_change_files_explain_rules_output_and_examples() {
    for command in ["move", "move-module", "rename", "doctor"] {
        let text = stdout_text(&run_cli(&[command, "--help"]));
        assert!(
            text.contains("EXAMPLES"),
            "{command} has no examples: {text}"
        );
        assert!(
            text.contains("WHAT YOU GET BACK"),
            "{command} does not say how to read its output: {text}"
        );
    }
}

#[test]
fn top_help_lists_languages_safety_costs_exit_codes_and_environment() {
    let text = stdout_text(&run_cli(&["--help"]));
    for heading in [
        "COMMANDS",
        "WHAT EACH LANGUAGE SUPPORTS",
        "HOW IT STAYS SAFE",
        "WHAT A COMMAND COSTS",
        "READING THE OUTPUT",
        "EXIT CODES",
        "ENVIRONMENT",
        "EXAMPLES",
    ] {
        assert!(text.contains(heading), "missing {heading}: {text}");
    }
}

#[test]
fn rename_help_describes_batch_dry_run_and_ambiguity() {
    let text = stdout_text(&run_cli(&["rename", "--help"]));
    for needle in [
        "--batch",
        "--dry-run",
        "--line",
        "all or nothing",
        "WHEN THE NAME IS NOT UNIQUE",
    ] {
        assert!(text.contains(needle), "missing {needle}: {text}");
    }
}

/// The server catalog is the one list of the variables that point refac at a server, so
/// the help cannot forget one when a language is added.
#[test]
fn top_help_names_the_variable_of_every_language_server() {
    let text = stdout_text(&run_cli(&["--help"]));
    for server in refac::servers::all() {
        if server.env_var.is_empty() {
            continue;
        }
        assert!(
            text.contains(server.env_var),
            "the top help does not name {}, the variable of {}",
            server.env_var,
            server.name
        );
    }
}

#[test]
fn a_bare_refac_shows_the_usage_and_exits_with_the_usage_code() {
    let output = run_cli(&[]);
    assert_eq!(output.status.code(), Some(2));
    let text = format!("{}{}", stdout_text(&output), stderr_text(&output));
    assert!(text.contains("Usage: refac <COMMAND>"), "{text}");
    assert!(text.contains("move-module"), "{text}");
}

#[test]
fn the_guide_lists_its_topics_and_prints_each_one() {
    let listing = run_cli(&["guide"]);
    assert!(listing.status.success(), "{}", stderr_text(&listing));
    let listing = stdout_text(&listing);
    let topics = ["languages", "safety", "batching", "output", "servers"];
    for topic in topics {
        assert!(listing.contains(topic), "the list lacks {topic}: {listing}");
        let page = run_cli(&["guide", topic]);
        assert!(
            page.status.success(),
            "guide {topic}: {}",
            stderr_text(&page)
        );
        assert!(
            stdout_text(&page).len() > 1500,
            "guide {topic} is too short"
        );
    }
    let all = stdout_text(&run_cli(&["guide", "all"]));
    for topic in topics {
        assert!(
            all.contains(&topic.to_uppercase()),
            "`guide all` lacks {topic}"
        );
    }
}

#[test]
fn an_unknown_guide_topic_fails_with_the_list_of_topics() {
    let output = run_cli(&["guide", "nonsense"]);
    assert_eq!(output.status.code(), Some(1));
    let error = stderr_text(&output);
    assert!(error.contains("`nonsense` is not a guide topic"), "{error}");
    assert!(
        error.contains("languages") && error.contains("servers"),
        "{error}"
    );
}
