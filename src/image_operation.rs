use gdk_pixbuf::{Pixbuf, PixbufRotation};

#[derive(Copy, Clone)]
pub enum ImageOperation {
    Rotate(PixbufRotation),
}

pub trait ApplyImageOperation {
    fn apply_operation(self, image_operation: &ImageOperation) -> Self;
}

impl ApplyImageOperation for Pixbuf {
    fn apply_operation(self, image_operation: &ImageOperation) -> Pixbuf {
        match image_operation {
            ImageOperation::Rotate(rotation) => self.rotate_simple(*rotation).unwrap(),
        }
    }
}
