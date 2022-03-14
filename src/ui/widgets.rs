use gtk::{
    prelude::{BuilderExtManual, GtkWindowExt, WidgetExt},
    ApplicationWindow, Builder,
};

#[derive(Clone)]
pub struct Widgets {
    window: ApplicationWindow,
    open_menu_button: gtk::Button,
    image_widget: gtk::Image,
    popover_menu: gtk::PopoverMenu,
    next_button: gtk::Button,
    previous_button: gtk::Button,
    preview_smaller_button: gtk::Button,
    preview_larger_button: gtk::Button,
    image_scrolled_window: gtk::ScrolledWindow,
    image_viewport: gtk::Viewport,
    preview_size_label: gtk::Label,
    image_event_box: gtk::EventBox,
    rotate_counterclockwise_button: gtk::Button,
    rotate_clockwise_button: gtk::Button,
    crop_button: gtk::ToggleButton,
    resize_button: gtk::MenuButton,
    width_spin_button: gtk::SpinButton,
    height_spin_button: gtk::SpinButton,
    link_aspect_ratio_button: gtk::ToggleButton,
    apply_resize_button: gtk::Button,
    info_bar: gtk::InfoBar,
    info_bar_text: gtk::Label,
    save_menu_button: gtk::Button,
    print_menu_button: gtk::Button,
    undo_button: gtk::Button,
    redo_button: gtk::Button,
    save_as_menu_button: gtk::Button,
    preview_fit_screen_button: gtk::Button,
    delete_button: gtk::Button,
    copy_menu_button: gtk::Button,
    set_as_wallpaper_menu_button: gtk::Button,
}

impl Widgets {
    pub fn init(builder: Builder, application: &gtk::Application) -> Self {
        let window: ApplicationWindow = builder
            .object("main_window")
            .expect("Couldn't get main_window");
        window.set_application(Some(application));

        let open_menu_button: gtk::Button = builder
            .object("open_menu_button")
            .expect("Couldn't get open_menu_button");

        let image_widget: gtk::Image = builder
            .object("image_widget")
            .expect("Couldn't get image_widget");

        let popover_menu: gtk::PopoverMenu = builder
            .object("popover_menu")
            .expect("Couldn't get popover_menu");

        let next_button: gtk::Button = builder
            .object("next_button")
            .expect("Couldn't get next_button");
        let previous_button: gtk::Button = builder
            .object("previous_button")
            .expect("Couldn't get previous_button");

        let preview_smaller_button: gtk::Button = builder
            .object("preview_smaller_button")
            .expect("Couldn't get preview_smaller_button");
        let preview_larger_button: gtk::Button = builder
            .object("preview_larger_button")
            .expect("Couldn't get preview_larger_button");

        let image_scrolled_window: gtk::ScrolledWindow = builder
            .object("image_scrolled_window")
            .expect("Couldn't get image_scrolled_window");

        let image_viewport: gtk::Viewport = builder
            .object("image_viewport")
            .expect("Couldn't get image_viewport");

        let preview_size_label: gtk::Label = builder
            .object("preview_size_label")
            .expect("Couldn't get preview_size_label");

        let image_event_box: gtk::EventBox = builder
            .object("image_event_box")
            .expect("Couldn't get image_preview_box");

        let rotate_counterclockwise_button: gtk::Button = builder
            .object("rotate_counterclockwise_button")
            .expect("Couldn't get rotate_counterclockwise_button");
        let rotate_clockwise_button: gtk::Button = builder
            .object("rotate_clockwise_button")
            .expect("Couldn't get rotate_clockwise_button");

        let crop_button: gtk::ToggleButton = builder
            .object("crop_button")
            .expect("Couldn't get crop_button");

        let resize_button: gtk::MenuButton = builder
            .object("resize_button")
            .expect("Couldn't get resize_button");
        resize_button.set_sensitive(false);

        let width_spin_button: gtk::SpinButton = builder
            .object("width_spin_button")
            .expect("Couldn't get width_spin_button");
        let height_spin_button: gtk::SpinButton = builder
            .object("height_spin_button")
            .expect("Couldn't get height_spin_button");

        let link_aspect_ratio_button: gtk::ToggleButton = builder
            .object("link_aspect_ratio_button")
            .expect("Couldn't get link_aspect_ratio_button");

        let apply_resize_button: gtk::Button = builder
            .object("apply_resize_button")
            .expect("Couldn't get apply_resize_button");

        let error_info_bar: gtk::InfoBar = builder
            .object("error_info_bar")
            .expect("Couldn't get error_info_bar");

        let error_info_bar_text: gtk::Label = builder
            .object("error_info_bar_text")
            .expect("Couldn't get error_info_bar_text");

        let save_menu_button: gtk::Button = builder
            .object("save_menu_button")
            .expect("Couldn't get save_menu_button");

        let print_menu_button: gtk::Button = builder
            .object("print_menu_button")
            .expect("Couldn't get print_menu_button");

        let undo_button: gtk::Button = builder
            .object("undo_button")
            .expect("Couldn't get undo_button");

        let redo_button: gtk::Button = builder
            .object("redo_button")
            .expect("Couldn't get redo_button");

        let save_as_menu_button: gtk::Button = builder
            .object("save_as_menu_button")
            .expect("Couldn't get save_as_menu_button");

        let preview_fit_screen_button: gtk::Button = builder
            .object("preview_fit_screen_button")
            .expect("Couldn't get preview_fit_screen_button");

        let delete_button: gtk::Button = builder
            .object("delete_button")
            .expect("Couldn't get delete_button");

        let copy_menu_button: gtk::Button = builder
            .object("copy_menu_button")
            .expect("Couldn't get copy_menu_button");

        let set_as_wallpaper_menu_button: gtk::Button = builder
            .object("set_as_wallpaper_menu_button")
            .expect("Couldn't get set_as_wallpaper_menu_button");

        Self {
            window,
            open_menu_button,
            image_widget,
            popover_menu,
            next_button,
            previous_button,
            preview_smaller_button,
            preview_larger_button,
            image_scrolled_window,
            image_viewport,
            preview_size_label,
            image_event_box,
            rotate_counterclockwise_button,
            rotate_clockwise_button,
            crop_button,
            resize_button,
            width_spin_button,
            height_spin_button,
            link_aspect_ratio_button,
            apply_resize_button,
            info_bar: error_info_bar,
            info_bar_text: error_info_bar_text,
            save_menu_button,
            print_menu_button,
            undo_button,
            redo_button,
            save_as_menu_button,
            preview_fit_screen_button,
            delete_button,
            copy_menu_button,
            set_as_wallpaper_menu_button,
        }
    }

    /// Get a reference to the widgets's window.
    pub fn window(&self) -> &ApplicationWindow {
        &self.window
    }

    /// Get a reference to the widgets's open menu button.
    pub fn open_menu_button(&self) -> &gtk::Button {
        &self.open_menu_button
    }

    /// Get a reference to the widgets's image widget.
    pub fn image_widget(&self) -> &gtk::Image {
        &self.image_widget
    }

    /// Get a reference to the widgets's popover menu.
    pub fn popover_menu(&self) -> &gtk::PopoverMenu {
        &self.popover_menu
    }

    /// Get a reference to the widgets's next button.
    pub fn next_button(&self) -> &gtk::Button {
        &self.next_button
    }

    /// Get a reference to the widgets's previous button.
    pub fn previous_button(&self) -> &gtk::Button {
        &self.previous_button
    }

    /// Get a reference to the widgets's preview smaller button.
    pub fn preview_smaller_button(&self) -> &gtk::Button {
        &self.preview_smaller_button
    }

    /// Get a reference to the widgets's preview larger button.
    pub fn preview_larger_button(&self) -> &gtk::Button {
        &self.preview_larger_button
    }

    /// Get a reference to the widgets's image viewport.
    pub fn image_viewport(&self) -> &gtk::Viewport {
        &self.image_viewport
    }

    /// Get a reference to the widgets's image event box.
    pub fn image_event_box(&self) -> &gtk::EventBox {
        &self.image_event_box
    }

    /// Get a reference to the widgets's rotate counterclockwise button.
    pub fn rotate_counterclockwise_button(&self) -> &gtk::Button {
        &self.rotate_counterclockwise_button
    }

    /// Get a reference to the widgets's rotate clockwise button.
    pub fn rotate_clockwise_button(&self) -> &gtk::Button {
        &self.rotate_clockwise_button
    }

    /// Get a reference to the widgets's crop button.
    pub fn crop_button(&self) -> &gtk::ToggleButton {
        &self.crop_button
    }

    /// Get a reference to the widgets's resize button.
    pub fn resize_button(&self) -> &gtk::MenuButton {
        &self.resize_button
    }

    /// Get a reference to the widgets's width spin button.
    pub fn width_spin_button(&self) -> &gtk::SpinButton {
        &self.width_spin_button
    }

    /// Get a reference to the widgets's height spin button.
    pub fn height_spin_button(&self) -> &gtk::SpinButton {
        &self.height_spin_button
    }

    /// Get a reference to the widgets's link aspect ratio button.
    pub fn link_aspect_ratio_button(&self) -> &gtk::ToggleButton {
        &self.link_aspect_ratio_button
    }

    /// Get a reference to the widgets's apply resize button.
    pub fn apply_resize_button(&self) -> &gtk::Button {
        &self.apply_resize_button
    }

    /// Get a reference to the widgets's error info bar.
    pub fn info_bar(&self) -> &gtk::InfoBar {
        &self.info_bar
    }

    /// Get a reference to the widgets's error info bar text.
    pub fn info_bar_text(&self) -> &gtk::Label {
        &self.info_bar_text
    }

    /// Get a reference to the widgets's save menu button.
    pub fn save_menu_button(&self) -> &gtk::Button {
        &self.save_menu_button
    }

    /// Get a reference to the widgets's print menu button.
    pub fn print_menu_button(&self) -> &gtk::Button {
        &self.print_menu_button
    }

    /// Get a reference to the widgets's undo button.
    pub fn undo_button(&self) -> &gtk::Button {
        &self.undo_button
    }

    /// Get a reference to the widgets's redo button.
    pub fn redo_button(&self) -> &gtk::Button {
        &self.redo_button
    }

    /// Get a reference to the widgets's save as menu button.
    pub fn save_as_menu_button(&self) -> &gtk::Button {
        &self.save_as_menu_button
    }

    /// Get a reference to the widgets's preview fit screen button.
    pub fn preview_fit_screen_button(&self) -> &gtk::Button {
        &self.preview_fit_screen_button
    }

    /// Get a reference to the widgets's delete button.
    pub fn delete_button(&self) -> &gtk::Button {
        &self.delete_button
    }

    /// Get a reference to the widgets's preview size label.
    pub fn preview_size_label(&self) -> &gtk::Label {
        &self.preview_size_label
    }

    /// Get a reference to the widgets's image scrolled window.
    pub fn image_scrolled_window(&self) -> &gtk::ScrolledWindow {
        &self.image_scrolled_window
    }

    /// Get a reference to the widgets's set as wallpaper menu button.
    pub fn set_as_wallpaper_menu_button(&self) -> &gtk::Button {
        &self.set_as_wallpaper_menu_button
    }

    /// Get a reference to the widget's copy menu button
    pub fn copy_menu_button(&self) -> &gtk::Button {
        &self.copy_menu_button
    }
}
