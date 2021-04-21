use std::env::args;

use app::App;
use gio::{prelude::ApplicationExtManual, ApplicationExt};
use gtk::Application;

extern crate gio;
extern crate gtk;

#[macro_use]
extern crate log;

mod app;
mod file_list;
mod image;
mod image_list;
mod image_operation;
mod settings;
mod ui;

fn main() {
    env_logger::init();

    let application = Application::new(Some("com.github.weclaw1.ImageRoll"), Default::default())
        .expect("Failed to initialize GTK application");

    application.connect_activate(|app| {
        App::new(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
