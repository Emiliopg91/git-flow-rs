mod logic;
use std::process::exit;

use slint::CloseRequestResponse;
slint::include_modules!();

use crate::gui::logic::{
    select_working_directory, spawn_create_develop_branch, spawn_finish_flow, spawn_refresh_items,
    spawn_start_flow, validate_name,
};

pub async fn gui_fn(path: Option<String>) {
    unsafe {
        std::env::remove_var("WAYLAND_DISPLAY");
    }

    let app = App::new().unwrap();

    select_working_directory(app.as_weak().clone(), path).await;

    spawn_refresh_items(app.as_weak().clone(), app.get_category());
    app.on_refresh_items({
        let weak = app.as_weak();

        move |category| {
            let weak = weak.clone();
            spawn_refresh_items(weak, category);
        }
    });

    app.on_create_develop_branch({
        let weak = app.as_weak();
        move || {
            let weak = weak.clone();
            spawn_create_develop_branch(weak);
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

    app.on_quit_application(|| {
        exit(2);
    });

    app.run().unwrap();
}
