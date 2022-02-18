use gtk::{
    gdk::{self, Rectangle, ScrollDirection},
    gdk_pixbuf::PixbufRotation,
    gio::{self, SimpleAction},
    glib::{self, Sender},
    prelude::{
        ActionMapExt, ButtonExt, FileChooserExt, GdkContextExt, InfoBarExt, NativeDialogExt,
        PopoverExt, SpinButtonExt, ToggleButtonExt, WidgetExt, WidgetExtManual,
    },
    MessageType, SpinButtonSignals, Window,
};
use std::{
    cell::{Cell, RefCell},
    path::PathBuf,
    rc::Rc,
};

use crate::{
    image::{CoordinatesPair, PreviewSize},
    image_list::ImageList,
    image_operation::ImageOperation,
    settings::Settings,
};

use super::widgets::Widgets;

#[derive(Debug)]
pub enum Event {
    OpenFile(gio::File),
    LoadImage(Option<PathBuf>),
    ImageViewportResize(Rectangle),
    RefreshPreview(PreviewSize),
    ChangePreviewSize(PreviewSize),
    ImageEdit(ImageOperation),
    StartSelection((u32, u32)),
    DragSelection((u32, u32)),
    SaveCurrentImage(Option<PathBuf>),
    DeleteCurrentImage,
    EndSelection,
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
    Quit,
}

pub fn post_event(sender: &glib::Sender<Event>, action: Event) {
    if let Err(err) = sender.send(action) {
        error!("Send error: {}", err);
    }
}

pub fn connect_events(
    application: gtk::Application,
    widgets: Widgets,
    sender: Sender<Event>,
    image_list: Rc<RefCell<ImageList>>,
    selection_coords: Rc<Cell<Option<CoordinatesPair>>>,
    settings: Settings,
) {
    widgets
        .image_event_box()
        .set_events(gdk::EventMask::POINTER_MOTION_MASK);

    connect_open_menu_button_clicked(widgets.clone(), sender.clone());
    connect_next_button_clicked(widgets.clone(), sender.clone());
    connect_previous_button_clicked(widgets.clone(), sender.clone());
    connect_image_viewport_size_allocate(widgets.clone(), sender.clone());
    connect_preview_smaller_button_clicked(widgets.clone(), sender.clone());
    connect_preview_larger_button_clicked(widgets.clone(), sender.clone());
    connect_preview_fit_screen_button_clicked(widgets.clone(), sender.clone());
    connect_rotate_counterclockwise_button_clicked(widgets.clone(), sender.clone());
    connect_rotate_clockwise_button_clicked(widgets.clone(), sender.clone());
    connect_image_event_box_button_press_event(widgets.clone(), sender.clone());
    connect_image_event_box_motion_notify_event(widgets.clone(), sender.clone());
    connect_image_event_box_button_release_event(widgets.clone(), sender.clone());
    connect_image_widget_draw(widgets.clone(), image_list.clone(), selection_coords);
    connect_resize_button_toggled(widgets.clone(), sender.clone());
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
    connect_window_resized(widgets.clone(), settings);
    connect_toggle_fullscreen(widgets.clone(), sender.clone());
    connect_quit(application, sender.clone());
    connect_image_scrolled_window_scroll_event(widgets.clone(), sender);

    widgets.window().show_all();
}

fn connect_open_menu_button_clicked(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .clone()
        .open_menu_button()
        .connect_clicked(move |_| {
            widgets.popover_menu().popdown();
            let file_chooser = gtk::FileChooserNative::new(
                Some("Open file"),
                <Option<&Window>>::None,
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
            });

            file_chooser.run();
            file_chooser.destroy();
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

fn connect_image_viewport_size_allocate(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .image_viewport()
        .connect_size_allocate(move |_, allocation| {
            post_event(&sender, Event::ImageViewportResize(*allocation));
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

fn connect_image_event_box_button_press_event(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .image_event_box()
        .connect_button_press_event(move |_, button_event| {
            let (position_x, position_y) = button_event.position();
            post_event(
                &sender,
                Event::StartSelection((position_x as u32, position_y as u32)),
            );
            gtk::Inhibit(false)
        });
}

fn connect_image_event_box_motion_notify_event(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .image_event_box()
        .connect_motion_notify_event(move |_, motion_event| {
            let (position_x, position_y) = motion_event.position();
            post_event(
                &sender,
                Event::DragSelection((position_x as u32, position_y as u32)),
            );
            gtk::Inhibit(false)
        });
}

fn connect_image_event_box_button_release_event(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .image_event_box()
        .connect_button_release_event(move |_, _| {
            post_event(&sender, Event::EndSelection);
            gtk::Inhibit(false)
        });
}

fn connect_image_widget_draw(
    widgets: Widgets,
    image_list: Rc<RefCell<ImageList>>,
    selection_coords: Rc<Cell<Option<CoordinatesPair>>>,
) {
    widgets
        .image_widget()
        .connect_draw(move |image_widget, cairo_context| {
            if let Some(current_image) = image_list.borrow().current_image() {
                if let Some((
                    (start_selection_coord_x, start_selection_coord_y),
                    (end_selection_coord_x, end_selection_coord_y),
                )) = selection_coords.get()
                {
                    let image_buffer = current_image.preview_image_buffer().unwrap();
                    cairo_context.set_source_pixbuf(
                        image_buffer,
                        (image_widget.allocation().width() as f64 - image_buffer.width() as f64)
                            / 2.0,
                        (image_widget.allocation().height() as f64 - image_buffer.height() as f64)
                            / 2.0,
                    );
                    if let Err(error) = cairo_context.paint() {
                        error!("{}", error);
                        return gtk::Inhibit(true);
                    }
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
                    return gtk::Inhibit(true);
                }
            }
            gtk::Inhibit(false)
        });
}

fn connect_resize_button_toggled(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .resize_button()
        .connect_toggled(move |resize_button| {
            if resize_button.is_active() {
                post_event(&sender, Event::ResizePopoverDisplayed);
            }
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
            widgets.resize_button().emit_clicked();
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
                file_chooser.set_filename(file_path);
            }

            let file_filter = gtk::FileFilter::new();
            file_filter.add_mime_type("image/*");
            file_filter.set_name(Some("Image"));

            file_chooser.add_filter(&file_filter);
            let sender = sender.clone();
            file_chooser.connect_response(move |file_chooser, response| {
                if response == gtk::ResponseType::Accept {
                    let filename = if let Some(filename) = file_chooser.filename() {
                        filename
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
                    post_event(&sender, Event::SaveCurrentImage(Some(filename)));
                }
            });
            file_chooser.run();
            file_chooser.destroy();
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

fn connect_window_resized(widgets: Widgets, settings: Settings) {
    widgets
        .window()
        .connect_size_allocate(move |_, allocation| {
            settings.set_window_size((allocation.width() as u32, allocation.height() as u32));
        });
}

fn connect_toggle_fullscreen(widgets: Widgets, sender: Sender<Event>) {
    let action_toggle_fullscreen = SimpleAction::new("toggle-fullscreen", None);
    action_toggle_fullscreen.connect_activate(move |_, _| {
        post_event(&sender, Event::ToggleFullscreen);
    });
    widgets.window().add_action(&action_toggle_fullscreen);
}

fn connect_quit(application: gtk::Application, sender: Sender<Event>) {
    let action_quit = SimpleAction::new("quit", None);
    action_quit.connect_activate(move |_, _| {
        post_event(&sender, Event::Quit);
    });
    application.add_action(&action_quit);
}

fn connect_image_scrolled_window_scroll_event(widgets: Widgets, sender: Sender<Event>) {
    widgets
        .image_scrolled_window()
        .connect_scroll_event(move |_, scroll_event| {
            if scroll_event.direction() == ScrollDirection::Up || scroll_event.delta().1 < 0.0 {
                post_event(&sender, Event::PreviewLarger(Some(5)));
            }
            if scroll_event.direction() == ScrollDirection::Down || scroll_event.delta().1 > 0.0 {
                post_event(&sender, Event::PreviewSmaller(Some(5)));
            }

            gtk::Inhibit(true)
        });
}
