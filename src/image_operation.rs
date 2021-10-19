use std::cmp;

use gtk::gdk_pixbuf::{InterpType, Pixbuf, PixbufRotation};

use crate::image::CoordinatesPair;

#[derive(Copy, Clone, Debug)]
pub enum ImageOperation {
    Rotate(PixbufRotation),
    Crop(CoordinatesPair),
    Resize((u32, u32)),
}

pub trait ApplyImageOperation {
    type Result;

    fn apply_operation(self, image_operation: &ImageOperation) -> Self::Result;
}

impl ApplyImageOperation for &Pixbuf {
    type Result = Option<Pixbuf>;

    fn apply_operation(self, image_operation: &ImageOperation) -> Self::Result {
        match image_operation {
            ImageOperation::Rotate(rotation) => self.rotate_simple(*rotation),
            ImageOperation::Crop((
                (start_position_x, start_position_y),
                (end_position_x, end_position_y),
            )) => {
                let x = *cmp::min(start_position_x, end_position_x);
                let y = *cmp::min(start_position_y, end_position_y);
                let width = *cmp::max(start_position_x, end_position_x) - x;
                let height = *cmp::max(start_position_y, end_position_y) - y;
                self.new_subpixbuf(x as i32, y as i32, width as i32, height as i32)
            }
            ImageOperation::Resize((width, height)) => {
                self.scale_simple(*width as i32, *height as i32, InterpType::Bilinear)
            }
        }
    }
}
