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

#[cfg(test)]
mod tests {
    use crate::test_utils::TestResources;

    use super::*;

    const TEST_IMAGE: &[u8] = include_bytes!("resources/test/test_image.png");

    #[test]
    fn test_apply_rotate_image_operation_on_pixbuf() {
        let mut test_resources =
            TestResources::new("test/test_apply_rotate_image_operation_on_pixbuf");
        test_resources.add_file("test.png", TEST_IMAGE);

        let pixbuf = Pixbuf::from_file(test_resources.file_folder().join("test.png")).unwrap();
        let image_operation = ImageOperation::Rotate(PixbufRotation::Clockwise);

        assert_eq!(
            pixbuf
                .rotate_simple(PixbufRotation::Clockwise)
                .unwrap()
                .pixel_bytes(),
            pixbuf
                .apply_operation(&image_operation)
                .unwrap()
                .pixel_bytes()
        );
    }

    #[test]
    fn test_apply_crop_image_operation_on_pixbuf() {
        let mut test_resources =
            TestResources::new("test/test_apply_crop_image_operation_on_pixbuf");
        test_resources.add_file("test.png", TEST_IMAGE);

        let pixbuf = Pixbuf::from_file(test_resources.file_folder().join("test.png")).unwrap();
        let image_operation = ImageOperation::Crop(((10, 10), (20, 20)));

        assert_eq!(
            pixbuf.new_subpixbuf(10, 10, 10, 10).unwrap().pixel_bytes(),
            pixbuf
                .apply_operation(&image_operation)
                .unwrap()
                .pixel_bytes()
        );
    }

    #[test]
    fn test_apply_resize_image_operation_on_pixbuf() {
        let mut test_resources =
            TestResources::new("test/test_apply_resize_image_operation_on_pixbuf");
        test_resources.add_file("test.png", TEST_IMAGE);

        let pixbuf = Pixbuf::from_file(test_resources.file_folder().join("test.png")).unwrap();
        let image_operation = ImageOperation::Resize((10, 10));

        assert_eq!(
            pixbuf
                .scale_simple(10, 10, InterpType::Bilinear)
                .unwrap()
                .pixel_bytes(),
            pixbuf
                .apply_operation(&image_operation)
                .unwrap()
                .pixel_bytes()
        );
    }
}
