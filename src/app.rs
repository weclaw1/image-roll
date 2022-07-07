use gtk::{
    gdk::Display,
    gio,
    glib::{self, timeout_future},
    prelude::*,
    Builder,
};

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    time::Duration,
};

use crate::image_list::ImageList;
use crate::settings::Settings;
use crate::ui::{
    event::{post_event, Event},
    widgets::Widgets,
};
use crate::{file_list::FileList, ui::controllers::Controllers};
use crate::{
    image::CoordinatesPair,
    ui::{action, event},
};

pub struct App {
    application: gtk::Application,
    controllers: Controllers,
    widgets: Widgets,
    image_list: Rc<RefCell<ImageList>>,
    file_list: FileList,
    selection_coords: Rc<Cell<Option<CoordinatesPair>>>,
    settings: Settings,
    sender: glib::Sender<Event>,
}

impl App {
    pub fn create(application: &gtk::Application, file: Option<&gio::File>) {
        let bytes = glib::Bytes::from_static(include_bytes!("resources/resources.gresource"));
        let resources = gio::Resource::from_data(&bytes).expect("Couldn't load resources");
        gio::resources_register(&resources);

        let builder = Builder::from_resource("/com/github/weclaw1/image-roll/image-roll.ui");

        let widgets: Widgets = Widgets::init(builder, application);

        let controllers = Controllers::init();

        gtk::IconTheme::for_display(&Display::default().unwrap())
            .add_resource_path("/com/github/weclaw1/image-roll/icons/");

        let image_list: Rc<RefCell<ImageList>> = Rc::new(RefCell::new(ImageList::new()));

        let file_list: FileList = FileList::new(None).unwrap();

        let selection_coords: Rc<Cell<Option<CoordinatesPair>>> = Rc::new(Cell::new(None));

        let settings: Settings = Settings::new(application.application_id().unwrap().as_str());

        let (window_width, window_height) = settings.window_size();
        widgets
            .window()
            .set_default_size(window_width as i32, window_height as i32);

        widgets
            .window()
            .settings()
            .set_gtk_application_prefer_dark_theme(true);

        let (sender, receiver) = glib::MainContext::channel::<Event>(glib::PRIORITY_DEFAULT);

        if let Some(file) = file {
            let main_context = glib::MainContext::default();
            let second_sender = sender.clone();
            let file = file.clone();
            main_context.spawn_local(async move {
                timeout_future(Duration::from_millis(10)).await;
                post_event(&second_sender, Event::OpenFile(file));
            });
        }

        let mut app = Self {
            application: application.clone(),
            controllers,
            widgets,
            image_list,
            file_list,
            selection_coords,
            settings,
            sender,
        };

        event::connect_events(
            app.widgets.clone(),
            app.sender.clone(),
            app.image_list.clone(),
            app.selection_coords.clone(),
            app.settings.clone(),
        );

        event::connect_controllers(
            app.sender.clone(),
            app.widgets.clone(),
            app.controllers.clone(),
        );

        action::update_buttons_state(
            &app.widgets,
            &app.file_list,
            app.image_list.clone(),
            &app.settings,
        );

        receiver.attach(None, move |e| {
            app.process_event(e);
            glib::Continue(true)
        });
    }

    pub fn process_event(&mut self, event: Event) {
        match event {
            Event::OpenFile(file) => action::open_file(
                &self.sender,
                self.image_list.clone(),
                &mut self.file_list,
                file,
            ),
            Event::LoadImage(file_path) => action::load_image(
                &self.sender,
                &mut self.settings,
                &self.widgets,
                self.image_list.clone(),
                file_path,
            ),
            Event::DisplayMessage(message, message_type) => {
                action::display_message(&self.widgets, message.as_str(), message_type)
            }
            Event::ImageViewportResize(viewport_size) => {
                action::image_viewport_resize(&self.sender, &mut self.settings, viewport_size)
            }
            Event::RefreshPreview(preview_size) => {
                action::refresh_preview(&self.widgets, self.image_list.clone(), preview_size)
            }
            Event::ChangePreviewSize(preview_size) => action::change_preview_size(
                &self.sender,
                &self.widgets,
                &mut self.settings,
                preview_size,
            ),
            Event::ImageEdit(image_operation) => action::image_edit(
                &self.sender,
                &self.settings,
                self.image_list.clone(),
                &self.file_list,
                image_operation,
            ),
            Event::StartSelection(position) if self.widgets.crop_button().is_active() => {
                action::start_selection(
                    &self.widgets,
                    self.image_list.clone(),
                    self.selection_coords.clone(),
                    position,
                )
            }
            Event::DragSelection(position) if self.widgets.crop_button().is_active() => {
                action::drag_selection(
                    &self.widgets,
                    self.image_list.clone(),
                    self.selection_coords.clone(),
                    position,
                )
            }
            Event::SaveCurrentImage(filename) => {
                action::save_current_image(&self.sender, self.image_list.clone(), filename);
                if self.file_list.current_folder_monitor_mut().is_none() {
                    action::refresh_file_list(&self.sender, &mut self.file_list);
                }
            }
            Event::DeleteCurrentImage => {
                action::delete_current_image(
                    &self.sender,
                    &mut self.file_list,
                    self.image_list.clone(),
                );
                if self.file_list.current_folder_monitor_mut().is_none() {
                    action::refresh_file_list(&self.sender, &mut self.file_list);
                }
            }
            Event::EndSelection if self.widgets.crop_button().is_active() => action::end_selection(
                &self.sender,
                &self.widgets,
                self.image_list.clone(),
                self.selection_coords.clone(),
            ),
            Event::PreviewSmaller(value) => {
                action::preview_smaller(&self.sender, &self.settings, value)
            }
            Event::PreviewLarger(value) => {
                action::preview_larger(&self.sender, &self.settings, value)
            }
            Event::PreviewFitScreen => action::preview_fit_screen(&self.sender),
            Event::NextImage => {
                action::next_image(&self.sender, self.image_list.clone(), &mut self.file_list)
            }
            Event::PreviousImage => {
                action::previous_image(&self.sender, self.image_list.clone(), &mut self.file_list)
            }
            Event::RefreshFileList => action::refresh_file_list(&self.sender, &mut self.file_list),
            Event::ResizePopoverDisplayed => {
                action::resize_popover_displayed(&self.widgets, self.image_list.clone())
            }
            Event::UpdateResizePopoverWidth => {
                action::update_resize_popover_width(&self.widgets, self.image_list.clone())
            }
            Event::UpdateResizePopoverHeight => {
                action::update_resize_popover_height(&self.widgets, self.image_list.clone())
            }
            Event::UndoOperation => {
                action::undo_operation(&self.sender, &self.settings, self.image_list.clone())
            }
            Event::RedoOperation => {
                action::redo_operation(&self.sender, &self.settings, self.image_list.clone())
            }
            Event::Print => action::print(&self.sender, &self.widgets, self.image_list.clone()),
            Event::HideInfoPanel => action::hide_info_panel(&self.widgets),
            Event::ToggleFullscreen => action::toggle_fullscreen(&self.widgets, &mut self.settings),
            Event::SetAsWallpaper => action::set_as_wallpaper(&self.sender, &self.file_list),
            Event::StartZoomGesture => action::start_zoom_gesture(&mut self.settings),
            Event::ZoomGestureScaleChanged(zoom_scale) => {
                action::change_scale_on_zoom_gesture(&self.sender, &self.settings, zoom_scale)
            }
            Event::CopyCurrentImage => action::copy_current_image(self.image_list.clone()),
            Event::Quit => action::quit(&self.application),
            event => debug!("Discarded unused event: {:?}", event),
        }
        action::update_buttons_state(
            &self.widgets,
            &self.file_list,
            self.image_list.clone(),
            &self.settings,
        );
    }
}
