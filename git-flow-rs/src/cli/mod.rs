use std::process::exit;

use fwkarq::logger::{level::Level, provider::Provider};
use git_flow_rs_core::logic::{
    bugfix::{bugfix_finish, bugfix_start},
    feature::{feature_finish, feature_start},
    hotfix::{hotfix_finish, hotfix_start},
    release::{release_finish, release_start},
};
use tokio::sync::mpsc;

use crate::{Action, Commands};

pub async fn cli_fn(command: Commands) {
    Provider::set_level(Level::WARNING);

    let (tx, mut rx) = mpsc::channel::<String>(100);

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
            _ => unreachable!(),
        };
        if let Err(e) = result {
            tx.send(format!("{}", e)).await.unwrap();
            exit(1);
        }
    });

    loop {
        while let Some(msg) = rx.recv().await {
            println!("{}", msg);
        }
        if worker.is_finished() {
            break;
        }
    }
}
