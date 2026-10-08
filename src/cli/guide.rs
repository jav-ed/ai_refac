//! `refac guide [topic]`: in-depth documentation inside the binary, for a
//! person or an agent that has only the command. Three depths in all: `-h`
//! (a few lines), `--help` (the full reference of one command) and this
//! (topics that cut across commands: languages, safety, cost, output,
//! servers). The texts are plain files under `guide/`, printed as they are.

use super::CliError;
use clap::Args;

/// (name, what it answers, text)
const TOPICS: &[(&str, &str, &str)] = &[
    (
        "languages",
        "what refac does in each language, and where it stops",
        include_str!("guide/languages.txt"),
    ),
    (
        "safety",
        "what is checked, what is written, what happens when something fails",
        include_str!("guide/safety.txt"),
    ),
    (
        "batching",
        "what a command costs and how to do many changes with few server starts",
        include_str!("guide/batching.txt"),
    ),
    (
        "output",
        "how to read the report, the JSON documents and the exit codes",
        include_str!("guide/output.txt"),
    ),
    (
        "servers",
        "which language servers are started, where they are found, how to fix a missing one",
        include_str!("guide/servers.txt"),
    ),
];

#[derive(Debug, Args)]
pub struct GuideArgs {
    /// The topic to print: languages, safety, batching, output or servers.
    ///
    /// Without it, the topics are listed with what each answers. `all` prints every topic.
    #[arg(value_name = "TOPIC")]
    pub topic: Option<String>,
}

pub fn execute_guide(args: GuideArgs) -> Result<(), CliError> {
    match args.topic.as_deref() {
        None => print!("{}", list()),
        Some("all") => {
            for (_, _, text) in TOPICS {
                println!("{text}");
            }
        }
        Some(name) => match TOPICS.iter().find(|(topic, _, _)| *topic == name) {
            Some((_, _, text)) => print!("{text}"),
            None => {
                return Err(CliError {
                    json: false,
                    error: anyhow::anyhow!("`{name}` is not a guide topic.\n\n{}", list()),
                });
            }
        },
    }
    Ok(())
}

fn list() -> String {
    let mut text =
        String::from("Guide topics (refac guide <topic>; `refac guide all` prints every one):\n\n");
    for (name, answers, _) in TOPICS {
        text.push_str(&format!("  {name:<10} {answers}\n"));
    }
    text.push_str(
        "\nFor one command: `refac <command> -h` (a few lines) and `refac <command> --help` (everything).\n",
    );
    text
}

#[cfg(test)]
mod tests;
