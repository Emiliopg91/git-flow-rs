use std::{env, error::Error, io::stdout, process::exit, sync::mpsc};

use clap_complete::{aot::Bash, generate};

use clap::{CommandFactory, Parser, Subcommand, ValueEnum};
use fwkarq::logger::{level::Level, provider::Provider};
use git_flow_rs_core::logic::{
    bugfix::{bugfix_finish, bugfix_start},
    feature::{feature_finish, feature_start},
    hotfix::{hotfix_finish, hotfix_start},
    release::{release_finish, release_start},
};

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
}

#[derive(Parser, Debug)]
struct CliArguments {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    if env::var("GFT_GEN_COMPLETION").is_ok() {
        let mut cmd = CliArguments::command();
        generate(Bash, &mut cmd, "git-flow", &mut stdout());
    } else {
        let cli: CliArguments = CliArguments::parse();
        let command = cli.command;

        Provider::set_level(Level::WARNING);

        let (tx, rx) = mpsc::channel::<String>();

        let worker = tokio::spawn(async move {
            let result = match command {
                Commands::Feature { name, action } => match action {
                    Action::Start => feature_start(&name, tx.clone()).await,
                    Action::Finish => feature_finish(&name, tx.clone()).await,
                },
                Commands::Release { name, action } => match action {
                    Action::Start => release_start(&name, tx.clone()).await,
                    Action::Finish => release_finish(&name, tx.clone()).await,
                },
                Commands::Hotfix { name, action } => match action {
                    Action::Start => hotfix_start(&name, tx.clone()).await,
                    Action::Finish => hotfix_finish(&name, tx.clone()).await,
                },
                Commands::Bugfix { name, action } => match action {
                    Action::Start => bugfix_start(&name, tx.clone()).await,
                    Action::Finish => bugfix_finish(&name, tx.clone()).await,
                },
            };
            if let Err(e) = result {
                tx.send(format!("{}", e)).unwrap();
                exit(1);
            }
        });

        loop {
            while let Ok(msg) = rx.try_recv() {
                println!("{}", msg);
            }
            if worker.is_finished() {
                break;
            }
        }
    }
    Ok(())
}
