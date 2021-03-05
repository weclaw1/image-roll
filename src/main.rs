extern crate gio;
extern crate gtk;

mod file_list;
mod image;
mod settings;

use gio::{prelude::*, Cancellable, FileMonitorFlags};
use gtk::{prelude::*, Builder};

use gtk::{Application, ApplicationWindow};

use std::{cell::RefCell, env::args, rc::Rc};

use file_list::FileList;
use image::PreviewSize;
use settings::Settings;

fn load_image(
    settings: Rc<RefCell<Settings>>,
    file: Option<&gio::File>,
    image_widget: &gtk::Image,
    current_image: Rc<RefCell<Option<image::Image>>>,
) {
    if let Some(file) = file {
        let mut image = image::Image::load_from_path(file.get_path().unwrap());
        image.create_preview_image_buffer(settings.borrow().scale());
        image_widget.set_from_pixbuf(Some(image.preview_image_buffer()));
        current_image.replace(Some(image));
    } else {
        image_widget.set_from_pixbuf(None);
        current_image.replace(None);
    }
}

fn build_ui(application: &gtk::Application) {
    let bytes = glib::Bytes::from_static(include_bytes!("resources/resources.gresource"));
    let resources = gio::Resource::from_data(&bytes).unwrap();
    gio::resources_register(&resources);

    let builder = Builder::from_resource("/com/github/weclaw1/image_roll/image_roll_ui.glade");

    let window: ApplicationWindow = builder
        .get_object("main_window")
        .expect("Couldn't get main_window");
    window.set_application(Some(application));

    let open_menu_button: gtk::Button = builder
        .get_object("open_menu_button")
        .expect("Couldn't get open_menu_button");

    let image_widget: gtk::Image = builder
        .get_object("image_widget")
        .expect("Couldn't get image_widget");

    let popover_menu: gtk::PopoverMenu = builder
        .get_object("popover_menu")
        .expect("Couldn't get popover_menu");

    let next_button: gtk::Button = builder
        .get_object("next_button")
        .expect("Couldn't get next_button");
    let previous_button: gtk::Button = builder
        .get_object("previous_button")
        .expect("Couldn't get previous_button");

    let preview_smaller_button: gtk::Button = builder
        .get_object("preview_smaller_button")
        .expect("Couldn't get preview_smaller_button");
    let preview_larger_button: gtk::Button = builder
        .get_object("preview_larger_button")
        .expect("Couldn't get preview_larger_button");

    let image_viewport: gtk::Viewport = builder
        .get_object("image_viewport")
        .expect("Couldn't get image_viewport");

    let preview_size_combobox: gtk::ComboBoxText = builder
        .get_object("preview_size_combobox")
        .expect("Couldn't get preview_size_combobox");

    let settings: Rc<RefCell<Settings>> =
        Rc::new(RefCell::new(Settings::new(PreviewSize::BestFit(
            image_viewport.get_allocation().width,
            image_viewport.get_allocation().height,
        ))));

    let current_image: Rc<RefCell<Option<image::Image>>> = Rc::new(RefCell::new(None));

    let file_list: Rc<RefCell<FileList>> = Rc::new(RefCell::new(FileList::new(None)));

    let current_folder_monitor: Rc<RefCell<Option<gio::FileMonitor>>> = Rc::new(RefCell::new(None));

    open_menu_button.connect_clicked(glib::clone!(@strong window, @strong popover_menu, @strong file_list, @strong image_widget, @strong current_image, @strong next_button, @strong previous_button, @strong settings => move |_| {
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
        file_chooser.connect_response(glib::clone!(@strong image_widget, @strong file_list, @strong current_folder_monitor, @strong current_image, @strong next_button, @strong previous_button, @strong settings => move |file_chooser, response| {
            if response == gtk::ResponseType::Ok {
                let file = file_chooser.get_file().expect("Couldn't get file");
                load_image(settings.clone(), Some(&file), &image_widget, current_image.clone());

                let new_file_list = FileList::new(Some(file));
                let buttons_active = new_file_list.len() > 1;

                next_button.set_sensitive(buttons_active);
                previous_button.set_sensitive(buttons_active);

                file_list.replace(new_file_list);
                let folder_monitor = file_list.borrow().current_folder().unwrap().monitor_directory::<Cancellable>(FileMonitorFlags::NONE, None).expect("Couldn't get monitor for directory");
                folder_monitor.connect_changed(glib::clone!(@strong file_list, @strong image_widget, @strong current_image, @strong next_button, @strong previous_button, @strong settings => move |_, _, _, _| {
                    let mut file_list = file_list.borrow_mut();

                    file_list.refresh();
                    let buttons_active = file_list.len() > 1;

                    next_button.set_sensitive(buttons_active);
                    previous_button.set_sensitive(buttons_active);

                    load_image(settings.clone(), file_list.current_file(), &image_widget, current_image.clone());
                }));
                current_folder_monitor.replace(Some(folder_monitor));
            }
            file_chooser.close();
        }));
        file_chooser.show_all();
    }));

    next_button.connect_clicked(
        glib::clone!(@strong settings, @strong file_list, @strong image_widget, @strong current_image => move |_| {
            let mut file_list = file_list.borrow_mut();
            file_list.next();

            load_image(settings.clone(), file_list.current_file(), &image_widget, current_image.clone());
        }),
    );

    previous_button.connect_clicked(
        glib::clone!(@strong settings, @strong file_list, @strong image_widget, @strong current_image => move |_| {
            let mut file_list = file_list.borrow_mut();
            file_list.previous();

            load_image(settings.clone(), file_list.current_file(), &image_widget, current_image.clone());
        }),
    );

    image_viewport.connect_size_allocate(glib::clone!(@strong settings, @strong image_widget, @strong current_image => move |_, allocation| {
        let mut settings = settings.borrow_mut();
        if let PreviewSize::BestFit(_, _) = settings.scale() {
            let new_scale = PreviewSize::BestFit(allocation.width, allocation.height);
            settings.set_scale(new_scale);
            if let Some(image) = current_image.borrow_mut().as_mut() {
                image.create_preview_image_buffer(new_scale);
                image_widget.set_from_pixbuf(Some(image.preview_image_buffer()));
            }
        }
    }));

    preview_size_combobox.connect_changed(glib::clone!(@strong settings, @strong image_widget, @strong current_image, @strong image_viewport, @strong preview_larger_button, @strong preview_smaller_button => move |preview_size_combobox| {
        let mut settings = settings.borrow_mut();
        let mut scale = PreviewSize::from(preview_size_combobox.get_active_id().unwrap().as_str());
        if let PreviewSize::BestFit(_, _) = scale {
            let viewport_allocation = image_viewport.get_allocation();
            scale = PreviewSize::BestFit(viewport_allocation.width, viewport_allocation.height);
        }
        preview_smaller_button.set_sensitive(scale.can_be_smaller());
        preview_larger_button.set_sensitive(scale.can_be_larger());
        settings.set_scale(scale);
        if let Some(image) = current_image.borrow_mut().as_mut() {
            image.create_preview_image_buffer(scale);
            image_widget.set_from_pixbuf(Some(image.preview_image_buffer()));
        }
    }));

    preview_smaller_button.connect_clicked(
        glib::clone!(@strong settings, @strong preview_size_combobox => move |_| {
            let new_scale = {
                let mut settings = settings.borrow_mut();
                let current_scale = settings.scale();
                let new_scale = current_scale.smaller();
                settings.set_scale(new_scale);
                new_scale
            };
            preview_size_combobox.set_active_id(Some(String::from(new_scale).as_ref()));
        }),
    );

    preview_larger_button.connect_clicked(
        glib::clone!(@strong settings, @strong preview_size_combobox => move |_| {
            let new_scale = {
                let mut settings = settings.borrow_mut();
                let current_scale = settings.scale();
                let new_scale = current_scale.larger();
                settings.set_scale(new_scale);
                new_scale
            };
            preview_size_combobox.set_active_id(Some(String::from(new_scale).as_ref()));
        }),
    );

    window.show_all();
}

fn main() {
    let application = Application::new(Some("com.github.weclaw1.ImageRoll"), Default::default())
        .expect("Failed to initialize GTK application");

    application.connect_activate(|app| {
        build_ui(app);
    });

    application.run(&args().collect::<Vec<_>>());
}
