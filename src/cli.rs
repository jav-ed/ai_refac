use std::io::{self, Write};
use std::process::ExitCode;

use anyhow::Result;
use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::generate;
use clap_mangen::Man;
use serde::Serialize;

mod args;
mod doctor;
mod guide;
mod help;
mod moves;
mod output;
mod rename;

use args::{CompletionsArgs, MoveArgs, MoveModuleArgs, RenameArgs};
use output::ErrorOutput;

#[derive(Debug, Parser)]
#[command(
    name = "refac",
    about = help::top::ABOUT,
    long_about = help::top::LONG_ABOUT,
    after_long_help = help::top::AFTER_LONG_HELP,
    version,
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    #[command(
        about = help::move_files::ABOUT,
        long_about = help::move_files::LONG_ABOUT,
        after_long_help = help::move_files::AFTER_LONG_HELP
    )]
    Move(MoveArgs),
    #[command(
        about = help::move_module::ABOUT,
        long_about = help::move_module::LONG_ABOUT,
        after_long_help = help::move_module::AFTER_LONG_HELP
    )]
    MoveModule(MoveModuleArgs),
    #[command(
        about = help::rename::ABOUT,
        long_about = help::rename::LONG_ABOUT,
        after_long_help = help::rename::AFTER_LONG_HELP
    )]
    Rename(RenameArgs),
    #[command(
        about = help::doctor::ABOUT,
        long_about = help::doctor::LONG_ABOUT,
        after_long_help = help::doctor::AFTER_LONG_HELP
    )]
    Doctor(doctor::DoctorArgs),
    /// In-depth documentation by topic: languages, safety, batching, output, servers.
    ///
    /// `refac guide` lists the topics; `refac guide <topic>` prints one; `refac guide all` prints
    /// every topic. The texts are inside the binary, so they are there when the repository is not.
    Guide(guide::GuideArgs),
    /// Print shell completions (bash, zsh, fish, elvish, powershell) to stdout.
    ///
    /// Example: `refac completions bash > ~/.local/share/bash-completion/completions/refac`
    Completions(CompletionsArgs),
    /// Print the manual page (roff) to stdout.
    ///
    /// Example: `refac man | man -l -`
    Man,
}

#[derive(Debug)]
struct CliError {
    json: bool,
    error: anyhow::Error,
}

pub async fn run() -> ExitCode {
    let cli = Cli::parse();

    match execute(cli).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            render_error(&err);
            ExitCode::FAILURE
        }
    }
}

async fn execute(cli: Cli) -> Result<(), CliError> {
    match cli.command {
        Commands::Move(args) => moves::execute_move(args).await,
        Commands::MoveModule(args) => moves::execute_move_module(args),
        Commands::Rename(args) => rename::execute_rename(args).await,
        Commands::Doctor(args) => doctor::execute_doctor(args).await,
        Commands::Guide(args) => guide::execute_guide(args),
        Commands::Completions(args) => execute_completions(args),
        Commands::Man => execute_man(),
    }
}

fn execute_completions(args: CompletionsArgs) -> Result<(), CliError> {
    let mut command = Cli::command();
    let command_name = command.get_name().to_string();
    generate(args.shell, &mut command, command_name, &mut io::stdout());
    Ok(())
}

fn execute_man() -> Result<(), CliError> {
    let command = Cli::command();
    let mut buffer = Vec::new();
    Man::new(command)
        .render(&mut buffer)
        .map_err(|error| CliError {
            json: false,
            error: error.into(),
        })?;

    io::stdout().write_all(&buffer).map_err(|error| CliError {
        json: false,
        error: error.into(),
    })?;

    Ok(())
}

fn render_error(err: &CliError) {
    if err.json {
        let payload = ErrorOutput {
            status: "error",
            error: &format!("{:#}", err.error),
        };
        let _ = write_json(io::stderr(), &payload);
    } else {
        eprintln!("{:#}", err.error);
    }
}

fn write_json<W: Write, T: Serialize>(mut writer: W, value: &T) -> Result<()> {
    serde_json::to_writer_pretty(&mut writer, value)?;
    writer.write_all(b"\n")?;
    writer.flush()?;
    Ok(())
}
