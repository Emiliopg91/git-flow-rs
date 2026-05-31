mod logic;

use std::env;

use git_flow_rs_core::git::GitWrapper;
use slint::CloseRequestResponse;

use crate::logic::{
    select_working_directory, spawn_finish_flow, spawn_refresh_items, spawn_start_flow,
    validate_name,
};

slint::include_modules!();

#[tokio::main]
async fn main() {
    unsafe {
        std::env::remove_var("WAYLAND_DISPLAY");
    }

    let app = App::new().unwrap();

    select_working_directory(app.as_weak().clone()).await;

    app.set_dirty(GitWrapper::has_changes().await.unwrap());

    spawn_refresh_items(app.as_weak().clone(), app.get_category());
    app.on_refresh_items({
        let weak = app.as_weak();

        move |category| {
            let weak = weak.clone();
            spawn_refresh_items(weak, category);
        }
    });

    app.on_start_flow({
        let weak = app.as_weak();
        move |category, name| {
            let weak = weak.clone();
            spawn_start_flow(weak, category, name.to_string());
        }
    });

    app.on_finish_flow({
        let weak = app.as_weak();
        move |category, name| {
            let weak = weak.clone();
            spawn_finish_flow(weak, category, name.to_string());
        }
    });

    app.on_validate_name(move |name| validate_name(name.to_string()));

    let app_weak = app.as_weak();
    app.window().on_close_requested(move || {
        if let Some(app) = app_weak.upgrade() {
            if app.get_show_console_dialog() && !app.get_console_finished() {
                return CloseRequestResponse::KeepWindowShown;
            }
        }

        CloseRequestResponse::HideWindow
    });

    app.run().unwrap();
}
