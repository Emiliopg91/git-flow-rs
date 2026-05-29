use std::{
    env,
    path::Path,
    process::exit,
    sync::{Arc, mpsc},
};

use fwkarq::logger::{Logger, provider::Provider};
use git_flow_rs::{
    git::GitWrapper,
    logic::{
        bugfix::bugfix_start, feature::feature_start, hotfix::hotfix_start, release::release_start,
    },
};
use rfd::AsyncFileDialog;
use slint::{ComponentHandle, SharedString, Weak};

use crate::App;

pub fn get_logger() -> Arc<Logger> {
    Provider::get_logger("App")
}

pub async fn select_working_directory() {
    let mut repo_path = None;
    let args: Vec<String> = std::env::args().collect();
    let logger = get_logger();

    if args.len() == 1 {
        logger.info("Showing repository folder selector...");
        loop {
            let selection = AsyncFileDialog::new()
                .set_directory(env::current_dir().unwrap())
                .set_can_create_directories(false)
                .pick_folder()
                .await;
            match selection {
                Some(folder) => {
                    logger.info(format!("Selected {}", folder.path().display()));
                    let dir = folder.path();
                    if verify_git_repository(dir).await {
                        let _ = env::set_current_dir(dir);
                        repo_path = Some(dir.to_path_buf());
                        break;
                    }
                    logger.error("Folder is not a repository");
                }
                _ => {
                    break;
                }
            }
        }
    } else {
        let selection = args.get(1).unwrap().as_str();
        let dir = Path::new(selection);
        logger.info(format!("Selected {} by argument", dir.display()));
        if verify_git_repository(dir).await {
            let _ = env::set_current_dir(dir);
            repo_path = Some(dir.to_path_buf());
        } else {
            logger.error(format!("Folder is not a repository {}", dir.display()));
        }
    }

    if repo_path.is_none() {
        logger.error("No git repository selected");
        exit(1);
    }
}

pub async fn verify_git_repository<P>(dir: P) -> bool
where
    P: AsRef<Path>,
{
    GitWrapper::check_if_repository(dir).await
}

pub fn spawn_refresh_items(weak: Weak<App>, category: i32) {
    tokio::spawn(async move {
        let logger = get_logger();

        let res = match category {
            0 => GitWrapper::get_features().await,
            1 => GitWrapper::get_releases().await,
            2 => GitWrapper::get_bugfixes().await,
            3 => GitWrapper::get_hotfixes().await,
            _ => unreachable!(),
        };
        match res {
            Ok(branches) => {
                if let Err(e) = weak.upgrade_in_event_loop(move |app| {
                    let model = slint::ModelRc::new(slint::VecModel::from(
                        branches
                            .into_iter()
                            .map(slint::SharedString::from)
                            .collect::<Vec<_>>(),
                    ));
                    app.set_items(model);
                }) {
                    logger.error(format!("UI error: {}", e));
                }
            }

            Err(e) => {
                logger.error(format!("Error fetching branches: {}", e));
            }
        }
    });
}

fn spawn_creation_process(app: Weak<App>, category: i32, name: String) {
    let (tx, rx) = mpsc::channel();

    tokio::spawn(async move {
        let res = match category {
            0 => feature_start(&name, tx.clone()).await,
            1 => release_start(&name, tx.clone()).await,
            2 => bugfix_start(&name, tx.clone()).await,
            3 => hotfix_start(&name, tx.clone()).await,
            _ => unreachable!(),
        };

        if let Err(e) = res {
            let _ = tx.send(e.to_string());
        }

        drop(tx);
    });

    tokio::spawn(async move {
        let mut lines: Vec<String> = Vec::new();
        let logger = get_logger();

        while let Ok(msg) = rx.recv() {
            logger.info(&msg);
            lines.push(msg);
            let text = lines.join("\n");
            app.upgrade_in_event_loop(|app| {
                app.set_console_text(SharedString::from(text));
            })
            .unwrap();
        }

        app.upgrade_in_event_loop(|app| {
            app.set_console_finished(true);
        })
        .unwrap();

        spawn_refresh_items(app, category);
    });
}

pub fn spawn_validate_name(weak: Weak<App>, category: i32, name: String) {
    tokio::spawn(async move {
        let branches = match category {
            0 => GitWrapper::get_features().await,
            1 => GitWrapper::get_releases().await,
            2 => GitWrapper::get_bugfixes().await,
            3 => GitWrapper::get_hotfixes().await,
            _ => unreachable!(),
        };

        let mut found = false;
        if let Ok(branches) = branches
            && branches.contains(&name)
        {
            found = true;
        }

        let branches = match category {
            0 => GitWrapper::get_remote_features().await,
            1 => GitWrapper::get_remote_releases().await,
            2 => GitWrapper::get_remote_bugfixes().await,
            3 => GitWrapper::get_remote_hotfixes().await,
            _ => unreachable!(),
        };
        if let Ok(branches) = branches
            && branches.contains(&name)
        {
            found = true;
        }

        weak.upgrade_in_event_loop(move |app| {
            if found {
                app.set_create_err(SharedString::from("Already exists"));
            } else {
                app.set_show_start_dialog(false);
                app.set_show_console_dialog(true);
                spawn_creation_process(app.as_weak(), category, name);
            }
        })
        .unwrap();
    });
}
