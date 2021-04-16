mod widgets;

use gdk::prelude::*;
use gdk_pixbuf::PixbufRotation;
use gio::prelude::*;
use gtk::{prelude::*, Builder};

use widgets::Widgets;

use anyhow::anyhow;

use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
};

use crate::image::{CoordinatesPair, PreviewSize};
use crate::image_list::ImageList;
use crate::image_operation::{ApplyImageOperation, ImageOperation};
use crate::settings::Settings;
use crate::{file_list::FileList, image};

fn load_image(
    widgets: &Widgets,
    settings: &Settings,
    file_path: Option<PathBuf>,
    image_list: &mut ImageList,
) {
    if let Some(file_path) = &file_path {
        let image = if let Some(image) = image_list.remove(&file_path) {
            image.reload(file_path)
        } else {
            image::Image::load(file_path)
        };
        let mut image = match image {
            Ok(image) => image,
            Err(error) => {
                widgets.image_widget().set_from_pixbuf(None);
                display_error(
                    widgets.error_info_bar(),
                    widgets.error_info_bar_text(),
                    error,
                );
                return;
            }
        };
        image.create_preview_image_buffer(settings.scale());
        widgets
            .save_menu_button()
            .set_sensitive(image.has_unsaved_operations());
        widgets
            .image_widget()
            .set_from_pixbuf(image.preview_image_buffer());
        image_list.insert(file_path.clone(), image);
    } else {
        widgets.image_widget().set_from_pixbuf(None);
        widgets.save_menu_button().set_sensitive(false);
    }
    image_list.set_current_image_path(file_path);
}

fn display_error(
    error_info_bar: &gtk::InfoBar,
    error_info_bar_text: &gtk::Label,
    error: anyhow::Error,
) {
    error_info_bar_text.set_text(&format!("ERROR: {:#}", error));
    error_info_bar.set_revealed(true);
}

pub fn build_ui(application: &gtk::Application) {
    let bytes = glib::Bytes::from_static(include_bytes!("resources/resources.gresource"));
    let resources = gio::Resource::from_data(&bytes).expect("Couldn't load resources");
    gio::resources_register(&resources);

    let builder = Builder::from_resource("/com/github/weclaw1/image-roll/image-roll_ui.glade");

    let widgets: Widgets = Widgets::init(builder, application);

    let image_list: Rc<RefCell<ImageList>> = Rc::new(RefCell::new(ImageList::new()));

    let file_list: Rc<RefCell<FileList>> = Rc::new(RefCell::new(FileList::new(None).unwrap()));

    let selection_coords: Rc<Cell<Option<CoordinatesPair>>> = Rc::new(Cell::new(None));

    let settings: Rc<RefCell<Settings>> =
        Rc::new(RefCell::new(Settings::new(PreviewSize::BestFit(
            widgets.image_viewport().get_allocation().width,
            widgets.image_viewport().get_allocation().height,
        ))));

    widgets.open_menu_button().connect_clicked(glib::clone!(@strong widgets, @strong file_list, @strong image_list, @strong settings => move |_| {
        widgets.popover_menu().popdown();
        let file_chooser = gtk::FileChooserDialog::new(
            Some("Open file"),
            Some(widgets.window()),
            gtk::FileChooserAction::Open,
        );

        file_chooser.add_buttons(&[
            ("Open", gtk::ResponseType::Ok),
            ("Cancel", gtk::ResponseType::Cancel),
        ]);
        file_chooser.connect_response(glib::clone!(@strong widgets, @strong file_list, @strong image_list, @strong settings => move |file_chooser, response| {
            file_chooser.close();
            if response == gtk::ResponseType::Ok {
                widgets.error_info_bar().set_revealed(false);
                image_list.replace(ImageList::new());
                let file = if let Some(file) = file_chooser.get_file() {
                    file
                } else {
                    file_chooser.close();
                    display_error(widgets.error_info_bar(), widgets.error_info_bar_text(), anyhow!("Couldn't load file"));
                    return;
                };
                load_image(&widgets, &settings.borrow(), file.get_path(), &mut image_list.borrow_mut());

                let mut new_file_list = match FileList::new(Some(file)) {
                    Ok(file_list) => file_list,
                    Err(error) => {
                        file_chooser.close();
                        display_error(widgets.error_info_bar(), widgets.error_info_bar_text(), error);
                        return;
                    },
                };
                let buttons_active = new_file_list.len() > 1;

                widgets.next_button().set_sensitive(buttons_active);
                widgets.previous_button().set_sensitive(buttons_active);
                widgets.rotate_counterclockwise_button().set_sensitive(buttons_active);
                widgets.rotate_clockwise_button().set_sensitive(buttons_active);
                widgets.crop_button().set_sensitive(buttons_active);
                widgets.resize_button().set_sensitive(buttons_active);

                new_file_list.current_folder_monitor_mut().unwrap().connect_changed(glib::clone!(@strong widgets, @strong file_list, @strong image_list, @strong settings => move |_, _, _, _| {
                    widgets.error_info_bar().set_revealed(false);
                    let mut file_list = file_list.borrow_mut();

                    match file_list.refresh() {
                        Ok(_) => (),
                        Err(error) => {
                            display_error(widgets.error_info_bar(), widgets.error_info_bar_text(), error);
                            return;
                        },
                    };
                    let buttons_active = file_list.len() > 1;

                    widgets.next_button().set_sensitive(buttons_active);
                    widgets.previous_button().set_sensitive(buttons_active);
                    widgets.rotate_counterclockwise_button().set_sensitive(buttons_active);
                    widgets.rotate_clockwise_button().set_sensitive(buttons_active);
                    widgets.crop_button().set_sensitive(buttons_active);
                    widgets.resize_button().set_sensitive(buttons_active);

                    load_image(&widgets, &settings.borrow(), file_list.current_file_path(), &mut image_list.borrow_mut());
                }));

                file_list.replace(new_file_list);
            }
        }));
        file_chooser.show_all();
    }));

    widgets.next_button().connect_clicked(
        glib::clone!(@strong widgets, @strong settings, @strong file_list, @strong image_list => move |_| {
            let mut file_list = file_list.borrow_mut();
            let mut image_list = image_list.borrow_mut();
            image_list.current_image_mut().unwrap().remove_image_buffers();
            match file_list.next() {
                Ok(_) => (),
                Err(error) => {
                    display_error(widgets.error_info_bar(), widgets.error_info_bar_text(), error);
                    return;
                },
            };
            load_image(&widgets, &settings.borrow(), file_list.current_file_path(), &mut image_list);
        }),
    );

    widgets.previous_button().connect_clicked(
        glib::clone!(@strong widgets, @strong settings, @strong file_list, @strong image_list => move |_| {
            let mut file_list = file_list.borrow_mut();
            let mut image_list = image_list.borrow_mut();
            image_list.current_image_mut().unwrap().remove_image_buffers();
            match file_list.previous() {
                Ok(_) => (),
                Err(error) => {
                    display_error(widgets.error_info_bar(), widgets.error_info_bar_text(), error);
                    return;
                },
            };
            load_image(&widgets, &settings.borrow(), file_list.current_file_path(), &mut image_list);
        }),
    );

    widgets.image_viewport().connect_size_allocate(
        glib::clone!(@strong widgets, @strong settings, @strong image_list => move |_, allocation| {
            let mut settings = settings.borrow_mut();
            if let PreviewSize::BestFit(_, _) = settings.scale() {
                let new_scale = PreviewSize::BestFit(allocation.width, allocation.height);
                settings.set_scale(new_scale);
                if let Some(image) = image_list.borrow_mut().current_image_mut() {
                    image.create_preview_image_buffer(new_scale);
                    widgets.image_widget().set_from_pixbuf(image.preview_image_buffer());
                }
            }
        }),
    );

    widgets.preview_size_combobox().connect_changed(glib::clone!(@strong widgets, @strong settings, @strong image_list => move |preview_size_combobox| {
        let mut settings = settings.borrow_mut();
        let mut scale = PreviewSize::from(preview_size_combobox.get_active_id().unwrap().as_str());
        if let PreviewSize::BestFit(_, _) = scale {
            let viewport_allocation = widgets.image_viewport().get_allocation();
            scale = PreviewSize::BestFit(viewport_allocation.width, viewport_allocation.height);
        }
        widgets.preview_smaller_button().set_sensitive(scale.can_be_smaller());
        widgets.preview_larger_button().set_sensitive(scale.can_be_larger());
        settings.set_scale(scale);
        if let Some(image) = image_list.borrow_mut().current_image_mut() {
            image.create_preview_image_buffer(scale);
            widgets.image_widget().set_from_pixbuf(image.preview_image_buffer());
        }
    }));

    widgets.preview_smaller_button().connect_clicked(
        glib::clone!(@strong widgets, @strong settings => move |_| {
            let new_scale = {
                let mut settings = settings.borrow_mut();
                let current_scale = settings.scale();
                let new_scale = current_scale.smaller();
                settings.set_scale(new_scale);
                new_scale
            };
            widgets.preview_size_combobox().set_active_id(Some(String::from(new_scale).as_ref()));
        }),
    );

    widgets.preview_larger_button().connect_clicked(
        glib::clone!(@strong widgets, @strong settings => move |_| {
            let new_scale = {
                let mut settings = settings.borrow_mut();
                let current_scale = settings.scale();
                let new_scale = current_scale.larger();
                settings.set_scale(new_scale);
                new_scale
            };
            widgets.preview_size_combobox().set_active_id(Some(String::from(new_scale).as_ref()));
        }),
    );

    widgets.rotate_counterclockwise_button().connect_clicked(glib::clone!(@strong widgets, @strong settings, @strong file_list, @strong image_list => move |_| {
        let mut image_list = image_list.borrow_mut();
        if let Some(mut current_image) = image_list.remove_current_image() {
            current_image = current_image.apply_operation(&ImageOperation::Rotate(PixbufRotation::Counterclockwise));
            current_image.create_preview_image_buffer(settings.borrow().scale());
            widgets.image_widget().set_from_pixbuf(current_image.preview_image_buffer());
            widgets.save_menu_button().set_sensitive(current_image.has_unsaved_operations());
            image_list.insert(file_list.borrow().current_file_path().unwrap(), current_image);
        }
    }));

    widgets.rotate_clockwise_button().connect_clicked(glib::clone!(@strong widgets, @strong settings, @strong file_list, @strong image_list => move |_| {
        let mut image_list = image_list.borrow_mut();
        if let Some(mut current_image) = image_list.remove_current_image() {
            current_image = current_image.apply_operation(&ImageOperation::Rotate(PixbufRotation::Clockwise));
            current_image.create_preview_image_buffer(settings.borrow().scale());
            widgets.image_widget().set_from_pixbuf(current_image.preview_image_buffer());
            widgets.save_menu_button().set_sensitive(current_image.has_unsaved_operations());
            image_list.insert(file_list.borrow().current_file_path().unwrap(), current_image);
        }
    }));

    widgets
        .image_event_box()
        .set_events(gdk::EventMask::POINTER_MOTION_MASK);

    widgets.image_event_box().connect_button_press_event(glib::clone!(@strong widgets, @strong image_list, @strong selection_coords => move |image_event_box, button_event| {
        if !widgets.crop_button().get_active() {
            return gtk::Inhibit(false)
        }
        let image_list = image_list.borrow();
        if let Some(current_image) = image_list.current_image() {
            let (image_width, image_height) = current_image.preview_image_buffer_size().unwrap();
            let (position_x, position_y) = button_event.get_position();
            let event_box_allocation = image_event_box.get_allocation();
            let (image_coords_position_x, image_coords_position_y) = (position_x as i32 - ((event_box_allocation.width - image_width) / 2), position_y as i32 - ((event_box_allocation.height - image_height) / 2));
            if image_coords_position_x >= 0 && image_coords_position_x < image_width && image_coords_position_y >= 0 && image_coords_position_y < image_height {
                selection_coords.replace(Some(((position_x as i32, position_y as i32), (position_x as i32, position_y as i32))));
                widgets.image_widget().queue_draw();
            }
        }
        gtk::Inhibit(false)
    }));

    widgets.image_event_box().connect_motion_notify_event(glib::clone!(@strong widgets, @strong image_list, @strong selection_coords => move |image_event_box, motion_event| {
        if !widgets.crop_button().get_active() {
            return gtk::Inhibit(false)
        }
        if let Some(((start_position_x, start_position_y),(_, _))) = selection_coords.get() {
            let image_list = image_list.borrow();
            if let Some(current_image) = image_list.current_image() {
                let (image_width, image_height) = current_image.preview_image_buffer_size().unwrap();
                let (position_x, position_y) = motion_event.get_position();
                let event_box_allocation = image_event_box.get_allocation();
                let (image_coords_position_x, image_coords_position_y) = (position_x as i32 - ((event_box_allocation.width - image_width) / 2), position_y as i32 - ((event_box_allocation.height - image_height) / 2));
                if image_coords_position_x >= 0 && image_coords_position_x < image_width && image_coords_position_y >= 0 && image_coords_position_y < image_height {
                    selection_coords.replace(Some(((start_position_x, start_position_y), (position_x as i32, position_y as i32))));
                    widgets.image_widget().queue_draw();
                }
            }
        }
        gtk::Inhibit(false)
    }));

    widgets.image_event_box().connect_button_release_event(glib::clone!(@strong widgets, @strong settings, @strong file_list, @strong image_list, @strong selection_coords => move |image_event_box, _| {
        if !widgets.crop_button().get_active() {
            return gtk::Inhibit(false)
        }
        if let Some(((start_position_x, start_position_y),(end_position_x, end_position_y))) = selection_coords.get() {
            let mut image_list = image_list.borrow_mut();
            if let Some(mut current_image) = image_list.remove_current_image() {
                let (image_width, image_height) = current_image.preview_image_buffer_size().unwrap();
                let event_box_allocation = image_event_box.get_allocation();
                let (image_coords_start_position_x, image_coords_start_position_y) = (start_position_x as i32 - ((event_box_allocation.width - image_width) / 2), start_position_y as i32 - ((event_box_allocation.height - image_height) / 2));
                let (image_coords_end_position_x, image_coords_end_position_y) = (end_position_x as i32 - ((event_box_allocation.width - image_width) / 2), end_position_y as i32 - ((event_box_allocation.height - image_height) / 2));

                selection_coords.replace(None);
                let crop_operation = ImageOperation::Crop(current_image.preview_coords_to_image_coords(((image_coords_start_position_x, image_coords_start_position_y),(image_coords_end_position_x, image_coords_end_position_y))).unwrap());
                current_image = current_image.apply_operation(&crop_operation);
                current_image.create_preview_image_buffer(settings.borrow().scale());
                widgets.image_widget().set_from_pixbuf(current_image.preview_image_buffer());
                widgets.save_menu_button().set_sensitive(current_image.has_unsaved_operations());
                image_list.insert(file_list.borrow().current_file_path().unwrap(), current_image);

                widgets.image_widget().queue_draw();
                widgets.crop_button().set_active(false);
            }
        }
        gtk::Inhibit(false)
    }));

    widgets.image_widget().connect_draw(glib::clone!(@strong selection_coords, @strong image_list => move |image_widget, cairo_context| {
        if let Some(current_image) = image_list.borrow().current_image() {
            if let Some(((start_selection_coord_x, start_selection_coord_y),(end_selection_coord_x, end_selection_coord_y))) = selection_coords.get() {
                let image_buffer = current_image.preview_image_buffer().unwrap();
                cairo_context.set_source_pixbuf(image_buffer, (image_widget.get_allocation().width as f64 - image_buffer.get_width() as f64) / 2.0, (image_widget.get_allocation().height as f64 - image_buffer.get_height() as f64) / 2.0);
                cairo_context.paint();
                cairo_context.set_source_rgb(0.0, 0.0, 0.0);
                cairo_context.set_line_width(1.0);
                cairo_context.rectangle(start_selection_coord_x as f64, start_selection_coord_y as f64, (end_selection_coord_x - start_selection_coord_x) as f64, (end_selection_coord_y - start_selection_coord_y) as f64);
                cairo_context.stroke();
                return gtk::Inhibit(true);
            }
        }
        gtk::Inhibit(false)
    }));

    widgets.resize_button().connect_toggled(
        glib::clone!(@strong widgets, @strong image_list => move |resize_button| {
            if resize_button.get_active() {
                let image_list = image_list.borrow();
                if let Some(current_image) = image_list.current_image() {
                    let (image_width, image_height) = current_image.image_size().unwrap();
                    widgets.width_spin_button().set_value(image_width as f64);
                    widgets.height_spin_button().set_value(image_height as f64);
                }
            }
        }),
    );

    widgets.width_spin_button().connect_value_changed(
        glib::clone!(@strong widgets, @strong image_list => move |width_spin_button| {
            if widgets.link_aspect_ratio_button().get_active() {
                let image_list = image_list.borrow();
                if let Some(current_image) = image_list.current_image() {
                    let aspect_ratio = current_image.image_aspect_ratio().unwrap();
                    let new_width = width_spin_button.get_value();
                    widgets.height_spin_button().set_value(new_width / aspect_ratio);
                }
            }
        }),
    );

    widgets.height_spin_button().connect_value_changed(
        glib::clone!(@strong widgets, @strong image_list => move |height_spin_button| {
            if widgets.link_aspect_ratio_button().get_active() {
                let image_list = image_list.borrow();
                if let Some(current_image) = image_list.current_image() {
                    let aspect_ratio = current_image.image_aspect_ratio().unwrap();
                    let new_height = height_spin_button.get_value();
                    widgets.width_spin_button().set_value(new_height * aspect_ratio);
                }
            }
        }),
    );

    widgets.apply_resize_button().connect_clicked(glib::clone!(@strong widgets, @strong settings, @strong image_list, @strong file_list => move |_| {
        let mut image_list = image_list.borrow_mut();
        if let Some(mut current_image) = image_list.remove_current_image() {
            current_image = current_image.apply_operation(&ImageOperation::Resize((widgets.width_spin_button().get_value() as i32, widgets.height_spin_button().get_value() as i32)));
            current_image.create_preview_image_buffer(settings.borrow().scale());
            widgets.image_widget().set_from_pixbuf(current_image.preview_image_buffer());
            widgets.save_menu_button().set_sensitive(current_image.has_unsaved_operations());
            image_list.insert(file_list.borrow().current_file_path().unwrap(), current_image);
        }
    }));

    widgets.save_menu_button().connect_clicked(
        glib::clone!(@strong widgets, @strong image_list => move |save_menu_button| {
            widgets.popover_menu().popdown();
            match image_list.borrow_mut().save_current_image() {
                Ok(()) => {
                    save_menu_button.set_sensitive(false);
                },
                Err(error) => {
                    display_error(
                        widgets.error_info_bar(),
                        widgets.error_info_bar_text(),
                        error,
                    );
                },
            };

        }),
    );

    widgets
        .error_info_bar()
        .connect_response(|error_info_bar, response| {
            if response == gtk::ResponseType::Close {
                error_info_bar.set_revealed(false);
            }
        });

    widgets.window().show_all();
}
