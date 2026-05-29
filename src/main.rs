mod logic;

use std::env;

use crate::logic::{select_working_directory, spawn_refresh_items, spawn_validate_name};

slint::include_modules!();

#[tokio::main]
async fn main() {
    unsafe {
        std::env::remove_var("WAYLAND_DISPLAY");
    }

    select_working_directory().await;

    let app = App::new().unwrap();

    spawn_refresh_items(app.as_weak().clone(), app.get_category());
    app.on_refresh_items({
        let weak = app.as_weak();

        move |category| {
            let weak = weak.clone();
            spawn_refresh_items(weak, category);
        }
    });

    app.on_validate_name({
        let weak = app.as_weak();
        move |category, name| {
            let weak = weak.clone();
            spawn_validate_name(weak, category, name.to_string());
        }
    });

    app.run().unwrap();
}
