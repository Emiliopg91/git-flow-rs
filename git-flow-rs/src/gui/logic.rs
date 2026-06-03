use std::{env, path::Path, process::exit};

use fwkarq::*;
use git_flow_rs_core::{
    git::GitWrapper,
    logic::{
        bugfix::{bugfix_finish, bugfix_start},
        feature::{feature_finish, feature_start},
        hotfix::{hotfix_finish, hotfix_start},
        release::{release_finish, release_start},
    },
};
use notify_rust::Notification;
use regex::Regex;
use rfd::AsyncFileDialog;
use slint::{ComponentHandle, SharedString, Weak};
use tokio::sync::mpsc;

use crate::gui::App;

pub async fn select_working_directory(weak: Weak<App>, path: Option<String>) {
    let mut repo_path = None;

    if let Some(selection) = path {
        let dir = std::fs::canonicalize(&selection).unwrap();
        let folder_2 = dir.to_path_buf();

        weak.upgrade_in_event_loop(move |app| {
            app.set_repository(SharedString::from(folder_2.display().to_string()));
        })
        .unwrap();

        info!("App", "Selected {} by argument", dir.display());

        if verify_git_repository(&dir).await {
            let _ = env::set_current_dir(&dir);
            repo_path = Some(dir.to_path_buf());
        } else {
            error!("App", "Folder is not a repository {}", dir.display());
        }
    } else {
        info!("App", "Showing repository folder selector...");
        loop {
            let selection = AsyncFileDialog::new()
                .set_directory(env::current_dir().unwrap())
                .set_can_create_directories(false)
                .pick_folder()
                .await;
            match selection {
                Some(folder) => {
                    if folder.path().display().to_string().is_empty() {
                        continue;
                    }
                    let folder_2 = folder.clone();
                    weak.upgrade_in_event_loop(move |app| {
                        app.set_repository(SharedString::from(
                            folder_2.path().display().to_string(),
                        ));
                    })
                    .unwrap();
                    info!("App", "Selected {}", folder.path().display());
                    let dir = folder.path();
                    if verify_git_repository(dir).await {
                        let _ = env::set_current_dir(dir);
                        repo_path = Some(dir.to_path_buf());
                        break;
                    }
                    error!("App", "Folder is not a repository");
                }
                _ => {
                    error!("App", "No folder selected");
                    exit(1);
                }
            }
        }
    }

    if repo_path.is_none() {
        weak.upgrade_in_event_loop(|app| {
            app.set_not_a_repository(true);
        })
        .unwrap()
    } else {
        let _ = Notification::new()
            .summary("Refreshing repository")
            .body("Please wait while refreshing the repository. This may lasts many seconds.")
            .icon("git-flow-gui")
            .timeout(3000)
            .show_async()
            .await;
        let _ = GitWrapper::fetch(true, true).await;
        let has_changes = GitWrapper::has_changes().await.unwrap();
        let develop_exists = GitWrapper::get_branches()
            .await
            .unwrap_or_default()
            .contains(&"develop".to_string())
            || GitWrapper::get_remote_branches()
                .await
                .unwrap_or_default()
                .contains(&"develop".to_string());

        weak.upgrade_in_event_loop(move |app| {
            app.set_dirty(has_changes);
            app.set_show_develop_dialog(!develop_exists);
        })
        .unwrap()
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
                    error!("App", "UI error: {}", e);
                }
            }

            Err(e) => {
                error!("App", "Error fetching branches: {}", e);
            }
        }
    });
}

fn spawn_creation_process(app: Weak<App>, category: i32, name: String) {
    let (tx, mut rx) = mpsc::channel(100);

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

        while let Some(msg) = rx.recv().await {
            info!("App", "{}", &msg);
            lines.push(msg);
            let text = lines.join("\n");
            app.upgrade_in_event_loop(|app| {
                app.set_console_text(SharedString::from(text));
            })
            .unwrap();
        }

        app.upgrade_in_event_loop(|app| {
            app.set_console_finished(true);
            app.set_item(-1);
        })
        .unwrap();

        spawn_refresh_items(app, category);
    });
}

pub fn spawn_create_develop_branch(weak: Weak<App>) {
    weak.upgrade_in_event_loop(move |app| {
        app.set_show_console_dialog(true);
        app.set_console_finished(false);
    })
    .unwrap();
    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let local_exists = GitWrapper::get_branches()
            .await
            .unwrap_or_default()
            .contains(&"develop".to_string());

        let remote_exists = GitWrapper::get_remote_branches()
            .await
            .unwrap_or_default()
            .contains(&"develop".to_string());

        if !local_exists {
            if remote_exists {
                let _ = tx.send("Checking out the develop branch...".to_string());
                if let Err(e) = GitWrapper::checkout("develop").await {
                    let _ = tx.send(format!("Failed to check out the develop branch: {}", e));
                    return;
                }
                let _ = tx.send("  Checked out develop branch".to_string());
            } else {
                let _ = tx.send("Creating local develop branch...".to_string());
                if let Err(e) = GitWrapper::create_branch("develop").await {
                    let _ = tx.send(format!("Error creating develop branch: {}", e));
                    return;
                }
                let _ = tx.send("  Local develop branch created".to_string());
            }
        }

        if !remote_exists {
            let _ = tx.send("Pushing to remote...".to_string());

            if let Err(e) = GitWrapper::push().await {
                let _ = tx.send(format!("Error pushing develop to remote: {}", e));
            } else {
                let _ = tx.send("  Remote branch develop pushed succesfully".to_string());
            }
        }

        let _ = tx.send("Process finished succesfully".into());
        drop(tx);
    });

    tokio::spawn(async move {
        let mut lines = Vec::new();
        while let Some(msg) = rx.recv().await {
            info!("App", "{}", msg);
            lines.push(msg);
            let text = lines.join("\n");
            weak.upgrade_in_event_loop(|app| {
                app.set_console_text(SharedString::from(text));
            })
            .unwrap();
        }

        weak.upgrade_in_event_loop(|app| {
            app.set_console_finished(true);
            app.set_show_develop_dialog(false)
        })
    });
}

pub fn spawn_start_flow(weak: Weak<App>, category: i32, name: String) {
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
                app.set_console_finished(false);
                spawn_creation_process(app.as_weak(), category, name);
            }
        })
        .unwrap();
    });
}

pub fn validate_name(name: String) -> bool {
    let re = Regex::new(r"^[a-zA-Z0-9_\-]+$").unwrap();
    re.is_match(&name)
}

pub fn spawn_finish_flow(weak: Weak<App>, category: i32, name: String) {
    weak.upgrade_in_event_loop(|app| {
        app.set_show_finish_dialog(false);
        app.set_show_console_dialog(true);
        app.set_console_finished(false);
    })
    .unwrap();

    let (tx, mut rx) = mpsc::channel(100);

    tokio::spawn(async move {
        let res = match category {
            0 => feature_finish(&name, tx.clone()).await,
            1 => release_finish(&name, tx.clone()).await,
            2 => bugfix_finish(&name, tx.clone()).await,
            3 => hotfix_finish(&name, tx.clone()).await,
            _ => unreachable!(),
        };

        if let Err(e) = res {
            let _ = tx.send(e.to_string());
        }

        drop(tx);
    });

    tokio::spawn(async move {
        let mut lines: Vec<String> = Vec::new();

        while let Some(msg) = rx.recv().await {
            info!("App", "{}", &msg);
            lines.push(msg);
            let text = lines.join("\n");
            weak.upgrade_in_event_loop(|app| {
                app.set_console_text(SharedString::from(text));
            })
            .unwrap();
        }

        weak.upgrade_in_event_loop(|app| {
            app.set_console_finished(true);
            app.set_item(-1);
        })
        .unwrap();

        spawn_refresh_items(weak, category);
    });
}
