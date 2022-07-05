use gtk::{
    gdk::{self, Key},
    gdk_pixbuf::PixbufRotation,
    gio,
    glib::{self, timeout_future, Sender},
    prelude::{
        ButtonExt, DrawingAreaExtManual, FileChooserExt, FileExt, GdkCairoContextExt,
        NativeDialogExt, PopoverExt, ToggleButtonExt, WidgetExt,
    },
    traits::{GestureExt, GestureSingleExt, GtkWindowExt},
    MessageType, Window,
};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
    time::Duration,
};

use crate::{
    image::{CoordinatesPair, PreviewSize},
    image_list::ImageList,
    image_operation::ImageOperation,
    settings::Settings,
};

use super::{controllers::Controllers, widgets::Widgets};

#[derive(Debug)]
pub enum Event {
    OpenFile(gio::File),
    LoadImage(Option<PathBuf>),
    ImageViewportResize((u32, u32)),
    RefreshPreview(PreviewSize),
    ChangePreviewSize(PreviewSize),
    ImageEdit(ImageOperation),
    StartSelection((u32, u32)),
    DragSelection((u32, u32)),
    SaveCurrentImage(Option<PathBuf>),
    DeleteCurrentImage,
    EndSelection,
    StartZoomGesture,
    ZoomGestureScaleChanged(f64),
    PreviewSmaller(Option<u32>),
    PreviewLarger(Option<u32>),
    PreviewFitScreen,
    NextImage,
    PreviousImage,
    RefreshFileList,
    ResizePopoverDisplayed,
    UpdateResizePopoverWidth,
    UpdateResizePopoverHeight,
    UndoOperation,
    RedoOperation,
    Print,
    DisplayMessage(String, gtk::MessageType),
    HideInfoPanel,
    ToggleFullscreen,
    CopyCurrentImage,
    Quit,
    SetAsWallpaper,
}

pub fn post_event(sender: &glib::Sender<Event>, action: Event) {
    if let Err(err) = sender.send(action) {
        error!("Send error: {}", err);
    }
}

pub fn connect_events(
    widgets: Widgets,
    sender: Sender<Event>,
    image_list: Rc<RefCell<ImageList>>,
    selection_coords: Rc<Cell<Option<CoordinatesPair>>>,
    settings: Settings,
) {
    connect_open_menu_button_clicked(widgets.clone(), sender.clone());
    connect_next_button_clicked(widgets.clone(), sender.clone());
    connect_previous_button_clicked(widgets.clone(), sender.clone());
    connect_window_default_width_notify(widgets.clone(), settings.clone(), sender.clone());
    connect_window_default_height_notify(widgets.clone(), settings, sender.clone());
    connect_window_maximized_notify(widgets.clone(), sender.clone());
    connect_window_fullscreened_notify(widgets.clone(), sender.clone());
    connect_preview_smaller_button_clicked(widgets.clone(), sender.clone());
    connect_preview_larger_button_clicked(widgets.clone(), sender.clone());
    connect_preview_fit_screen_button_clicked(widgets.clone(), sender.clone());
    connect_rotate_counterclockwise_button_clicked(widgets.clone(), sender.clone());
    connect_rotate_clockwise_button_clicked(widgets.clone(), sender.clone());
    connect_image_widget_draw(widgets.clone(), image_list.clone(), selection_coords);
    connect_resize_button_activated(widgets.clone(), sender.clone());
    connect_width_spin_button_value_changed(widgets.clone(), sender.clone());
    connect_height_spin_button_value_changed(widgets.clone(), sender.clone());
    connect_apply_resize_button_clicked(widgets.clone(), sender.clone());
    connect_save_menu_button_clicked(widgets.clone(), sender.clone());
    connect_print_menu_button_clicked(widgets.clone(), sender.clone());
    connect_undo_button_clicked(widgets.clone(), sender.clone());
    connect_redo_button_clicked(widgets.clone(), sender.clone());
    connect_save_as_menu_button_clicked(widgets.clone(), image_list, sender.clone());
    connect_delete_button_clicked(widgets.clone(), sender.clone());
    connect_info_bar_response(widgets.clone());
    connect_set_as_wallpaper_menu_button_clicked(widgets.clone(), sender.clone());
    connect_copy_menu_button_clicked(widgets.clone(), sender);

    widgets.window().present();
}

pub fn connect_controllers(sender: Sender<Event>, widgets: Widgets, controllers: Controllers) {
    controllers
        .image_click_gesture()
        .set_button(gtk::gdk::BUTTON_PRIMARY);
    connect_controllers_to_widgets(widgets.clone(), controllers.clone());
    connect_keybinds(controllers.clone(), widgets, sender.clone());
    connect_image_click_pressed_gesture(controllers.clone(), sender.clone());
    connect_image_motion_event_controller_motion(controllers.clone(), sender.clone());
    connect_image_click_released_gesture(controllers.clone(), sender.clone());
    connect_zoom_gesture_begin(controllers.clone(), sender.clone());
    connect_zoom_gesture_scale_changed(controllers.clone(), sender.clone());
    connect_image_scrolled_window_scroll_controller_scroll(controllers, sender);
}

fn connect_controllers_to_widgets(widgets: Widgets, controllers: Controllers) {
    widgets
        .window()
        .add_controller(controllers.window_key_event_controller());
    widgets
        .image_widget()
        .add_controller(controllers.image_click_gesture());
    widgets
        .image_widget()
        .add_controller(controllers.image_motion_event_controller());
    widgets
        .image_widget()
        .add_controller(controllers.image_zoom_gesture());
    widgets
        .image_scrolled_window()
        .add_controller(controllers.image_scrolled_window_scroll_controller());
}

pub fn connect_keybinds(controllers: Controllers, widgets: Widgets, sender: Sender<Event>) {
    controllers
        .window_key_event_controller()
        .connect_key_pressed(move |_, key, _, state| {
            match key {
                Key::F11 => post_event(&sender, Event::ToggleFullscreen),
                Key::Left | Key::h => {
                    if widgets.previous_button().is_sensitive() {
                        widgets.previous_button().emit_clicked();
                    }
                }
                Key::Right | Key::l => {
                    if widgets.next_button().is_sensitive() {
                        widgets.next_button().emit_clicked();
                    }
                }
                Key::minus | Key::KP_Subtract => {
                    if widgets.preview_smaller_button().is_sensitive() {
                        widgets.preview_smaller_button().emit_clicked();
                    }
                }
                Key::plus | Key::KP_Add => {
                    if widgets.preview_larger_button().is_sensitive() {
                        widgets.preview_larger_button().emit_clicked();
                    }
                }
                Key::f => {
                    if widgets.preview_fit_screen_button().is_sensitive() {
                        widgets.preview_fit_screen_button().emit_clicked();
                    }
                }
                Key::Delete => {
                    if widgets.delete_button().is_sensitive() {
                        widgets.delete_button().emit_clicked();
                    }
                }
                Key::S
                    if state
                        == (gdk::ModifierType::SHIFT_MASK | gdk::ModifierType::CONTROL_MASK) =>
                {
                    if widgets.save_as_menu_button().is_sensitive() {
                        widgets.save_as_menu_button().emit_clicked();
                    }
                }
                Key::R
                    if state
                        == (gdk::ModifierType::SHIFT_MASK | gdk::ModifierType::CONTROL_MASK) =>
                {
                    if widgets.rotate_counterclockwise_button().is_sensitive() {
                        widgets.rotate_counterclockwise_button().emit_clicked();
                    }
                }
                Key::C if state == gdk::ModifierType::SHIFT_MASK => {
                    if widgets.crop_button().is_sensitive() {
                        widgets.crop_button().emit_clicked();
                    }
                }
                Key::S if state == gdk::ModifierType::SHIFT_MASK => {
                    if widgets.resize_button().is_sensitive() {
                        widgets.resize_button().emit_activate();
                    }
                }
                Key::q if state == gdk::ModifierType::CONTROL_MASK => {
                    post_event(&sender, Event::Quit)
                }
                Key::o if state == gdk::ModifierType::CONTROL_MASK => {
                    widgets.open_menu_button().emit_clicked();
                }
                Key::s if state == gdk::ModifierType::CONTROL_MASK => {
                    if widgets.save_menu_button().is_sensitive() {
                        widgets.save_menu_button().emit_clicked();
                    }
                }
                Key::c if state == gdk::ModifierType::CONTROL_MASK => {
                    if widgets.copy_menu_button().is_sensitive() {
                        widgets.copy_menu_button().emit_clicked();
                    }
                }
                Key::p if state == gdk::ModifierType::CONTROL_MASK => {
                    if widgets.print_menu_button().is_sensitive() {
                        widgets.print_menu_button().emit_clicked();
                    }
                }
                Key::z if state == gdk::ModifierType::CONTROL_MASK => {
                    if widgets.undo_button().is_sensitive() {
                        widgets.undo_button().emit_clicked();
                    }
                }
                Key::y if state == gdk::ModifierType::CONTROL_MASK => {
                    if widgets.redo_button().is_sensitive() {
                        widgets.redo_button().emit_clicked();
                    }
                }
                Key::r if state == gdk::ModifierType::CONTROL_MASK => {
                    if widgets.rotate_clockwise_button().is_sensitive() {
                        widgets.rotate_clockwise_button().emit_clicked();
                    }
                }
                Key::j if state == gdk::ModifierType::CONTROL_MASK => {
                    if widgets.preview_smaller_button().is_sensitive() {
                        widgets.preview_smaller_button().emit_clicked();
                    }
                }
                Key::k if state == gdk::ModifierType::CONTROL_MASK => {
                    if widgets.preview_larger_button().is_sensitive() {
                        widgets.preview_larger_button().emit_clicked();
                    }
                }
                _ => {}
            }
            gtk::Inhibit(false)
        });
}

fn connect_open_menu_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .clone()
        .open_menu_button()
        .connect_clicked(move |_| {
            widgets.popover_menu().popdown();
            let file_chooser = gtk::FileChooserNative::new(
                Some("Open file"),
                gtk::Window::NONE,
                gtk::FileChooserAction::Open,
                None,
                None,
            );
            file_chooser.set_transient_for(Some(widgets.window()));

            let file_filter = gtk::FileFilter::new();
            file_filter.add_mime_type("image/*");
            file_filter.set_name(Some("Image"));

            file_chooser.add_filter(&file_filter);

            let sender = sender.clone();

            file_chooser.connect_response(move |file_chooser, response| {
                if response == gtk::ResponseType::Accept {
                    let file = if let Some(file) = file_chooser.file() {
                        file
                    } else {
                        post_event(
                            &sender,
                            Event::DisplayMessage(
                                String::from("Couldn't load file"),
                                MessageType::Error,
                            ),
                        );
                        return;
                    };
                    post_event(&sender, Event::OpenFile(file));
                }
                file_chooser.destroy();
            });
            file_chooser.show();
            widgets.file_chooser().replace(Some(file_chooser));
        });
}

fn connect_next_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets.next_button().connect_clicked(move |_| {
        post_event(&sender, Event::NextImage);
    });
}

fn connect_previous_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets.previous_button().connect_clicked(move |_| {
        post_event(&sender, Event::PreviousImage);
    });
}

fn connect_window_default_width_notify(
    widgets: Widgets,
    settings: Settings,
    sender: Sender<Event>,
) {
    widgets
        .clone()
        .window()
        .connect_default_width_notify(move |window| {
            settings.set_window_size((window.width() as u32, window.height() as u32));
            post_event(
                &sender,
                Event::ImageViewportResize((
                    widgets.image_viewport().allocation().width() as u32,
                    widgets.image_viewport().allocation().height() as u32,
                )),
            );
        });
}

fn connect_window_default_height_notify(
    widgets: Widgets,
    settings: Settings,
    sender: Sender<Event>,
) {
    widgets
        .clone()
        .window()
        .connect_default_height_notify(move |window| {
            settings.set_window_size((window.width() as u32, window.height() as u32));
            post_event(
                &sender,
                Event::ImageViewportResize((
                    widgets.image_viewport().allocation().width() as u32,
                    widgets.image_viewport().allocation().height() as u32,
                )),
            );
        });
}

fn connect_window_fullscreened_notify(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .clone()
        .window()
        .connect_fullscreened_notify(move |_| {
            let main_context = glib::MainContext::default();
            let sender = sender.clone();
            let widgets = widgets.clone();
            main_context.spawn_local(async move {
                timeout_future(Duration::from_millis(5)).await;
                post_event(
                    &sender,
                    Event::ImageViewportResize((
                        widgets.image_viewport().allocation().width() as u32,
                        widgets.image_viewport().allocation().height() as u32,
                    )),
                );
            });
        });
}

fn connect_window_maximized_notify(widgets: Widgets, sender: Sender<Event>) {
    widgets.clone().window().connect_maximized_notify(move |_| {
        let main_context = glib::MainContext::default();
        let sender = sender.clone();
        let widgets = widgets.clone();
        main_context.spawn_local(async move {
            timeout_future(Duration::from_millis(5)).await;
            post_event(
                &sender,
                Event::ImageViewportResize((
                    widgets.image_viewport().allocation().width() as u32,
                    widgets.image_viewport().allocation().height() as u32,
                )),
            );
        });
    });
}

fn connect_preview_smaller_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets.preview_smaller_button().connect_clicked(move |_| {
        post_event(&sender, Event::PreviewSmaller(None));
    });
}

fn connect_preview_larger_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets.preview_larger_button().connect_clicked(move |_| {
        post_event(&sender, Event::PreviewLarger(None));
    });
}

fn connect_preview_fit_screen_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .preview_fit_screen_button()
        .connect_clicked(move |_| {
            post_event(&sender, Event::PreviewFitScreen);
        });
}

fn connect_rotate_counterclockwise_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .rotate_counterclockwise_button()
        .connect_clicked(move |_| {
            post_event(
                &sender,
                Event::ImageEdit(ImageOperation::Rotate(PixbufRotation::Counterclockwise)),
            );
        });
}

fn connect_rotate_clockwise_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets.rotate_clockwise_button().connect_clicked(move |_| {
        post_event(
            &sender,
            Event::ImageEdit(ImageOperation::Rotate(PixbufRotation::Clockwise)),
        );
    });
}

fn connect_image_click_pressed_gesture(controllers: Controllers, sender: Sender<Event>) {
    controllers
        .image_click_gesture()
        .connect_pressed(move |_, _, x, y| {
            post_event(&sender, Event::StartSelection((x as u32, y as u32)));
        });
}

fn connect_image_motion_event_controller_motion(controllers: Controllers, sender: Sender<Event>) {
    controllers
        .image_motion_event_controller()
        .connect_motion(move |_, x, y| {
            post_event(&sender, Event::DragSelection((x as u32, y as u32)));
        });
}

fn connect_image_click_released_gesture(controllers: Controllers, sender: Sender<Event>) {
    controllers
        .image_click_gesture()
        .connect_released(move |_, _, _, _| {
            post_event(&sender, Event::EndSelection);
        });
}

fn connect_image_widget_draw(
    widgets: Widgets,
    image_list: Rc<RefCell<ImageList>>,
    selection_coords: Rc<Cell<Option<CoordinatesPair>>>,
) {
    widgets
        .image_widget()
        .set_draw_func(move |_, cairo_context, _, _| {
            if let Some(current_image) = image_list.borrow().current_image() {
                let image_buffer = current_image.preview_image_buffer().unwrap();
                cairo_context.set_source_pixbuf(image_buffer, 0.0, 0.0);
                if let Err(error) = cairo_context.paint() {
                    error!("{}", error);
                    return;
                }
                if let Some((
                    (start_selection_coord_x, start_selection_coord_y),
                    (end_selection_coord_x, end_selection_coord_y),
                )) = selection_coords.get()
                {
                    cairo_context.set_source_rgb(0.0, 0.0, 0.0);
                    cairo_context.set_line_width(1.0);
                    cairo_context.rectangle(
                        start_selection_coord_x as f64,
                        start_selection_coord_y as f64,
                        (end_selection_coord_x as i32 - start_selection_coord_x as i32) as f64,
                        (end_selection_coord_y as i32 - start_selection_coord_y as i32) as f64,
                    );
                    if let Err(error) = cairo_context.stroke() {
                        error!("{}", error);
                    }
                }
            }
        });
}

fn connect_resize_button_activated(widgets: Widgets, sender: Sender<Event>) {
    widgets.resize_button().connect_activate(move |_| {
        post_event(&sender, Event::ResizePopoverDisplayed);
    });
}

fn connect_width_spin_button_value_changed(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .clone()
        .width_spin_button()
        .connect_value_changed(move |_| {
            if widgets.link_aspect_ratio_button().is_active() {
                post_event(&sender, Event::UpdateResizePopoverHeight);
            }
        });
}

fn connect_height_spin_button_value_changed(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .clone()
        .height_spin_button()
        .connect_value_changed(move |_| {
            if widgets.link_aspect_ratio_button().is_active() {
                post_event(&sender, Event::UpdateResizePopoverWidth);
            }
        });
}

fn connect_apply_resize_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .clone()
        .apply_resize_button()
        .connect_clicked(move |_| {
            post_event(
                &sender,
                Event::ImageEdit(ImageOperation::Resize((
                    widgets.width_spin_button().value() as u32,
                    widgets.height_spin_button().value() as u32,
                ))),
            );
            widgets.resize_button().popdown();
        });
}

fn connect_save_menu_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .clone()
        .save_menu_button()
        .connect_clicked(move |_| {
            widgets.popover_menu().popdown();
            post_event(&sender, Event::SaveCurrentImage(None));
        });
}

fn connect_save_as_menu_button_clicked(
    widgets: Widgets,
    image_list: Rc<RefCell<ImageList>>,
    sender: Sender<Event>,
) {
    widgets
        .clone()
        .save_as_menu_button()
        .connect_clicked(move |_| {
            widgets.popover_menu().popdown();
            let file_chooser = gtk::FileChooserNative::new(
                Some("Save as..."),
                <Option<&Window>>::None,
                gtk::FileChooserAction::Save,
                None,
                None,
            );

            file_chooser.set_transient_for(Some(widgets.window()));

            if let Some(file_path) = image_list.borrow().current_image_path() {
                if let Err(error) = file_chooser.set_file(&gio::File::for_path(file_path)) {
                    post_event(
                        &sender,
                        Event::DisplayMessage(error.to_string(), MessageType::Warning),
                    );
                }
            }

            let file_filter = gtk::FileFilter::new();
            file_filter.add_mime_type("image/*");
            file_filter.set_name(Some("Image"));

            file_chooser.add_filter(&file_filter);
            let sender = sender.clone();
            file_chooser.connect_response(move |file_chooser, response| {
                if response == gtk::ResponseType::Accept {
                    let file = if let Some(file) = file_chooser.file() {
                        file
                    } else {
                        post_event(
                            &sender,
                            Event::DisplayMessage(
                                String::from("Couldn't save file"),
                                MessageType::Error,
                            ),
                        );
                        return;
                    };
                    post_event(&sender, Event::SaveCurrentImage(Some(file.path().unwrap())));
                }
                file_chooser.destroy();
            });
            file_chooser.show();
            widgets.file_chooser().replace(Some(file_chooser));
        });
}

fn connect_print_menu_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .clone()
        .print_menu_button()
        .connect_clicked(move |_| {
            widgets.popover_menu().popdown();

            post_event(&sender, Event::Print);
        });
}

fn connect_undo_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets.undo_button().connect_clicked(move |_| {
        post_event(&sender, Event::UndoOperation);
    });
}

fn connect_redo_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets.redo_button().connect_clicked(move |_| {
        post_event(&sender, Event::RedoOperation);
    });
}

fn connect_delete_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets.delete_button().connect_clicked(move |_| {
        post_event(&sender, Event::DeleteCurrentImage);
    });
}

fn connect_info_bar_response(widgets: Widgets) {
    widgets.info_bar().connect_response(|info_bar, response| {
        if response == gtk::ResponseType::Close {
            info_bar.set_revealed(false);
        }
    });
}

fn connect_image_scrolled_window_scroll_controller_scroll(
    controllers: Controllers,
    sender: Sender<Event>,
) {
    controllers
        .image_scrolled_window_scroll_controller()
        .connect_scroll(move |_, _, y| {
            if y < 0.0 {
                post_event(&sender, Event::PreviewLarger(Some(5)));
            }
            if y > 0.0 {
                post_event(&sender, Event::PreviewSmaller(Some(5)));
            }

            gtk::Inhibit(true)
        });
}

fn connect_set_as_wallpaper_menu_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .set_as_wallpaper_menu_button()
        .connect_clicked(move |_| {
            post_event(&sender, Event::SetAsWallpaper);
        });
}

fn connect_copy_menu_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .clone()
        .copy_menu_button()
        .connect_clicked(move |_| {
            widgets.popover_menu().popdown();
            post_event(&sender, Event::CopyCurrentImage);
        });
}

fn connect_zoom_gesture_begin(controllers: Controllers, sender: Sender<Event>) {
    controllers.image_zoom_gesture().connect_begin(move |_, _| {
        post_event(&sender, Event::StartZoomGesture);
    });
}

fn connect_zoom_gesture_scale_changed(controllers: Controllers, sender: Sender<Event>) {
    controllers
        .image_zoom_gesture()
        .connect_scale_changed(move |_, scale| {
            post_event(&sender, Event::ZoomGestureScaleChanged(scale));
        });
}
