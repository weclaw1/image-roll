use gtk::gio;
use gtk::gio::prelude::SettingsExt;

use crate::image::PreviewSize;

#[derive(Clone)]
pub struct Settings {
    settings: gio::Settings,
    scale: PreviewSize,
}

impl Settings {
    pub fn new(application_id: &str) -> Settings {
        Settings {
            settings: gio::Settings::new(application_id),
            scale: PreviewSize::BestFit(0, 0),
        }
    }

    pub fn set_window_size(&self, window_size: (u32, u32)) {
        let (window_width, window_height) = window_size;
        self.settings
            .set_uint("window-width", window_width)
            .expect("Could not set setting window-width.");
        self.settings
            .set_uint("window-height", window_height)
            .expect("Could not set setting window-height.");
    }

    pub fn window_size(&self) -> (u32, u32) {
        let window_width = self.settings.uint("window-width");
        let window_height = self.settings.uint("window-height");
        (window_width, window_height)
    }

    pub fn set_scale(&mut self, preview_size: PreviewSize) {
        self.scale = preview_size;
    }

    pub fn scale(&self) -> PreviewSize {
        self.scale
    }
}
