extern crate gtk;
extern crate gio;

mod image;
mod file_list;

use gtk::{Builder, prelude::*};
use gio::{Cancellable, FileMonitorFlags, prelude::*};

use gtk::{Application, ApplicationWindow};

use std::{cell::RefCell, env::args, rc::Rc};

use file_list::FileList;

fn load_image(file: Option<&gio::File>, image_widget: &gtk::Image, current_image: Rc<RefCell<Option<image::Image>>>) {
    if let Some(file) = file {
        let image = image::Image::load_from_path(file.get_path().unwrap());
        image_widget.set_from_pixbuf(Some(image.image_buffer()));
        current_image.replace(Some(image));
    } else {
        image_widget.set_from_pixbuf(None);
        current_image.replace(None);
    }
}

fn build_ui(application: &gtk::Application) {
    let glade_src = include_str!("resources/image_roll_ui.glade");
    let builder = Builder::from_string(glade_src);

    let window: ApplicationWindow = builder.get_object("main_window").expect("Couldn't get main_window");
    window.set_application(Some(application));

    let open_menu_button: gtk::Button = builder.get_object("open_menu_button").expect("Couldn't get open_menu_button");

    let image_widget: gtk::Image = builder.get_object("image_widget").expect("Couldn't get image_widget");

    let popover_menu: gtk::PopoverMenu = builder.get_object("popover_menu").expect("Couldn't get popover_menu");

    let next_button: gtk::Button = builder.get_object("next_button").expect("Couldn't get next_button");
    let previous_button: gtk::Button = builder.get_object("previous_button").expect("Couldn't get previous_button");

    let current_image: Rc<RefCell<Option<image::Image>>> = Rc::new(RefCell::new(None));

    let file_list: Rc<RefCell<FileList>> = Rc::new(RefCell::new(FileList::new(None)));

    let current_folder_monitor: Rc<RefCell<Option<gio::FileMonitor>>> = Rc::new(RefCell::new(None));

    open_menu_button.connect_clicked(glib::clone!(@strong window, @strong popover_menu, @strong file_list, @strong image_widget, @strong current_image, @strong next_button, @strong previous_button => move |_| {
        popover_menu.popdown();
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Open file"),
            Some(&window),
            gtk::FileChooserAction::Open,
        );

        file_chooser.add_buttons(&[
            ("Open", gtk::ResponseType::Ok),
            ("Cancel", gtk::ResponseType::Cancel),
        ]);
        file_chooser.connect_response(glib::clone!(@strong image_widget, @strong file_list, @strong current_folder_monitor, @strong current_image, @strong next_button, @strong previous_button => move |file_chooser, response| {
            if response == gtk::ResponseType::Ok {
                let file = file_chooser.get_file().expect("Couldn't get file");
                load_image(Some(&file), &image_widget, current_image.clone());

                let new_file_list = FileList::new(Some(file));
                let buttons_active = new_file_list.len() > 1;

                next_button.set_sensitive(buttons_active);
                previous_button.set_sensitive(buttons_active);

                file_list.replace(new_file_list);
                let folder_monitor = file_list.borrow_mut().current_folder().unwrap().monitor_directory::<Cancellable>(FileMonitorFlags::NONE, None).expect("Couldn't get monitor for directory");
                folder_monitor.connect_changed(glib::clone!(@strong file_list, @strong image_widget, @strong current_image, @strong next_button, @strong previous_button => move |_, _, _, _| {
                    let mut file_list = file_list.borrow_mut();

                    file_list.refresh();
                    let buttons_active = file_list.len() > 1;

                    next_button.set_sensitive(buttons_active);
                    previous_button.set_sensitive(buttons_active);

                    load_image(file_list.current_file(), &image_widget, current_image.clone());
                }));
                current_folder_monitor.replace(Some(folder_monitor));
            }
            file_chooser.close();
        }));
        file_chooser.show_all();
    }));

    next_button.connect_clicked(glib::clone!(@strong file_list, @strong image_widget, @strong current_image => move |_| {
        let mut file_list = file_list.borrow_mut();
        file_list.next();

        load_image(file_list.current_file(), &image_widget, current_image.clone());
    }));

    previous_button.connect_clicked(glib::clone!(@strong file_list, @strong image_widget, @strong current_image => move |_| {
        let mut file_list = file_list.borrow_mut();
        file_list.previous();

        load_image(file_list.current_file(), &image_widget, current_image.clone());
    }));

    window.show_all();
}

fn main() {
    let application = Application::new(
        Some("com.github.weclaw1.ImageRoll"),
        Default::default(),
    ).expect("Failed to initialize GTK application");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
