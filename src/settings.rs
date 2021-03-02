use crate::image::PreviewSize;

pub struct Settings {
    preview_size: PreviewSize,
}

impl Settings {
    pub fn new(preview_size: PreviewSize) -> Settings {
        Settings { preview_size }
    }

    pub fn set_scale(&mut self, preview_size: PreviewSize) {
        self.preview_size = preview_size;
    }

    pub fn scale(&self) -> PreviewSize {
        self.preview_size
    }
}
