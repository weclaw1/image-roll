use app::App;
use gtk::{gio::ApplicationFlags, prelude::*, Application};

#[macro_use]
extern crate log;

mod app;
mod file_list;
mod image;
mod image_list;
mod image_operation;
mod settings;
mod ui;

#[cfg(test)]
mod test_utils;

fn main() {
    env_logger::init();

    let application = Application::new(
        Some("com.github.weclaw1.ImageRoll"),
        ApplicationFlags::HANDLES_OPEN | ApplicationFlags::NON_UNIQUE,
    );

    application.connect_activate(|app| {
        App::create(app, None);
    });

    application.connect_open(move |app, files, _| {
        App::create(app, Some(&files[0]));
    });

    application.run();
}
