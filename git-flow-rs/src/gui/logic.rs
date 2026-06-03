use std::{env, process::exit};

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

        info!("App", "Selected {} by argument", dir.display());
        let _ = env::set_current_dir(&dir);

        if let Ok(rp) = GitWrapper::get_repo_path().await {
            repo_path = Some(rp);
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
                    info!("App", "Selected {}", folder.path().display());

                    let dir = folder.path();
                    let _ = env::set_current_dir(&dir);

                    if let Ok(rp) = GitWrapper::get_repo_path().await {
                        repo_path = Some(rp);
                        break;
                    } else {
                        error!("App", "Folder is not a repository {}", dir.display());
                    }
                }
                _ => {
                    error!("App", "No folder selected");
                    exit(1);
                }
            }
        }
    }

    let mut not_repository_state = false;
    let mut repo_state = "".to_string();
    let mut has_changes_state = false;
    let mut develop_exists_state = true;

    if let Some(repo_path) = repo_path {
        not_repository_state = false;
        repo_state = repo_path.clone();
        info!("App", "Repository root: {}", repo_state);

        env::set_current_dir(repo_path).unwrap();
        if let Ok(origin) = GitWrapper::get_origin().await
            && let Some(mut origin) = origin
        {
            info!("App", "Origin URL: {}", origin);
            origin = origin.to_lowercase();
            if origin.starts_with("http://") || origin.starts_with("https://") {
                error!("App", "Not allowed HTTP/S repositories");
            } else {
                let _ = Notification::new()
                    .summary("Refreshing repository")
                    .body(
                        "Please wait while refreshing the repository. This may lasts many seconds.",
                    )
                    .icon("git-flow-gui")
                    .timeout(3000)
                    .show_async()
                    .await;

                info!("App", "Fetching from remote...");
                let _ = GitWrapper::fetch(true, true).await;

                has_changes_state = GitWrapper::has_changes().await.unwrap();
                info!(
                    "App",
                    "Repository state: {}",
                    if has_changes_state { "dirty" } else { "clean" }
                );

                develop_exists_state = GitWrapper::get_branches()
                    .await
                    .unwrap_or_default()
                    .contains(&"develop".to_string())
                    && GitWrapper::get_remote_branches()
                        .await
                        .unwrap_or_default()
                        .contains(&"develop".to_string());
                info!("App", "Branch develop exists: {}", develop_exists_state);
            }
        }
    }

    weak.upgrade_in_event_loop(move |app| {
        app.set_not_a_repository(not_repository_state);
        app.set_repository(SharedString::from(repo_state));
        app.set_dirty(has_changes_state);
        app.set_show_develop_dialog(!develop_exists_state);
    })
    .unwrap();
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
            let _ = tx.send(e.to_string()).await;
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
                let _ = tx
                    .send("Checking out the develop branch...".to_string())
                    .await;
                if let Err(e) = GitWrapper::checkout("develop").await {
                    let _ = tx.send(format!("Failed to check out the develop branch: {}", e));
                    return;
                }
                let _ = tx.send("  Checked out develop branch".to_string()).await;
            } else {
                let _ = tx
                    .send("Creating local develop branch...".to_string())
                    .await;
                if let Err(e) = GitWrapper::create_branch("develop").await {
                    let _ = tx
                        .send(format!("Error creating develop branch: {}", e))
                        .await;
                    return;
                }
                let _ = tx.send("  Local develop branch created".to_string()).await;
            }
        }

        if !remote_exists {
            let _ = tx.send("Pushing to remote...".to_string()).await;

            if let Err(e) = GitWrapper::push().await {
                let _ = tx
                    .send(format!("Error pushing develop to remote: {}", e))
                    .await;
            } else {
                let _ = tx
                    .send("  Remote branch develop pushed succesfully".to_string())
                    .await;
            }
        }

        let _ = tx.send("Process finished succesfully".into()).await;
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
            let _ = tx.send(e.to_string()).await;
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
