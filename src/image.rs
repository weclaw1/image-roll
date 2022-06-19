use std::path::Path;

use anyhow::{anyhow, Result};
use gtk::gdk_pixbuf::{InterpType, Pixbuf};

use crate::image_operation::{ApplyImageOperation, ImageOperation};

pub type Coordinates = (u32, u32);
pub type CoordinatesPair = (Coordinates, Coordinates);

pub struct Image {
    original_image_buffer: Option<Pixbuf>,
    current_image_buffer: Option<Pixbuf>,
    preview_image_buffer: Option<Pixbuf>,
    operations: Vec<ImageOperation>,
    current_operation_index: Option<usize>,
}

impl Image {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Image> {
        let image_buffer = Pixbuf::from_file(path)?;
        Ok(Image {
            original_image_buffer: Some(image_buffer.clone()),
            current_image_buffer: Some(image_buffer),
            preview_image_buffer: None,
            operations: Vec::new(),
            current_operation_index: None,
        })
    }

    pub fn save<P: AsRef<Path>>(&mut self, path: P, clear_operations: bool) -> Result<()> {
        let current_image_buffer = self
            .current_image_buffer
            .as_mut()
            .ok_or_else(|| anyhow!("Image buffer is missing!"))?;
        let extension = path
            .as_ref()
            .extension()
            .and_then(|extension| extension.to_str())
            .ok_or_else(|| anyhow!("File path doesn't have file extension"))?;
        let lowercase_extension = extension.to_lowercase();
        let file_type = match lowercase_extension.as_str() {
            file_type @ "jpeg"
            | file_type @ "png"
            | file_type @ "tiff"
            | file_type @ "ico"
            | file_type @ "bmp" => file_type,
            "jpg" => "jpeg",
            "tif" => "tiff",
            _ => "png",
        };

        let options: &[(&str, &str)] = match file_type {
            "jpeg" => &[("quality", "100")],
            "png" => &[("compression", "9")],
            _ => &[],
        };
        current_image_buffer.savev(path.as_ref(), file_type, options)?;
        if clear_operations {
            self.original_image_buffer = Some(current_image_buffer.clone());
            self.current_operation_index = None;
            self.operations.clear();
        }

        Ok(())
    }

    pub fn reload<P: AsRef<Path>>(self, path: P) -> Result<Image> {
        let original_image_buffer = Pixbuf::from_file(path)?;
        let mut current_image_buffer = original_image_buffer.clone();
        if let Some(current_operation_index) = self.current_operation_index {
            current_image_buffer = self
                .operations
                .iter()
                .take(current_operation_index + 1)
                .fold(current_image_buffer, |image, operation| {
                    image.apply_operation(operation).unwrap_or(image)
                });
        }
        Ok(Image {
            original_image_buffer: Some(original_image_buffer),
            current_image_buffer: Some(current_image_buffer),
            preview_image_buffer: None,
            operations: self.operations,
            current_operation_index: self.current_operation_index,
        })
    }

    pub fn remove_image_buffers(&mut self) {
        self.original_image_buffer = None;
        self.current_image_buffer = None;
        self.preview_image_buffer = None;
    }

    fn image_buffer_scale_to_fit(&self, canvas_width: u32, canvas_height: u32) -> Option<Pixbuf> {
        if let Some(image_buffer) = &self.current_image_buffer {
            let image_width = image_buffer.width() as f64;
            let image_height = image_buffer.height() as f64;
            let width_ratio = canvas_width as f64 / image_width;
            let height_ratio = canvas_height as f64 / image_height;
            let scale_ratio = width_ratio.min(height_ratio);
            image_buffer.scale_simple(
                (image_width * scale_ratio) as i32,
                (image_height * scale_ratio) as i32,
                InterpType::Nearest,
            )
        } else {
            None
        }
    }

    fn image_buffer_resize(&self, scale: u32) -> Option<Pixbuf> {
        if let Some(image_buffer) = &self.current_image_buffer {
            image_buffer.scale_simple(
                (image_buffer.width() as f64 * (scale as f64 / 100.0)) as i32,
                (image_buffer.height() as f64 * (scale as f64 / 100.0)) as i32,
                InterpType::Bilinear,
            )
        } else {
            None
        }
    }

    pub fn create_preview_image_buffer(&mut self, preview_size: PreviewSize) {
        self.preview_image_buffer = match preview_size {
            PreviewSize::BestFit(canvas_width, canvas_height) => {
                self.image_buffer_scale_to_fit(canvas_width, canvas_height)
            }
            PreviewSize::OriginalSize => self.current_image_buffer.clone(),
            PreviewSize::Resized(scale) => self.image_buffer_resize(scale),
        };
    }

    pub fn create_print_image_buffer(
        &self,
        canvas_width: u32,
        canvas_height: u32,
    ) -> Option<Pixbuf> {
        if let Some((image_width, image_height)) = self.image_size() {
            if image_width > canvas_width || image_height > canvas_height {
                self.image_buffer_scale_to_fit(canvas_width, canvas_height)
            } else {
                self.current_image_buffer.clone()
            }
        } else {
            None
        }
    }

    pub fn preview_image_buffer(&self) -> Option<&Pixbuf> {
        self.preview_image_buffer.as_ref()
    }

    pub fn current_image_buffer(&self) -> Option<&Pixbuf> {
        self.current_image_buffer.as_ref()
    }

    pub fn image_size(&self) -> Option<(u32, u32)> {
        self.current_image_buffer
            .as_ref()
            .map(|image_buffer| (image_buffer.width() as u32, image_buffer.height() as u32))
    }

    pub fn image_aspect_ratio(&self) -> Option<f64> {
        self.image_size()
            .map(|(image_width, image_height)| image_width as f64 / image_height as f64)
    }

    pub fn preview_image_buffer_size(&self) -> Option<(u32, u32)> {
        self.preview_image_buffer
            .as_ref()
            .map(|image_buffer| (image_buffer.width() as u32, image_buffer.height() as u32))
    }

    pub fn preview_coords_to_image_coords(
        &self,
        coords: CoordinatesPair,
    ) -> Option<CoordinatesPair> {
        let ((start_coord_x, start_coord_y), (end_coord_x, end_coord_y)) = coords;
        if let Some((image_width, image_height)) = self.image_size() {
            if let Some((preview_width, preview_height)) = self.preview_image_buffer_size() {
                Some((
                    (
                        (start_coord_x as f64 * (image_width as f64 / preview_width as f64)) as u32,
                        (start_coord_y as f64 * (image_height as f64 / preview_height as f64))
                            as u32,
                    ),
                    (
                        (end_coord_x as f64 * (image_width as f64 / preview_width as f64)) as u32,
                        (end_coord_y as f64 * (image_height as f64 / preview_height as f64)) as u32,
                    ),
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn has_operations(&self) -> bool {
        !self.operations.is_empty() && self.current_operation_index.is_some()
    }

    pub fn can_undo_operation(&self) -> bool {
        self.current_operation_index.is_some()
    }

    pub fn undo_operation(&mut self) {
        if self.can_undo_operation() {
            self.current_operation_index = self.current_operation_index.unwrap().checked_sub(1);
            self.current_image_buffer = Some(
                self.operations
                    .iter()
                    .take(
                        self.current_operation_index
                            .map_or(0, |operation_index| operation_index + 1),
                    )
                    .fold(
                        self.original_image_buffer.clone().unwrap(),
                        |image, operation| image.apply_operation(operation).unwrap_or(image),
                    ),
            );
        }
    }

    pub fn can_redo_operation(&self) -> bool {
        match self.current_operation_index {
            Some(operation_index) => operation_index + 1 < self.operations.len(),
            None => !self.operations.is_empty(),
        }
    }

    pub fn redo_operation(&mut self) {
        if self.can_redo_operation() {
            self.current_operation_index = self
                .current_operation_index
                .map_or(Some(0), |current_operation_index| {
                    Some(current_operation_index + 1)
                });
            self.current_image_buffer = Some(
                self.operations
                    .iter()
                    .take(self.current_operation_index.unwrap() + 1)
                    .fold(
                        self.original_image_buffer.clone().unwrap(),
                        |image, operation| image.apply_operation(operation).unwrap_or(image),
                    ),
            );
        }
    }
}

impl ApplyImageOperation for Image {
    type Result = Self;

    fn apply_operation(mut self, image_operation: &ImageOperation) -> Self::Result {
        if let Some(image_buffer) = self.current_image_buffer {
            let applied_operation_image_buffer = image_buffer.apply_operation(image_operation);
            if applied_operation_image_buffer.is_some() {
                if let Some(current_operation_index) = self.current_operation_index {
                    self.operations.truncate(current_operation_index + 1);
                }
                self.operations.push(*image_operation);
                self.current_operation_index = Some(self.operations.len() - 1);
            }
            self.current_image_buffer =
                Some(applied_operation_image_buffer.unwrap_or(image_buffer));
        }
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PreviewSize {
    BestFit(u32, u32),
    OriginalSize,
    Resized(u32),
}

impl From<PreviewSize> for String {
    fn from(value: PreviewSize) -> Self {
        match value {
            PreviewSize::BestFit(_, _) => String::from("Fit screen"),
            PreviewSize::OriginalSize => String::from("100%"),
            PreviewSize::Resized(value) => format!("{}%", value),
        }
    }
}

impl PreviewSize {
    pub fn smaller(self) -> Option<PreviewSize> {
        match self {
            PreviewSize::BestFit(_, _) => Some(PreviewSize::OriginalSize),
            PreviewSize::OriginalSize => Some(PreviewSize::Resized(75)),
            PreviewSize::Resized(value) if value > 200 => Some(PreviewSize::Resized(200)),
            PreviewSize::Resized(value) if value > 150 => Some(PreviewSize::Resized(150)),
            PreviewSize::Resized(value) if value > 133 => Some(PreviewSize::Resized(133)),
            PreviewSize::Resized(value) if value > 100 => Some(PreviewSize::OriginalSize),
            PreviewSize::Resized(value) if value > 75 => Some(PreviewSize::Resized(75)),
            PreviewSize::Resized(value) if value > 66 => Some(PreviewSize::Resized(66)),
            PreviewSize::Resized(value) if value > 50 => Some(PreviewSize::Resized(50)),
            PreviewSize::Resized(value) if value > 33 => Some(PreviewSize::Resized(33)),
            PreviewSize::Resized(value) if value > 25 => Some(PreviewSize::Resized(25)),
            PreviewSize::Resized(value) if value > 10 => Some(PreviewSize::Resized(10)),
            PreviewSize::Resized(value) if value > 5 => Some(PreviewSize::Resized(5)),
            PreviewSize::Resized(_) => None,
        }
    }

    pub fn smaller_by(self, value: u32) -> Option<PreviewSize> {
        let old_value = match self {
            PreviewSize::BestFit(_, _) => return Some(PreviewSize::OriginalSize),
            PreviewSize::OriginalSize => 100,
            PreviewSize::Resized(value) => value,
        };

        old_value
            .checked_sub(value)
            .filter(|value| value >= &5)
            .map(|value| {
                if value == 100 {
                    PreviewSize::OriginalSize
                } else {
                    PreviewSize::Resized(value)
                }
            })
    }

    pub fn can_be_smaller(&self) -> bool {
        !matches!(self, PreviewSize::Resized(value) if value <= &5)
    }

    pub fn larger(self) -> Option<PreviewSize> {
        match self {
            PreviewSize::BestFit(_, _) => Some(PreviewSize::OriginalSize),
            PreviewSize::OriginalSize => Some(PreviewSize::Resized(133)),
            PreviewSize::Resized(value) if value < 10 => Some(PreviewSize::Resized(10)),
            PreviewSize::Resized(value) if value < 25 => Some(PreviewSize::Resized(25)),
            PreviewSize::Resized(value) if value < 33 => Some(PreviewSize::Resized(33)),
            PreviewSize::Resized(value) if value < 50 => Some(PreviewSize::Resized(50)),
            PreviewSize::Resized(value) if value < 66 => Some(PreviewSize::Resized(66)),
            PreviewSize::Resized(value) if value < 75 => Some(PreviewSize::Resized(75)),
            PreviewSize::Resized(value) if value < 100 => Some(PreviewSize::OriginalSize),
            PreviewSize::Resized(value) if value < 133 => Some(PreviewSize::Resized(133)),
            PreviewSize::Resized(value) if value < 150 => Some(PreviewSize::Resized(150)),
            PreviewSize::Resized(value) if value < 200 => Some(PreviewSize::Resized(200)),
            PreviewSize::Resized(value) if value < 500 => Some(PreviewSize::Resized(500)),
            PreviewSize::Resized(_) => None,
        }
    }

    pub fn larger_by(self, value: u32) -> Option<PreviewSize> {
        let old_value = match self {
            PreviewSize::BestFit(_, _) => return Some(PreviewSize::OriginalSize),
            PreviewSize::OriginalSize => 100,
            PreviewSize::Resized(value) => value,
        };

        old_value
            .checked_add(value)
            .filter(|value| value <= &500)
            .map(|value| {
                if value == 100 {
                    PreviewSize::OriginalSize
                } else {
                    PreviewSize::Resized(value)
                }
            })
    }

    pub fn can_be_larger(&self) -> bool {
        !matches!(self, PreviewSize::Resized(value) if value >= &500)
    }
}

#[cfg(test)]
mod tests {
    use gtk::gdk_pixbuf::PixbufRotation;

    use crate::test_utils::TestResources;

    use super::*;

    const TEST_IMAGE: &[u8] = include_bytes!("resources/test/test_image.png");

    #[test]
    fn test_load_image() {
        let mut test_resources = TestResources::new("test/test_load_image");
        test_resources.add_file("test.png", TEST_IMAGE);

        let image = Image::load(test_resources.file_folder().join("test.png")).unwrap();
        assert_eq!(
            Pixbuf::from_file(test_resources.file_folder().join("test.png"))
                .unwrap()
                .pixel_bytes(),
            image.original_image_buffer.unwrap().pixel_bytes()
        );
        assert_eq!(
            Pixbuf::from_file(test_resources.file_folder().join("test.png"))
                .unwrap()
                .pixel_bytes(),
            image.current_image_buffer.unwrap().pixel_bytes()
        );
        assert!(image.operations.is_empty());
    }

    #[test]
    fn save_image() {
        let mut test_resources = TestResources::new("test/save_image");
        test_resources.add_file("test.png", TEST_IMAGE);

        let mut image = Image::load(test_resources.file_folder().join("test.png")).unwrap();
        let saved_file_path = test_resources.file_folder().join("test2.png");
        image.save(&saved_file_path, false).unwrap();
        assert!(std::fs::File::open(saved_file_path).is_ok());
    }

    #[test]
    fn test_save_image_without_clear_operations() {
        let mut test_resources =
            TestResources::new("test/test_save_image_without_clear_operations");
        test_resources.add_file("test.png", TEST_IMAGE);

        let mut image = Image::load(test_resources.file_folder().join("test.png")).unwrap();
        image = image.apply_operation(&ImageOperation::Rotate(PixbufRotation::Clockwise));
        assert!(image.has_operations());
        image
            .save(test_resources.file_folder().join("test2.png"), false)
            .unwrap();
        assert!(image.has_operations());
        assert_ne!(
            image.original_image_buffer.unwrap().pixel_bytes(),
            image.current_image_buffer.unwrap().pixel_bytes()
        )
    }

    #[test]
    fn test_save_image_with_clear_operations() {
        let mut test_resources = TestResources::new("test/test_save_image_with_clear_operations");
        test_resources.add_file("test.png", TEST_IMAGE);

        let mut image = Image::load(test_resources.file_folder().join("test.png")).unwrap();
        image = image.apply_operation(&ImageOperation::Rotate(PixbufRotation::Clockwise));
        assert!(image.has_operations());
        image
            .save(test_resources.file_folder().join("test2.png"), true)
            .unwrap();
        assert!(!image.has_operations());
        assert_eq!(
            image.original_image_buffer.unwrap().pixel_bytes(),
            image.current_image_buffer.unwrap().pixel_bytes()
        )
    }

    #[test]
    fn save_image_uses_extensions_for_file_types_supported_by_pixbuf_save() {
        let mut test_resources = TestResources::new(
            "test/save_image_uses_extensions_for_file_types_supported_by_pixbuf_save",
        );
        let file_extensions = vec!["png", "jpg", "tif", "ico", "bmp"];
        for extension in file_extensions {
            let file_name = format!("{}.{}", "test", extension);
            test_resources.add_file(&file_name, TEST_IMAGE);
            let mut image = Image::load(test_resources.file_folder().join(file_name)).unwrap();
            let saved_file_path = test_resources
                .file_folder()
                .join(format!("{}.{}", "test2", extension));
            image.save(&saved_file_path, false).unwrap();
            let saved_file_inferred_extension = infer::get_from_path(saved_file_path)
                .unwrap()
                .unwrap()
                .extension();

            assert_eq!(saved_file_inferred_extension, extension);
        }
    }

    #[test]
    fn file_extensions_jpg_and_jpeg_are_supported() {
        let mut test_resources =
            TestResources::new("test/save_file_extensions_jpg_and_jpeg_are_supported");
        test_resources.add_file("test.jpg", TEST_IMAGE);

        let mut image = Image::load(test_resources.file_folder().join("test.jpg")).unwrap();
        let saved_file_path = test_resources.file_folder().join("test2.jpg");
        image.save(&saved_file_path, false).unwrap();
        let saved_file_inferred_extension = infer::get_from_path(saved_file_path)
            .unwrap()
            .unwrap()
            .extension();
        assert_eq!(saved_file_inferred_extension, "jpg");

        test_resources.add_file("test.jpeg", TEST_IMAGE);

        let mut image = Image::load(test_resources.file_folder().join("test.jpeg")).unwrap();
        let saved_file_path = test_resources.file_folder().join("test2.jpeg");
        image.save(&saved_file_path, false).unwrap();
        let saved_file_inferred_extension = infer::get_from_path(saved_file_path)
            .unwrap()
            .unwrap()
            .extension();
        assert_eq!(saved_file_inferred_extension, "jpg");
    }

    #[test]
    fn test_image_reload() {
        let mut test_resources = TestResources::new("test/test_image_reload");
        test_resources.add_file("test.png", TEST_IMAGE);

        let mut image = Image::load(test_resources.file_folder().join("test.png")).unwrap();
        image = image.apply_operation(&ImageOperation::Rotate(PixbufRotation::Clockwise));
        let original_image_buffer = image.original_image_buffer.clone();
        let current_image_buffer = image.current_image_buffer.clone();
        image.remove_image_buffers();
        assert!(image.original_image_buffer.is_none() && image.current_image_buffer.is_none());

        image = image
            .reload(test_resources.file_folder().join("test.png"))
            .unwrap();
        assert_eq!(
            original_image_buffer.unwrap().pixel_bytes(),
            image.original_image_buffer.unwrap().pixel_bytes()
        );
        assert_eq!(
            current_image_buffer.unwrap().pixel_bytes(),
            image.current_image_buffer.unwrap().pixel_bytes()
        );
    }

    #[test]
    fn create_preview_original_size() {
        let mut test_resources = TestResources::new("test/create_preview_original_size");
        test_resources.add_file("test.png", TEST_IMAGE);

        let mut image = Image::load(test_resources.file_folder().join("test.png")).unwrap();
        image.create_preview_image_buffer(PreviewSize::OriginalSize);

        assert_eq!(
            image.current_image_buffer.unwrap().pixel_bytes(),
            image.preview_image_buffer.unwrap().pixel_bytes()
        );
    }

    #[test]
    fn create_preview_scale_to_fit() {
        let mut test_resources = TestResources::new("test/create_preview_scale_to_fit");
        test_resources.add_file("test.png", TEST_IMAGE);

        let mut image = Image::load(test_resources.file_folder().join("test.png")).unwrap();
        image = image.apply_operation(&ImageOperation::Resize((1000, 500)));
        image.create_preview_image_buffer(PreviewSize::BestFit(500, 500));

        assert_eq!((500, 250), image.preview_image_buffer_size().unwrap());
    }

    #[test]
    fn create_preview_resized() {
        let mut test_resources = TestResources::new("test/create_preview_resized");
        test_resources.add_file("test.png", TEST_IMAGE);

        let mut image = Image::load(test_resources.file_folder().join("test.png")).unwrap();
        image = image.apply_operation(&ImageOperation::Resize((100, 100)));
        image.create_preview_image_buffer(PreviewSize::Resized(90));

        assert_eq!((90, 90), image.preview_image_buffer_size().unwrap());
    }

    #[test]
    fn preview_coords_to_image_coords() {
        let mut test_resources = TestResources::new("test/preview_coords_to_image_coords");
        test_resources.add_file("test.png", TEST_IMAGE);

        let mut image = Image::load(test_resources.file_folder().join("test.png")).unwrap();
        image = image.apply_operation(&ImageOperation::Resize((100, 100)));
        image.create_preview_image_buffer(PreviewSize::Resized(200));

        assert_eq!(
            ((10, 10), (20, 20)),
            image
                .preview_coords_to_image_coords(((20, 20), (40, 40)))
                .unwrap()
        );
    }

    #[test]
    fn undo_operation() {
        let mut test_resources = TestResources::new("test/undo_operation");
        test_resources.add_file("test.png", TEST_IMAGE);

        let mut image = Image::load(test_resources.file_folder().join("test.png")).unwrap();
        image = image.apply_operation(&ImageOperation::Resize((100, 100)));
        image = image.apply_operation(&ImageOperation::Rotate(PixbufRotation::Clockwise));

        assert!(image.can_undo_operation());
        image.undo_operation();
        assert!(
            image.can_redo_operation()
                && image.current_operation_index == Some(0)
                && image.operations.len() == 2
        );
        image.undo_operation();
        assert!(
            image.can_redo_operation()
                && image.current_operation_index == None
                && image.operations.len() == 2
        );
    }

    #[test]
    fn redo_operation() {
        let mut test_resources = TestResources::new("test/redo_operation");
        test_resources.add_file("test.png", TEST_IMAGE);

        let mut image = Image::load(test_resources.file_folder().join("test.png")).unwrap();
        image = image.apply_operation(&ImageOperation::Resize((100, 100)));
        image = image.apply_operation(&ImageOperation::Rotate(PixbufRotation::Clockwise));

        assert!(!image.can_redo_operation());
        image.undo_operation();
        assert!(
            image.can_redo_operation()
                && image.current_operation_index == Some(0)
                && image.operations.len() == 2
        );
        image.redo_operation();
        assert!(
            !image.can_redo_operation()
                && image.current_operation_index == Some(1)
                && image.operations.len() == 2
        );
    }

    #[test]
    fn apply_operation() {
        let mut test_resources = TestResources::new("test/apply_operation");
        test_resources.add_file("test.png", TEST_IMAGE);

        let mut image = Image::load(test_resources.file_folder().join("test.png")).unwrap();

        assert!(image.operations.is_empty() && image.current_operation_index.is_none());
        image = image.apply_operation(&ImageOperation::Resize((100, 100)));
        assert!(image.operations.len() == 1 && image.current_operation_index == Some(0));
        assert!(
            image.original_image_buffer.unwrap().pixel_bytes()
                != image.current_image_buffer.unwrap().pixel_bytes()
        );
    }
}
