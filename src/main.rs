extern crate gio;
extern crate gtk;

mod file_list;
mod image;
mod image_list;
mod image_operation;
mod settings;

use gdk_pixbuf::PixbufRotation;
use gio::prelude::*;
use gtk::{prelude::*, Builder};

use gtk::{Application, ApplicationWindow};

use std::{cell::RefCell, env::args, path::PathBuf, rc::Rc};

use file_list::FileList;
use image::PreviewSize;
use image_list::ImageList;
use image_operation::{ApplyImageOperation, ImageOperation};
use settings::Settings;

fn load_image(
    settings: &Settings,
    file_path: Option<PathBuf>,
    image_widget: &gtk::Image,
    image_list: &mut ImageList,
) {
    if let Some(file_path) = &file_path {
        let mut image = if let Some(image) = image_list.remove(&file_path) {
            image.reload(file_path)
        } else {
            image::Image::load(file_path)
        };
        image.create_preview_image_buffer(settings.scale());
        image_widget.set_from_pixbuf(image.preview_image_buffer());
        image_list.insert(file_path.clone(), image);
    } else {
        image_widget.set_from_pixbuf(None);
    }
    image_list.set_current_image_path(file_path);
}

fn build_ui(application: &gtk::Application) {
    let bytes = glib::Bytes::from_static(include_bytes!("resources/resources.gresource"));
    let resources = gio::Resource::from_data(&bytes).expect("Couldn't load resources");
    gio::resources_register(&resources);

    let builder = Builder::from_resource("/com/github/weclaw1/image-roll/image-roll_ui.glade");

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

    let image_event_box: gtk::EventBox = builder
        .get_object("image_event_box")
        .expect("Couldn't get image_preview_box");

    let rotate_counterclockwise_button: gtk::Button = builder
        .get_object("rotate_counterclockwise_button")
        .expect("Couldn't get rotate_counterclockwise_button");
    let rotate_clockwise_button: gtk::Button = builder
        .get_object("rotate_clockwise_button")
        .expect("Couldn't get rotate_clockwise_button");

    let settings: Rc<RefCell<Settings>> =
        Rc::new(RefCell::new(Settings::new(PreviewSize::BestFit(
            image_viewport.get_allocation().width,
            image_viewport.get_allocation().height,
        ))));

    let image_list: Rc<RefCell<ImageList>> = Rc::new(RefCell::new(ImageList::new()));

    let file_list: Rc<RefCell<FileList>> = Rc::new(RefCell::new(FileList::new(None)));

    open_menu_button.connect_clicked(glib::clone!(@strong window, @strong popover_menu, @strong file_list, @strong image_widget, @strong image_list, @strong next_button, @strong previous_button, @strong settings => move |_| {
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
        file_chooser.connect_response(glib::clone!(@strong image_widget, @strong file_list, @strong image_list, @strong next_button, @strong previous_button, @strong settings => move |file_chooser, response| {
            if response == gtk::ResponseType::Ok {
                image_list.replace(ImageList::new());
                let file = file_chooser.get_file().expect("Couldn't get file");
                load_image(&settings.borrow(), file.get_path(), &image_widget, &mut image_list.borrow_mut());

                let mut new_file_list = FileList::new(Some(file));
                let buttons_active = new_file_list.len() > 1;

                next_button.set_sensitive(buttons_active);
                previous_button.set_sensitive(buttons_active);

                new_file_list.current_folder_monitor_mut().unwrap().connect_changed(glib::clone!(@strong file_list, @strong image_widget, @strong image_list, @strong next_button, @strong previous_button, @strong settings => move |_, _, _, _| {
                    let mut file_list = file_list.borrow_mut();

                    file_list.refresh();
                    let buttons_active = file_list.len() > 1;

                    next_button.set_sensitive(buttons_active);
                    previous_button.set_sensitive(buttons_active);

                    load_image(&settings.borrow(), file_list.current_file_path(), &image_widget, &mut image_list.borrow_mut());
                }));

                file_list.replace(new_file_list);
            }
            file_chooser.close();
        }));
        file_chooser.show_all();
    }));

    next_button.connect_clicked(
        glib::clone!(@strong settings, @strong file_list, @strong image_widget, @strong image_list => move |_| {
            let mut file_list = file_list.borrow_mut();
            let mut image_list = image_list.borrow_mut();
            image_list.current_image_mut().unwrap().remove_image_buffers();
            file_list.next();
            load_image(&settings.borrow(), file_list.current_file_path(), &image_widget, &mut image_list);
        }),
    );

    previous_button.connect_clicked(
        glib::clone!(@strong settings, @strong file_list, @strong image_widget, @strong image_list => move |_| {
            let mut file_list = file_list.borrow_mut();
            let mut image_list = image_list.borrow_mut();
            image_list.current_image_mut().unwrap().remove_image_buffers();
            file_list.previous();
            load_image(&settings.borrow(), file_list.current_file_path(), &image_widget, &mut image_list);
        }),
    );

    image_viewport.connect_size_allocate(glib::clone!(@strong settings, @strong image_widget, @strong image_list => move |_, allocation| {
        let mut settings = settings.borrow_mut();
        if let PreviewSize::BestFit(_, _) = settings.scale() {
            let new_scale = PreviewSize::BestFit(allocation.width, allocation.height);
            settings.set_scale(new_scale);
            if let Some(image) = image_list.borrow_mut().current_image_mut() {
                image.create_preview_image_buffer(new_scale);
                image_widget.set_from_pixbuf(image.preview_image_buffer());
            }
        }
    }));

    preview_size_combobox.connect_changed(glib::clone!(@strong settings, @strong image_widget, @strong image_list, @strong image_viewport, @strong preview_larger_button, @strong preview_smaller_button => move |preview_size_combobox| {
        let mut settings = settings.borrow_mut();
        let mut scale = PreviewSize::from(preview_size_combobox.get_active_id().unwrap().as_str());
        if let PreviewSize::BestFit(_, _) = scale {
            let viewport_allocation = image_viewport.get_allocation();
            scale = PreviewSize::BestFit(viewport_allocation.width, viewport_allocation.height);
        }
        preview_smaller_button.set_sensitive(scale.can_be_smaller());
        preview_larger_button.set_sensitive(scale.can_be_larger());
        settings.set_scale(scale);
        if let Some(image) = image_list.borrow_mut().current_image_mut() {
            image.create_preview_image_buffer(scale);
            image_widget.set_from_pixbuf(image.preview_image_buffer());
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

    rotate_counterclockwise_button.connect_clicked(glib::clone!(@strong settings, @strong file_list, @strong image_widget, @strong image_list => move |_| {
        let mut image_list = image_list.borrow_mut();
        let current_image_path = file_list.borrow().current_file_path();
        if let Some(mut current_image) = image_list.remove_current_image() {
            current_image = current_image.apply_operation(&ImageOperation::Rotate(PixbufRotation::Counterclockwise));
            current_image.create_preview_image_buffer(settings.borrow().scale());
            image_widget.set_from_pixbuf(current_image.preview_image_buffer());
            image_list.insert(current_image_path.unwrap(), current_image);
        }
    }));

    rotate_clockwise_button.connect_clicked(glib::clone!(@strong settings, @strong file_list, @strong image_widget, @strong image_list => move |_| {
        let mut image_list = image_list.borrow_mut();
        let current_image_path = file_list.borrow().current_file_path();
        if let Some(mut current_image) = image_list.remove_current_image() {
            current_image = current_image.apply_operation(&ImageOperation::Rotate(PixbufRotation::Clockwise));
            current_image.create_preview_image_buffer(settings.borrow().scale());
            image_widget.set_from_pixbuf(current_image.preview_image_buffer());
            image_list.insert(current_image_path.unwrap(), current_image);
        }
    }));

    image_event_box.connect_button_press_event(glib::clone!(@strong image_widget, @strong file_list, @strong image_list => move |image_event_box, button_event| {
        let file_list = file_list.borrow();
        if let Some(file) = file_list.current_file() {
            let image_list = image_list.borrow();
            let current_image = &image_list[&file.get_path().unwrap()];
            let (image_width, image_height) = current_image.preview_image_buffer_size().unwrap();
            let position = button_event.get_position();
            let event_box_allocation = image_event_box.get_allocation();
            let (image_coords_position_x, image_coords_position_y) = (position.0 as i32 - ((event_box_allocation.width - image_width) / 2), position.1 as i32 - ((event_box_allocation.height - image_height) / 2));
            println!("Allocation: {:?}", event_box_allocation);
            println!("Button pressed at x: {}, y: {}", image_coords_position_x, image_coords_position_y);
        }
        gtk::Inhibit(false)
    }));

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
