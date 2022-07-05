use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
};

#[cfg(feature = "wallpaper")]
use ashpd::{
    desktop::wallpaper::{self, SetOn},
    WindowIdentifier,
};
use gtk::{
    gdk, gio,
    glib::{self, timeout_future_seconds, Sender},
    prelude::{
        DisplayExt, FileMonitorExt, GdkCairoContextExt, GtkApplicationExt, GtkWindowExt,
        PrintOperationExt, ToggleButtonExt, WidgetExt,
    },
    traits::DrawingAreaExt,
    MessageType,
};

use crate::{
    file_list::FileList,
    image::{self, CoordinatesPair, PreviewSize},
    image_list::ImageList,
    image_operation::{ApplyImageOperation, ImageOperation},
    settings::Settings,
};

use super::{
    event::{post_event, Event},
    widgets::Widgets,
};

pub fn refresh_file_list(sender: &Sender<Event>, file_list: &mut FileList) {
    post_event(sender, Event::HideInfoPanel);
    if let Err(error) = file_list.refresh() {
        post_event(
            sender,
            Event::DisplayMessage(error.to_string(), MessageType::Error),
        );
        return;
    };

    post_event(sender, Event::LoadImage(file_list.current_file_path()));
}

pub fn open_file(
    sender: &Sender<Event>,
    image_list: Rc<RefCell<ImageList>>,
    file_list: &mut FileList,
    file: gio::File,
) {
    post_event(sender, Event::HideInfoPanel);
    image_list.replace(ImageList::new());

    let new_file_list = match FileList::new(Some(file)) {
        Ok(file_list) => file_list,
        Err(error) => {
            post_event(
                sender,
                Event::DisplayMessage(error.to_string(), MessageType::Error),
            );
            return;
        }
    };

    *file_list = new_file_list;

    post_event(sender, Event::LoadImage(file_list.current_file_path()));

    let sender = sender.clone();
    file_list
        .current_folder_monitor_mut()
        .unwrap()
        .connect_changed(move |_, _, _, _| {
            post_event(&sender, Event::RefreshFileList);
        });
}

pub fn load_image(
    sender: &Sender<Event>,
    settings: &mut Settings,
    widgets: &Widgets,
    image_list: Rc<RefCell<ImageList>>,
    file_path: Option<PathBuf>,
) {
    hide_info_panel(widgets);
    let mut image_list = image_list.borrow_mut();
    if let Some(file_path) = file_path {
        let image = if let Some(image) = image_list.remove(&file_path) {
            image.reload(&file_path)
        } else {
            image::Image::load(&file_path)
        };
        let image = match image {
            Ok(image) => image,
            Err(error) => {
                image_list.set_current_image_path(None);
                post_event(sender, Event::RefreshPreview(settings.scale()));
                post_event(
                    sender,
                    Event::DisplayMessage(error.to_string(), MessageType::Error),
                );
                return;
            }
        };
        image_list.insert(file_path.clone(), image);
        widgets.window().set_title(
            file_path
                .file_name()
                .and_then(|file_name| file_name.to_str()),
        );
        image_list.set_current_image_path(Some(file_path));
        if let PreviewSize::BestFit(0, 0) = settings.scale() {
            let new_scale = PreviewSize::BestFit(
                widgets.image_viewport().allocation().width() as u32,
                widgets.image_viewport().allocation().height() as u32,
            );
            settings.set_scale(new_scale);
        }
        post_event(sender, Event::RefreshPreview(settings.scale()));
    } else {
        widgets.window().set_title(Some("Image Roll"));
        image_list.set_current_image_path(None);
        post_event(sender, Event::RefreshPreview(settings.scale()));
    }
}

pub fn next_image(
    sender: &Sender<Event>,
    image_list: Rc<RefCell<ImageList>>,
    file_list: &mut FileList,
) {
    if let Some(current_image) = image_list.borrow_mut().current_image_mut() {
        current_image.remove_image_buffers();
    }
    file_list.next();
    post_event(sender, Event::LoadImage(file_list.current_file_path()));
}

pub fn previous_image(
    sender: &Sender<Event>,
    image_list: Rc<RefCell<ImageList>>,
    file_list: &mut FileList,
) {
    if let Some(current_image) = image_list.borrow_mut().current_image_mut() {
        current_image.remove_image_buffers();
    }
    file_list.previous();
    post_event(sender, Event::LoadImage(file_list.current_file_path()));
}

pub fn image_viewport_resize(
    sender: &Sender<Event>,
    settings: &mut Settings,
    allocation: gdk::Rectangle,
) {
    if let PreviewSize::BestFit(_, _) = settings.scale() {
        let new_scale = PreviewSize::BestFit(allocation.width() as u32, allocation.height() as u32);
        settings.set_scale(new_scale);
        post_event(sender, Event::RefreshPreview(new_scale));
    }
}

pub fn refresh_preview(
    widgets: &Widgets,
    image_list: Rc<RefCell<ImageList>>,
    preview_size: PreviewSize,
) {
    widgets
        .preview_size_label()
        .set_text(String::from(preview_size).as_str());
    if let Some(image) = image_list.borrow_mut().current_image_mut() {
        image.create_preview_image_buffer(preview_size);
        let (preview_image_width, preview_image_height) =
            image.preview_image_buffer_size().unwrap();
        widgets
            .image_widget()
            .set_content_width(preview_image_width as i32);
        widgets
            .image_widget()
            .set_content_height(preview_image_height as i32);
    } else {
        widgets.image_widget().set_content_width(0);
        widgets.image_widget().set_content_height(0);
    }
    widgets.image_widget().queue_draw();
}

pub fn change_preview_size(
    sender: &Sender<Event>,
    widgets: &Widgets,
    settings: &mut Settings,
    mut preview_size: PreviewSize,
) {
    if let PreviewSize::BestFit(_, _) = preview_size {
        let viewport_allocation = widgets.image_viewport().allocation();
        preview_size = PreviewSize::BestFit(
            viewport_allocation.width() as u32,
            viewport_allocation.height() as u32,
        );
    }
    settings.set_scale(preview_size);
    post_event(sender, Event::RefreshPreview(preview_size));
}

pub fn preview_smaller(sender: &Sender<Event>, settings: &Settings, value: Option<u32>) {
    let new_scale = match value {
        None => settings.scale().smaller(),
        Some(value) => settings.scale().smaller_by(value),
    };
    if let Some(new_scale) = new_scale {
        post_event(sender, Event::ChangePreviewSize(new_scale));
    }
}

pub fn preview_larger(sender: &Sender<Event>, settings: &Settings, value: Option<u32>) {
    let new_scale = match value {
        None => settings.scale().larger(),
        Some(value) => settings.scale().larger_by(value),
    };
    if let Some(new_scale) = new_scale {
        post_event(sender, Event::ChangePreviewSize(new_scale));
    }
}

pub fn preview_fit_screen(sender: &Sender<Event>) {
    let new_scale = PreviewSize::BestFit(0, 0);
    post_event(sender, Event::ChangePreviewSize(new_scale));
}

pub fn image_edit(
    sender: &Sender<Event>,
    settings: &Settings,
    image_list: Rc<RefCell<ImageList>>,
    file_list: &FileList,
    image_operation: ImageOperation,
) {
    let mut image_list = image_list.borrow_mut();
    if let Some(mut current_image) = image_list.remove_current_image() {
        current_image = current_image.apply_operation(&image_operation);
        image_list.insert(file_list.current_file_path().unwrap(), current_image);
        post_event(sender, Event::RefreshPreview(settings.scale()));
    }
}

pub fn start_selection(
    widgets: &Widgets,
    image_list: Rc<RefCell<ImageList>>,
    selection_coords: Rc<Cell<Option<CoordinatesPair>>>,
    position: (u32, u32),
) {
    if image_list.borrow().current_image().is_some() {
        selection_coords.replace(Some((position, position)));
        widgets.image_widget().queue_draw();
    }
}

pub fn drag_selection(
    widgets: &Widgets,
    image_list: Rc<RefCell<ImageList>>,
    selection_coords: Rc<Cell<Option<CoordinatesPair>>>,
    position: (u32, u32),
) {
    if let Some(((start_position_x, start_position_y), (_, _))) = selection_coords.get() {
        if let Some(current_image) = image_list.borrow().current_image() {
            let (position_x, position_y) = position;
            let (image_width, image_height) = current_image.preview_image_buffer_size().unwrap();
            if position_x >= image_width || position_y >= image_height {
                return;
            }
            selection_coords.replace(Some(((start_position_x, start_position_y), position)));
            widgets.image_widget().queue_draw();
        }
    }
}

pub fn end_selection(
    sender: &Sender<Event>,
    widgets: &Widgets,
    image_list: Rc<RefCell<ImageList>>,
    selection_coords: Rc<Cell<Option<CoordinatesPair>>>,
) {
    if let Some(selection_coords) = selection_coords.take() {
        if let Some(current_image) = image_list.borrow().current_image() {
            let crop_operation = ImageOperation::Crop(
                current_image
                    .preview_coords_to_image_coords(selection_coords)
                    .unwrap(),
            );
            post_event(sender, Event::ImageEdit(crop_operation));

            widgets.image_widget().queue_draw();
            widgets.crop_button().set_active(false);
        }
    }
}

pub fn resize_popover_displayed(widgets: &Widgets, image_list: Rc<RefCell<ImageList>>) {
    if let Some(current_image) = image_list.borrow().current_image() {
        let (image_width, image_height) = current_image.image_size().unwrap();
        widgets.width_spin_button().set_value(image_width as f64);
        widgets.height_spin_button().set_value(image_height as f64);
    }
}

pub fn update_resize_popover_width(widgets: &Widgets, image_list: Rc<RefCell<ImageList>>) {
    if let Some(current_image) = image_list.borrow().current_image() {
        let aspect_ratio = current_image.image_aspect_ratio().unwrap();
        widgets
            .width_spin_button()
            .set_value(widgets.height_spin_button().value() * aspect_ratio);
    }
}

pub fn update_resize_popover_height(widgets: &Widgets, image_list: Rc<RefCell<ImageList>>) {
    if let Some(current_image) = image_list.borrow().current_image() {
        let aspect_ratio = current_image.image_aspect_ratio().unwrap();
        widgets
            .height_spin_button()
            .set_value(widgets.width_spin_button().value() / aspect_ratio);
    }
}

pub fn save_current_image(
    sender: &Sender<Event>,
    image_list: Rc<RefCell<ImageList>>,
    filename: Option<PathBuf>,
) {
    if let Err(error) = image_list.borrow_mut().save_current_image(filename) {
        post_event(
            sender,
            Event::DisplayMessage(error.to_string(), MessageType::Error),
        );
    }
}

pub fn delete_current_image(
    sender: &Sender<Event>,
    file_list: &mut FileList,
    image_list: Rc<RefCell<ImageList>>,
) {
    match file_list.delete_current_file() {
        Ok(image_path) => {
            image_list.borrow_mut().remove(image_path.as_path());
            post_event(
                sender,
                Event::DisplayMessage(
                    format!(
                        "Image {} was moved to trash",
                        image_path
                            .file_name()
                            .and_then(|file_name| file_name.to_str())
                            .unwrap_or_default()
                    ),
                    MessageType::Info,
                ),
            )
        }
        Err(error) => post_event(
            sender,
            Event::DisplayMessage(error.to_string(), MessageType::Error),
        ),
    }
}

pub fn print(sender: &Sender<Event>, widgets: &Widgets, image_list: Rc<RefCell<ImageList>>) {
    let print_operation = gtk::PrintOperation::new();

    print_operation.connect_begin_print(move |print_operation, _| {
        print_operation.set_n_pages(1);
    });

    let cloned_sender = sender.clone();
    print_operation.connect_draw_page(move |_, print_context, _| {
        if let Some(print_image_buffer) =
            image_list
                .borrow()
                .current_image()
                .and_then(|current_image| {
                    current_image.create_print_image_buffer(
                        print_context.width() as u32,
                        print_context.height() as u32,
                    )
                })
        {
            let cairo_context = print_context.cairo_context();
            cairo_context.set_source_pixbuf(
                &print_image_buffer,
                (print_context.width() - print_image_buffer.width() as f64) / 2.0,
                (print_context.height() - print_image_buffer.height() as f64) / 2.0,
            );
            if let Err(error) = cairo_context.paint() {
                post_event(
                    &cloned_sender,
                    Event::DisplayMessage(
                        format!("Couldn't print current image: {}", error),
                        MessageType::Error,
                    ),
                );
            }
        }
    });

    print_operation.set_allow_async(true);
    if let Err(error) = print_operation.run(
        gtk::PrintOperationAction::PrintDialog,
        Option::from(widgets.window()),
    ) {
        post_event(
            sender,
            Event::DisplayMessage(
                format!("Couldn't print current image: {}", error),
                MessageType::Error,
            ),
        );
    };
}

pub fn undo_operation(
    sender: &Sender<Event>,
    settings: &Settings,
    image_list: Rc<RefCell<ImageList>>,
) {
    if let Some(current_image) = image_list.borrow_mut().current_image_mut() {
        current_image.undo_operation();
        post_event(sender, Event::RefreshPreview(settings.scale()));
    }
}

pub fn redo_operation(
    sender: &Sender<Event>,
    settings: &Settings,
    image_list: Rc<RefCell<ImageList>>,
) {
    if let Some(current_image) = image_list.borrow_mut().current_image_mut() {
        current_image.redo_operation();
        post_event(sender, Event::RefreshPreview(settings.scale()));
    }
}

pub fn display_message(widgets: &Widgets, message: &str, message_type: gtk::MessageType) {
    match message_type {
        MessageType::Error => error!("{}", message),
        MessageType::Warning => warn!("{}", message),
        MessageType::Info => info!("{}", message),
        _ => info!("{}", message),
    };
    widgets.info_bar().set_message_type(message_type);
    widgets.info_bar_text().set_text(message);
    widgets.info_bar().set_revealed(true);
    let main_context = glib::MainContext::default();
    let info_bar = widgets.info_bar().clone();
    main_context.spawn_local(async move {
        timeout_future_seconds(5).await;
        info_bar.set_revealed(false);
    });
}

pub fn hide_info_panel(widgets: &Widgets) {
    if widgets.info_bar().message_type() != gtk::MessageType::Info
        && widgets.info_bar().message_type() != gtk::MessageType::Warning
    {
        widgets.info_bar().set_revealed(false);
    }
}

pub fn toggle_fullscreen(widgets: &Widgets, settings: &mut Settings) {
    if !settings.fullscreen() {
        widgets.window().fullscreen();
        settings.set_fullscreen(true);
    } else {
        widgets.window().unfullscreen();
        settings.set_fullscreen(false);
    }
}

pub fn quit(application: &gtk::Application) {
    application
        .windows()
        .iter()
        .for_each(|window| window.close());
}

#[cfg(feature = "wallpaper")]
pub fn set_as_wallpaper(sender: &Sender<Event>, file_list: &FileList) {
    if let Some(current_file_uri) = file_list.current_file_uri() {
        let sender = sender.clone();
        let main_context = glib::MainContext::default();
        main_context.spawn_local(async move {
            if let Err(error) = wallpaper::set_from_uri(
                &WindowIdentifier::default(),
                current_file_uri.as_str(),
                true,
                SetOn::Background,
            )
            .await
            {
                post_event(
                    &sender,
                    Event::DisplayMessage(error.to_string(), MessageType::Error),
                );
            }
        });
    }
}
#[cfg(not(feature = "wallpaper"))]
pub fn set_as_wallpaper(_sender: &Sender<Event>, _file_list: &FileList) {
    error!("This program was built without the wallpaper feature");
}

pub fn copy_current_image(image_list: Rc<RefCell<ImageList>>) {
    let display = gdk::Display::default().unwrap();
    image_list.borrow().copy_current_image(display.clipboard());
}

pub fn start_zoom_gesture(settings: &mut Settings) {
    settings.set_scale_before_zoom_gesture(Some(settings.scale()));
}

pub fn change_scale_on_zoom_gesture(sender: &Sender<Event>, settings: &Settings, zoom_scale: f64) {
    if let Some(scale_before_zoom_gesture) = settings.scale_before_zoom_gesture() {
        let new_preview_size = match scale_before_zoom_gesture {
            PreviewSize::BestFit(_, _) | PreviewSize::OriginalSize => {
                PreviewSize::Resized((zoom_scale * 100.0) as u32)
            }
            PreviewSize::Resized(old_scale) => {
                PreviewSize::Resized((old_scale as f64 * zoom_scale) as u32)
            }
        };
        post_event(sender, Event::ChangePreviewSize(new_preview_size));
    }
}

pub fn update_buttons_state(
    widgets: &Widgets,
    file_list: &FileList,
    image_list: Rc<RefCell<ImageList>>,
    settings: &Settings,
) {
    let previous_next_active = file_list.len() > 1;
    widgets.next_button().set_sensitive(previous_next_active);
    widgets
        .previous_button()
        .set_sensitive(previous_next_active);

    let buttons_active = if let Some(current_image) = image_list.borrow().current_image() {
        widgets
            .undo_button()
            .set_sensitive(current_image.can_undo_operation());
        widgets
            .redo_button()
            .set_sensitive(current_image.can_redo_operation());
        widgets
            .save_menu_button()
            .set_sensitive(current_image.has_operations());
        true
    } else {
        widgets.undo_button().set_sensitive(false);
        widgets.redo_button().set_sensitive(false);
        widgets.save_menu_button().set_sensitive(false);
        false
    };

    widgets
        .rotate_counterclockwise_button()
        .set_sensitive(buttons_active);
    widgets
        .rotate_clockwise_button()
        .set_sensitive(buttons_active);
    widgets.crop_button().set_sensitive(buttons_active);
    widgets.resize_button().set_sensitive(buttons_active);
    widgets.print_menu_button().set_sensitive(buttons_active);
    widgets.save_as_menu_button().set_sensitive(buttons_active);
    widgets.delete_button().set_sensitive(buttons_active);

    #[cfg(feature = "wallpaper")]
    widgets
        .set_as_wallpaper_menu_button()
        .set_sensitive(buttons_active);
    #[cfg(not(feature = "wallpaper"))]
    widgets.set_as_wallpaper_menu_button().set_sensitive(false);

    widgets.copy_menu_button().set_sensitive(buttons_active);

    widgets
        .preview_smaller_button()
        .set_sensitive(settings.scale().can_be_smaller());
    widgets
        .preview_larger_button()
        .set_sensitive(settings.scale().can_be_larger());
}
