use std::path::Path;

use gdk_pixbuf::Pixbuf;

pub struct Image {
    image_buffer: Pixbuf,
}

impl Image {
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Image {
        let image_buffer = Pixbuf::from_file(path).expect("Couldn't load image");
        Image { image_buffer }
    }

    /// Get a reference to the image's image buffer.
    pub fn image_buffer(&self) -> &Pixbuf {
        &self.image_buffer
    }

    /// Get a reference to the image's image buffer.
    pub fn image_buffer_scaled(&self, width: i32, height: i32) -> Pixbuf {
        self.image_buffer
            .scale_simple(width, height, gdk_pixbuf::InterpType::Bilinear)
            .unwrap()
    }
}
