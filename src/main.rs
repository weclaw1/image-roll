use std::env::args;

use gio::{prelude::ApplicationExtManual, ApplicationExt};
use gtk::Application;

extern crate gio;
extern crate gtk;

mod file_list;
mod image;
mod image_list;
mod image_operation;
mod settings;
mod ui;

fn main() {
    let application = Application::new(Some("com.github.weclaw1.ImageRoll"), Default::default())
        .expect("Failed to initialize GTK application");

    application.connect_activate(|app| {
        ui::build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
