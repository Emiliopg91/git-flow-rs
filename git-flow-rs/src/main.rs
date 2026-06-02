mod cli;
mod gui;

use std::{env, io::stdout};

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use clap_complete::{Shell::Bash, generate};

use crate::{cli::cli_fn, gui::gui_fn};

#[derive(Clone, Debug, ValueEnum, PartialEq)]
enum Action {
    Start,
    Finish,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Feature {
        name: String,
        #[arg(value_enum)]
        action: Action,
    },

    Release {
        name: String,
        #[arg(value_enum)]
        action: Action,
    },

    Hotfix {
        name: String,
        #[arg(value_enum)]
        action: Action,
    },

    Bugfix {
        name: String,
        #[arg(value_enum)]
        action: Action,
    },

    Gui {
        path: Option<String>,
    },
}

#[derive(Parser, Debug)]
struct CliArguments {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() {
    if env::var("GFT_GEN_COMPLETION").is_ok() {
        let mut cmd = CliArguments::command();
        generate(Bash, &mut cmd, "git-flow", &mut stdout());
    } else {
        let cli: CliArguments = CliArguments::parse();
        let command = cli.command;

        match command {
            Commands::Gui { path } => {
                gui_fn(path).await;
            }
            _ => cli_fn(command).await,
        }
    }
}
