use std::cmp;

use gdk_pixbuf::{Pixbuf, PixbufRotation};

use crate::image::CoordinatesPair;

#[derive(Copy, Clone)]
pub enum ImageOperation {
    Rotate(PixbufRotation),
    Crop(CoordinatesPair),
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
                self.new_subpixbuf(x, y, width, height)
            }
        }
    }
}
