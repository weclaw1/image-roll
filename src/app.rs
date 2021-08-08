use gtk::{
    gdk::{self, Rectangle},
    gdk_pixbuf::PixbufRotation,
    gio, glib,
    prelude::*,
    Builder,
};

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
use crate::ui::{
    event::{post_event, Event},
    widgets::Widgets,
};
use crate::{file_list::FileList, image};

pub struct App {
    widgets: Widgets,
    image_list: Rc<RefCell<ImageList>>,
    file_list: FileList,
    selection_coords: Rc<Cell<Option<CoordinatesPair>>>,
    settings: Settings,
    sender: glib::Sender<Event>,
}

impl App {
    pub fn new(app: &gtk::Application, file: Option<&gio::File>) -> Rc<RefCell<Self>> {
        let bytes = glib::Bytes::from_static(include_bytes!("resources/resources.gresource"));
        let resources = gio::Resource::from_data(&bytes).expect("Couldn't load resources");
        gio::resources_register(&resources);

        let builder = Builder::from_resource("/com/github/weclaw1/image-roll/image-roll_ui.glade");

        let widgets: Widgets = Widgets::init(builder, app);

        if let Some(theme) = gtk::IconTheme::default() {
            theme.add_resource_path("/com/github/weclaw1/image-roll");
        }

        let image_list: Rc<RefCell<ImageList>> = Rc::new(RefCell::new(ImageList::new()));

        let file_list: FileList = FileList::new(None).unwrap();

        let selection_coords: Rc<Cell<Option<CoordinatesPair>>> = Rc::new(Cell::new(None));

        let settings: Settings = Settings::new(PreviewSize::BestFit(
            widgets.image_viewport().allocation().width,
            widgets.image_viewport().allocation().height,
        ));

        let (sender, receiver) = glib::MainContext::channel::<Event>(glib::PRIORITY_DEFAULT);

        let second_sender = sender.clone();
        if let Some(file) = file {
            post_event(&second_sender, Event::OpenFile(file.clone()));
        }

        let app = Self {
            widgets,
            image_list,
            file_list,
            selection_coords,
            settings,
            sender,
        };

        let app = Rc::new(RefCell::new(app.connect_events()));
        let this = app.clone();

        receiver.attach(None, move |e| {
            this.borrow_mut().process_event(e);
            glib::Continue(true)
        });

        app
    }

    fn connect_events(self) -> Self {
        self.widgets
            .image_event_box()
            .set_events(gdk::EventMask::POINTER_MOTION_MASK);

        self.connect_open_menu_button_clicked();
        self.connect_next_button_clicked();
        self.connect_previous_button_clicked();
        self.connect_image_viewport_size_allocate();
        self.connect_preview_smaller_button_clicked();
        self.connect_preview_larger_button_clicked();
        self.connect_preview_fit_screen_button_clicked();
        self.connect_rotate_counterclockwise_button_clicked();
        self.connect_rotate_clockwise_button_clicked();
        self.connect_image_event_box_button_press_event();
        self.connect_image_event_box_motion_notify_event();
        self.connect_image_event_box_button_release_event();
        self.connect_image_widget_draw();
        self.connect_resize_button_toggled();
        self.connect_width_spin_button_value_changed();
        self.connect_height_spin_button_value_changed();
        self.connect_apply_resize_button_clicked();
        self.connect_save_menu_button_clicked();
        self.connect_print_menu_button_clicked();
        self.connect_undo_button_clicked();
        self.connect_redo_button_clicked();
        self.connect_save_as_menu_button_clicked();
        self.connect_delete_button_clicked();
        self.connect_error_info_bar_response();

        self.widgets.window().show_all();

        self
    }

    pub fn process_event(&mut self, event: Event) {
        match event {
            Event::OpenFile(file) => self.open_file(file),
            Event::LoadImage(file_path) => self.load_image(file_path),
            Event::DisplayError(error) => self.display_error(error),
            Event::ImageViewportResize(allocation) => self.image_viewport_resize(allocation),
            Event::RefreshPreview(preview_size) => self.refresh_preview(preview_size),
            Event::ChangePreviewSize(preview_size) => self.change_preview_size(preview_size),
            Event::ImageEdit(image_operation) => self.image_edit(image_operation),
            Event::StartSelection(position) if self.widgets.crop_button().is_active() => {
                self.start_selection(position)
            }
            Event::DragSelection(position) if self.widgets.crop_button().is_active() => {
                self.drag_selection(position)
            }
            Event::SaveCurrentImage(filename) => self.save_current_image(filename),
            Event::DeleteCurrentImage => self.delete_current_image(),
            Event::EndSelection if self.widgets.crop_button().is_active() => self.end_selection(),
            Event::PreviewSmaller => self.preview_smaller(),
            Event::PreviewLarger => self.preview_larger(),
            Event::PreviewFitScreen => self.preview_fit_screen(),
            Event::NextImage => self.next_image(),
            Event::PreviousImage => self.previous_image(),
            Event::RefreshFileList => self.refresh_file_list(),
            Event::ResizePopoverDisplayed => self.resize_popover_displayed(),
            Event::UpdateResizePopoverWidth => self.update_resize_popover_width(),
            Event::UpdateResizePopoverHeight => self.update_resize_popover_height(),
            Event::UndoOperation => self.undo_operation(),
            Event::RedoOperation => self.redo_operation(),
            Event::Print => self.print(),
            Event::HideErrorPanel => self.hide_error_panel(),
            event => debug!("Discarded unused event: {:?}", event),
        }
        self.update_buttons_state();
    }

    fn connect_open_menu_button_clicked(&self) {
        let widgets = self.widgets.clone();
        let sender = self.sender.clone();
        self.widgets.open_menu_button().connect_clicked(move |_| {
            widgets.popover_menu().popdown();
            let file_chooser = gtk::FileChooserNative::new(
                Some("Open file"),
                gtk::NONE_WINDOW,
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
                        post_event(&sender, Event::DisplayError(anyhow!("Couldn't load file")));
                        return;
                    };
                    post_event(&sender, Event::OpenFile(file));
                }
                file_chooser.destroy();
            });

            file_chooser.run();
        });
    }

    fn connect_next_button_clicked(&self) {
        let sender = self.sender.clone();
        self.widgets.next_button().connect_clicked(move |_| {
            post_event(&sender, Event::NextImage);
        });
    }

    fn connect_previous_button_clicked(&self) {
        let sender = self.sender.clone();
        self.widgets.previous_button().connect_clicked(move |_| {
            post_event(&sender, Event::PreviousImage);
        });
    }

    fn connect_image_viewport_size_allocate(&self) {
        let sender = self.sender.clone();
        self.widgets
            .image_viewport()
            .connect_size_allocate(move |_, allocation| {
                post_event(&sender, Event::ImageViewportResize(*allocation));
            });
    }

    fn connect_preview_smaller_button_clicked(&self) {
        let sender = self.sender.clone();
        self.widgets
            .preview_smaller_button()
            .connect_clicked(move |_| {
                post_event(&sender, Event::PreviewSmaller);
            });
    }

    fn connect_preview_larger_button_clicked(&self) {
        let sender = self.sender.clone();
        self.widgets
            .preview_larger_button()
            .connect_clicked(move |_| {
                post_event(&sender, Event::PreviewLarger);
            });
    }

    fn connect_preview_fit_screen_button_clicked(&self) {
        let sender = self.sender.clone();
        self.widgets
            .preview_fit_screen_button()
            .connect_clicked(move |_| {
                post_event(&sender, Event::PreviewFitScreen);
            });
    }

    fn connect_rotate_counterclockwise_button_clicked(&self) {
        let sender = self.sender.clone();
        self.widgets
            .rotate_counterclockwise_button()
            .connect_clicked(move |_| {
                post_event(
                    &sender,
                    Event::ImageEdit(ImageOperation::Rotate(PixbufRotation::Counterclockwise)),
                );
            });
    }

    fn connect_rotate_clockwise_button_clicked(&self) {
        let sender = self.sender.clone();
        self.widgets
            .rotate_clockwise_button()
            .connect_clicked(move |_| {
                post_event(
                    &sender,
                    Event::ImageEdit(ImageOperation::Rotate(PixbufRotation::Clockwise)),
                );
            });
    }

    fn connect_image_event_box_button_press_event(&self) {
        let sender = self.sender.clone();
        self.widgets
            .image_event_box()
            .connect_button_press_event(move |_, button_event| {
                let (position_x, position_y) = button_event.position();
                post_event(
                    &sender,
                    Event::StartSelection((position_x as i32, position_y as i32)),
                );
                gtk::Inhibit(false)
            });
    }

    fn connect_image_event_box_motion_notify_event(&self) {
        let sender = self.sender.clone();
        self.widgets
            .image_event_box()
            .connect_motion_notify_event(move |_, motion_event| {
                let (position_x, position_y) = motion_event.position();
                post_event(
                    &sender,
                    Event::DragSelection((position_x as i32, position_y as i32)),
                );
                gtk::Inhibit(false)
            });
    }

    fn connect_image_event_box_button_release_event(&self) {
        let sender = self.sender.clone();
        self.widgets
            .image_event_box()
            .connect_button_release_event(move |_, _| {
                post_event(&sender, Event::EndSelection);
                gtk::Inhibit(false)
            });
    }

    fn connect_image_widget_draw(&self) {
        let image_list = self.image_list.clone();
        let selection_coords = self.selection_coords.clone();
        self.widgets
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
                            (image_widget.allocation().width as f64 - image_buffer.width() as f64)
                                / 2.0,
                            (image_widget.allocation().height as f64
                                - image_buffer.height() as f64)
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
                            (end_selection_coord_x - start_selection_coord_x) as f64,
                            (end_selection_coord_y - start_selection_coord_y) as f64,
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

    fn connect_resize_button_toggled(&self) {
        let sender = self.sender.clone();
        self.widgets
            .resize_button()
            .connect_toggled(move |resize_button| {
                if resize_button.is_active() {
                    post_event(&sender, Event::ResizePopoverDisplayed);
                }
            });
    }

    fn connect_width_spin_button_value_changed(&self) {
        let sender = self.sender.clone();
        let widgets = self.widgets.clone();
        self.widgets
            .width_spin_button()
            .connect_value_changed(move |_| {
                if widgets.link_aspect_ratio_button().is_active() {
                    post_event(&sender, Event::UpdateResizePopoverHeight);
                }
            });
    }

    fn connect_height_spin_button_value_changed(&self) {
        let sender = self.sender.clone();
        let widgets = self.widgets.clone();
        self.widgets
            .height_spin_button()
            .connect_value_changed(move |_| {
                if widgets.link_aspect_ratio_button().is_active() {
                    post_event(&sender, Event::UpdateResizePopoverWidth);
                }
            });
    }

    fn connect_apply_resize_button_clicked(&self) {
        let sender = self.sender.clone();
        let widgets = self.widgets.clone();
        self.widgets
            .apply_resize_button()
            .connect_clicked(move |_| {
                post_event(
                    &sender,
                    Event::ImageEdit(ImageOperation::Resize((
                        widgets.width_spin_button().value() as i32,
                        widgets.height_spin_button().value() as i32,
                    ))),
                );
                widgets.resize_button().emit_clicked();
            });
    }

    fn connect_save_menu_button_clicked(&self) {
        let sender = self.sender.clone();
        let widgets = self.widgets.clone();
        self.widgets.save_menu_button().connect_clicked(move |_| {
            widgets.popover_menu().popdown();
            post_event(&sender, Event::SaveCurrentImage(None));
        });
    }

    fn connect_save_as_menu_button_clicked(&self) {
        let widgets = self.widgets.clone();
        let image_list = self.image_list.clone();
        let sender = self.sender.clone();
        self.widgets
            .save_as_menu_button()
            .connect_clicked(move |_| {
                widgets.popover_menu().popdown();
                let file_chooser = gtk::FileChooserNative::new(
                    Some("Save as..."),
                    gtk::NONE_WINDOW,
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
                            post_event(&sender, Event::DisplayError(anyhow!("Couldn't save file")));
                            return;
                        };
                        post_event(&sender, Event::SaveCurrentImage(Some(filename)));
                    }
                    file_chooser.destroy();
                });
                file_chooser.run();
            });
    }

    fn connect_print_menu_button_clicked(&self) {
        let sender = self.sender.clone();
        let widgets = self.widgets.clone();
        self.widgets.print_menu_button().connect_clicked(move |_| {
            widgets.popover_menu().popdown();

            post_event(&sender, Event::Print);
        });
    }

    fn connect_undo_button_clicked(&self) {
        let sender = self.sender.clone();
        self.widgets.undo_button().connect_clicked(move |_| {
            post_event(&sender, Event::UndoOperation);
        });
    }

    fn connect_redo_button_clicked(&self) {
        let sender = self.sender.clone();
        self.widgets.redo_button().connect_clicked(move |_| {
            post_event(&sender, Event::RedoOperation);
        });
    }

    fn connect_delete_button_clicked(&self) {
        let sender = self.sender.clone();
        self.widgets.delete_button().connect_clicked(move |_| {
            post_event(&sender, Event::DeleteCurrentImage);
        });
    }

    fn connect_error_info_bar_response(&self) {
        self.widgets
            .error_info_bar()
            .connect_response(|error_info_bar, response| {
                if response == gtk::ResponseType::Close {
                    error_info_bar.set_revealed(false);
                }
            });
    }

    fn refresh_file_list(&mut self) {
        post_event(&self.sender, Event::HideErrorPanel);
        if let Err(error) = self.file_list.refresh() {
            post_event(&self.sender, Event::DisplayError(error));
            return;
        };

        post_event(
            &self.sender,
            Event::LoadImage(self.file_list.current_file_path()),
        );
    }

    fn open_file(&mut self, file: gio::File) {
        post_event(&self.sender, Event::HideErrorPanel);
        self.image_list.replace(ImageList::new());

        let new_file_list = match FileList::new(Some(file)) {
            Ok(file_list) => file_list,
            Err(error) => {
                post_event(&self.sender, Event::DisplayError(error));
                return;
            }
        };

        self.file_list = new_file_list;

        post_event(
            &self.sender,
            Event::LoadImage(self.file_list.current_file_path()),
        );

        let sender = self.sender.clone();
        self.file_list
            .current_folder_monitor_mut()
            .unwrap()
            .connect_changed(move |_, _, _, _| {
                post_event(&sender, Event::RefreshFileList);
            });
    }

    fn load_image(&mut self, file_path: Option<PathBuf>) {
        self.hide_error_panel();
        let mut image_list = self.image_list.borrow_mut();
        if let Some(file_path) = file_path {
            let image = if let Some(image) = image_list.remove(&file_path) {
                image.reload(&file_path)
            } else {
                image::Image::load(&file_path)
            };
            let image = match image {
                Ok(image) => image,
                Err(error) => {
                    self.widgets.image_widget().set_from_pixbuf(None);
                    post_event(&self.sender, Event::DisplayError(error));
                    return;
                }
            };
            image_list.insert(file_path.clone(), image);
            self.widgets.window().set_title(
                file_path
                    .file_name()
                    .map(|file_name| file_name.to_str())
                    .flatten()
                    .unwrap_or_default(),
            );
            image_list.set_current_image_path(Some(file_path));
            post_event(&self.sender, Event::RefreshPreview(self.settings.scale()));
        } else {
            self.widgets.image_widget().set_from_pixbuf(None);
            self.widgets.window().set_title("Image Roll");
            image_list.set_current_image_path(None);
        }
    }

    fn next_image(&mut self) {
        if let Some(current_image) = self.image_list.borrow_mut().current_image_mut() {
            current_image.remove_image_buffers();
        }
        self.file_list.next();
        post_event(
            &self.sender,
            Event::LoadImage(self.file_list.current_file_path()),
        );
    }

    fn previous_image(&mut self) {
        if let Some(current_image) = self.image_list.borrow_mut().current_image_mut() {
            current_image.remove_image_buffers();
        }
        self.file_list.previous();
        post_event(
            &self.sender,
            Event::LoadImage(self.file_list.current_file_path()),
        );
    }

    fn image_viewport_resize(&mut self, allocation: Rectangle) {
        if let PreviewSize::BestFit(_, _) = self.settings.scale() {
            let new_scale = PreviewSize::BestFit(allocation.width, allocation.height);
            self.settings.set_scale(new_scale);
            post_event(&self.sender, Event::RefreshPreview(new_scale));
        }
    }

    fn refresh_preview(&mut self, preview_size: PreviewSize) {
        self.widgets.preview_size_label().set_text(String::from(preview_size).as_str());
        if let Some(image) = self.image_list.borrow_mut().current_image_mut() {
            image.create_preview_image_buffer(preview_size);
            self.widgets
                .image_widget()
                .set_from_pixbuf(image.preview_image_buffer());
        }
    }

    fn change_preview_size(&mut self, mut preview_size: PreviewSize) {
        if let PreviewSize::BestFit(_, _) = preview_size {
            let viewport_allocation = self.widgets.image_viewport().allocation();
            preview_size =
                PreviewSize::BestFit(viewport_allocation.width, viewport_allocation.height);
        }
        self.settings.set_scale(preview_size);
        post_event(&self.sender, Event::RefreshPreview(preview_size));
    }

    fn preview_smaller(&self) {
        let new_scale = self.settings.scale().smaller();
        post_event(
            &self.sender,
            Event::ChangePreviewSize(new_scale),
        );
    }

    fn preview_larger(&self) {
        let new_scale = self.settings.scale().larger();
        post_event(
            &self.sender,
            Event::ChangePreviewSize(new_scale),
        );
    }

    fn preview_fit_screen(&self) {
        let new_scale = PreviewSize::BestFit(0, 0);
        post_event(
            &self.sender,
            Event::ChangePreviewSize(new_scale),
        );
    }

    fn image_edit(&mut self, image_operation: ImageOperation) {
        let mut image_list = self.image_list.borrow_mut();
        if let Some(mut current_image) = image_list.remove_current_image() {
            current_image = current_image.apply_operation(&image_operation);
            image_list.insert(self.file_list.current_file_path().unwrap(), current_image);
            post_event(&self.sender, Event::RefreshPreview(self.settings.scale()));
        }
    }

    fn start_selection(&mut self, position: (i32, i32)) {
        if let Some(current_image) = self.image_list.borrow().current_image() {
            let (image_width, image_height) = current_image.preview_image_buffer_size().unwrap();
            let (position_x, position_y) = position;
            let event_box_allocation = self.widgets.image_event_box().allocation();
            let (image_coords_position_x, image_coords_position_y) = (
                position_x - ((event_box_allocation.width - image_width) / 2),
                position_y - ((event_box_allocation.height - image_height) / 2),
            );
            if image_coords_position_x >= 0
                && image_coords_position_x < image_width
                && image_coords_position_y >= 0
                && image_coords_position_y < image_height
            {
                self.selection_coords
                    .replace(Some(((position_x, position_y), (position_x, position_y))));
                self.widgets.image_widget().queue_draw();
            }
        }
    }

    fn drag_selection(&mut self, position: (i32, i32)) {
        if let Some(((start_position_x, start_position_y), (_, _))) = self.selection_coords.get() {
            if let Some(current_image) = self.image_list.borrow().current_image() {
                let (image_width, image_height) =
                    current_image.preview_image_buffer_size().unwrap();
                let (position_x, position_y) = position;
                let event_box_allocation = self.widgets.image_event_box().allocation();
                let (image_coords_position_x, image_coords_position_y) = (
                    position_x - ((event_box_allocation.width - image_width) / 2),
                    position_y - ((event_box_allocation.height - image_height) / 2),
                );
                if image_coords_position_x >= 0
                    && image_coords_position_x < image_width
                    && image_coords_position_y >= 0
                    && image_coords_position_y < image_height
                {
                    self.selection_coords.replace(Some((
                        (start_position_x, start_position_y),
                        (position_x, position_y),
                    )));
                    self.widgets.image_widget().queue_draw();
                }
            }
        }
    }

    fn end_selection(&mut self) {
        if let Some(((start_position_x, start_position_y), (end_position_x, end_position_y))) =
            self.selection_coords.take()
        {
            if let Some(current_image) = self.image_list.borrow().current_image() {
                let (image_width, image_height) =
                    current_image.preview_image_buffer_size().unwrap();
                let event_box_allocation = self.widgets.image_event_box().allocation();
                let (image_coords_start_position_x, image_coords_start_position_y) = (
                    start_position_x - ((event_box_allocation.width - image_width) / 2),
                    start_position_y - ((event_box_allocation.height - image_height) / 2),
                );
                let (image_coords_end_position_x, image_coords_end_position_y) = (
                    end_position_x - ((event_box_allocation.width - image_width) / 2),
                    end_position_y - ((event_box_allocation.height - image_height) / 2),
                );

                let crop_operation = ImageOperation::Crop(
                    current_image
                        .preview_coords_to_image_coords((
                            (image_coords_start_position_x, image_coords_start_position_y),
                            (image_coords_end_position_x, image_coords_end_position_y),
                        ))
                        .unwrap(),
                );
                post_event(&self.sender, Event::ImageEdit(crop_operation));

                self.widgets.image_widget().queue_draw();
                self.widgets.crop_button().set_active(false);
            }
        }
    }

    fn resize_popover_displayed(&self) {
        if let Some(current_image) = self.image_list.borrow().current_image() {
            let (image_width, image_height) = current_image.image_size().unwrap();
            self.widgets
                .width_spin_button()
                .set_value(image_width as f64);
            self.widgets
                .height_spin_button()
                .set_value(image_height as f64);
        }
    }

    fn update_resize_popover_width(&self) {
        if let Some(current_image) = self.image_list.borrow().current_image() {
            let aspect_ratio = current_image.image_aspect_ratio().unwrap();
            self.widgets
                .width_spin_button()
                .set_value(self.widgets.height_spin_button().value() * aspect_ratio);
        }
    }

    fn update_resize_popover_height(&self) {
        if let Some(current_image) = self.image_list.borrow().current_image() {
            let aspect_ratio = current_image.image_aspect_ratio().unwrap();
            self.widgets
                .height_spin_button()
                .set_value(self.widgets.width_spin_button().value() / aspect_ratio);
        }
    }

    fn save_current_image(&mut self, filename: Option<PathBuf>) {
        if let Err(error) = self.image_list.borrow_mut().save_current_image(filename) {
            post_event(&self.sender, Event::DisplayError(error));
        }
    }

    fn delete_current_image(&mut self) {
        if let Err(error) = self.image_list.borrow_mut().delete_current_image() {
            post_event(&self.sender, Event::DisplayError(error));
        }
    }

    fn print(&self) {
        let print_operation = gtk::PrintOperation::new();

        print_operation.connect_begin_print(move |print_operation, _| {
            print_operation.set_n_pages(1);
        });

        let sender = self.sender.clone();
        let image_list = self.image_list.clone();
        print_operation.connect_draw_page(move |_, print_context, _| {
            if let Some(print_image_buffer) = image_list
                .borrow()
                .current_image()
                .map(|current_image| {
                    current_image.create_print_image_buffer(
                        print_context.width() as i32,
                        print_context.height() as i32,
                    )
                })
                .flatten()
            {
                let cairo_context = print_context
                    .cairo_context()
                    .expect("Couldn't get cairo context");
                cairo_context.set_source_pixbuf(
                    &print_image_buffer,
                    (print_context.width() - print_image_buffer.width() as f64) / 2.0,
                    (print_context.height() - print_image_buffer.height() as f64) / 2.0,
                );
                if let Err(error) = cairo_context.paint() {
                    post_event(
                        &sender,
                        Event::DisplayError(anyhow!("Couldn't print current image: {}", error)),
                    );
                }
            }
        });

        print_operation.set_allow_async(true);
        if let Err(error) = print_operation.run(
            gtk::PrintOperationAction::PrintDialog,
            Option::from(self.widgets.window()),
        ) {
            post_event(
                &self.sender,
                Event::DisplayError(anyhow!("Couldn't print current image: {}", error)),
            );
        };
    }

    fn undo_operation(&mut self) {
        if let Some(current_image) = self.image_list.borrow_mut().current_image_mut() {
            current_image.undo_operation();
        }
    }

    fn redo_operation(&mut self) {
        if let Some(current_image) = self.image_list.borrow_mut().current_image_mut() {
            current_image.redo_operation();
        }
    }

    fn display_error(&self, error: anyhow::Error) {
        let error_text = format!("ERROR: {:#}", error);
        error!("{}", error_text);
        self.widgets.error_info_bar_text().set_text(&error_text);
        self.widgets.error_info_bar().set_revealed(true);
    }

    fn hide_error_panel(&self) {
        self.widgets.error_info_bar().set_revealed(false);
    }

    fn update_buttons_state(&self) {
        let previous_next_active = self.file_list.len() > 1;
        let buttons_active = self.file_list.len() > 0;

        self.widgets
            .next_button()
            .set_sensitive(previous_next_active);
        self.widgets
            .previous_button()
            .set_sensitive(previous_next_active);
        self.widgets
            .rotate_counterclockwise_button()
            .set_sensitive(buttons_active);
        self.widgets
            .rotate_clockwise_button()
            .set_sensitive(buttons_active);
        self.widgets.crop_button().set_sensitive(buttons_active);
        self.widgets.resize_button().set_sensitive(buttons_active);
        self.widgets
            .print_menu_button()
            .set_sensitive(buttons_active);

        if let Some(current_image) = self.image_list.borrow().current_image() {
            self.widgets
                .undo_button()
                .set_sensitive(current_image.can_undo_operation());
            self.widgets
                .redo_button()
                .set_sensitive(current_image.can_redo_operation());
            self.widgets
                .save_menu_button()
                .set_sensitive(current_image.has_unsaved_operations());
            self.widgets.save_as_menu_button().set_sensitive(true);
            self.widgets.delete_button().set_sensitive(true);
        } else {
            self.widgets.undo_button().set_sensitive(false);
            self.widgets.redo_button().set_sensitive(false);
            self.widgets.save_menu_button().set_sensitive(false);
            self.widgets.save_as_menu_button().set_sensitive(false);
            self.widgets.delete_button().set_sensitive(false);
        }

        self.widgets
            .preview_smaller_button()
            .set_sensitive(self.settings.scale().can_be_smaller());
        self.widgets
            .preview_larger_button()
            .set_sensitive(self.settings.scale().can_be_larger());
    }
}
