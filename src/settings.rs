use gtk::gio;
use gtk::gio::prelude::SettingsExt;
use gtk::gio::SettingsSchemaSource;

use crate::image::PreviewSize;

#[derive(Clone)]
pub struct Settings {
    gio_settings: Option<gio::Settings>,
    scale: PreviewSize,
}

impl Settings {
    pub fn new(application_id: &str) -> Settings {
        let gio_settings = match SettingsSchemaSource::default() {
            Some(schema_source) => {
                if schema_source.lookup(application_id, true).is_some() {
                    Some(gio::Settings::new(application_id))
                } else {
                    None
                }
            }
            None => None,
        };

        Settings {
            gio_settings,
            scale: PreviewSize::BestFit(0, 0),
        }
    }

    pub fn set_window_size(&self, window_size: (u32, u32)) {
        if let Some(gio_settings) = self.gio_settings.as_ref() {
            let (window_width, window_height) = window_size;
            gio_settings
                .set_uint("window-width", window_width)
                .expect("Could not set setting window-width.");
            gio_settings
                .set_uint("window-height", window_height)
                .expect("Could not set setting window-height.");
        }
    }

    pub fn window_size(&self) -> (u32, u32) {
        match self.gio_settings.as_ref() {
            Some(gio_settings) => (
                gio_settings.uint("window-width"),
                gio_settings.uint("window-height"),
            ),
            None => (1024, 768),
        }
    }

    pub fn set_scale(&mut self, preview_size: PreviewSize) {
        self.scale = preview_size;
    }

    pub fn scale(&self) -> PreviewSize {
        self.scale
    }
}
