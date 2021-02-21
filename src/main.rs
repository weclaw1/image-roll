extern crate gtk;
extern crate gio;

use gtk::{Builder, prelude::*};
use gio::prelude::*;

use gtk::{Application, ApplicationWindow};

use std::env::args;

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("image_roll_ui.glade");
    let builder = Builder::from_string(glade_src);

    let window: ApplicationWindow = builder.get_object("main_window").expect("Couldn't get main_window");
    window.set_application(Some(application));
    window.show_all();
}

fn main() {
    let application = Application::new(
        Some("com.github.weclaw1.image_roll"),
        Default::default(),
    ).expect("Failed to initialize GTK application");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
