use std::path::Path;

use approx::abs_diff_eq;
use gdk_pixbuf::{InterpType, Pixbuf};

use anyhow::{anyhow, Result};

use crate::image_operation::{ApplyImageOperation, ImageOperation};

pub type Coordinates = (i32, i32);
pub type CoordinatesPair = (Coordinates, Coordinates);

pub struct Image {
    image_buffer: Option<Pixbuf>,
    preview_image_buffer: Option<Pixbuf>,
    operations: Vec<ImageOperation>,
}

impl Image {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Image> {
        let image_buffer = Pixbuf::from_file(path)?;
        let preview_image_buffer = image_buffer.clone();
        Ok(Image {
            image_buffer: Some(image_buffer),
            preview_image_buffer: Some(preview_image_buffer),
            operations: Vec::new(),
        })
    }

    pub fn save<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let image_buffer = self
            .image_buffer
            .as_mut()
            .ok_or_else(|| anyhow!("Couldn't load image buffer"))?;
        let extension = path
            .as_ref()
            .extension()
            .map(|extension| extension.to_str())
            .flatten()
            .ok_or_else(|| anyhow!("File path doesn't have file extension"))?;
        let lowercase_extension = extension.to_lowercase();
        let file_type = match lowercase_extension.as_str() {
            file_type @ "jpeg"
            | file_type @ "png"
            | file_type @ "tiff"
            | file_type @ "ico"
            | file_type @ "bmp" => file_type,
            "jpg" => "jpeg",
            _ => "png",
        };

        let options: &[(&str, &str)] = match file_type {
            "jpeg" => &[("quality", "100")],
            "png" => &[("compression", "9")],
            _ => &[],
        };
        image_buffer.savev(path.as_ref(), file_type, options)?;
        self.operations.clear();
        Ok(())
    }

    pub fn reload<P: AsRef<Path>>(self, path: P) -> Result<Image> {
        let mut image_buffer = Pixbuf::from_file(path)?;
        image_buffer = self
            .operations
            .iter()
            .fold(image_buffer, |image, operation| {
                image.apply_operation(operation).unwrap_or(image)
            });
        let preview_image_buffer = image_buffer.clone();
        Ok(Image {
            image_buffer: Some(image_buffer),
            preview_image_buffer: Some(preview_image_buffer),
            operations: self.operations,
        })
    }

    // pub fn image_buffer(&self) -> &Pixbuf {
    //     &self.image_buffer
    // }

    pub fn remove_image_buffers(&mut self) {
        self.image_buffer = None;
        self.preview_image_buffer = None;
    }

    fn image_buffer_scale_to_fit(&self, canvas_width: i32, canvas_height: i32) -> Option<Pixbuf> {
        if let Some(image_buffer) = &self.image_buffer {
            let image_width = image_buffer.get_width();
            let image_height = image_buffer.get_height();
            let width_ratio = canvas_width as f32 / image_width as f32;
            let height_ratio = canvas_height as f32 / image_height as f32;
            let scale_ratio = width_ratio.min(height_ratio);
            image_buffer.scale_simple(
                (image_width as f32 * scale_ratio) as i32,
                (image_height as f32 * scale_ratio) as i32,
                InterpType::Bilinear,
            )
        } else {
            None
        }
    }

    fn image_buffer_resize(&self, scale: f32) -> Option<Pixbuf> {
        if let Some(image_buffer) = &self.image_buffer {
            image_buffer.scale_simple(
                (image_buffer.get_width() as f32 * scale) as i32,
                (image_buffer.get_height() as f32 * scale) as i32,
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
            PreviewSize::OriginalSize => self.image_buffer.clone(),
            PreviewSize::Resized(scale) => self.image_buffer_resize(scale),
        };
    }

    pub fn preview_image_buffer(&self) -> Option<&Pixbuf> {
        self.preview_image_buffer.as_ref()
    }

    pub fn image_size(&self) -> Option<(i32, i32)> {
        self.image_buffer
            .as_ref()
            .map(|image_buffer| (image_buffer.get_width(), image_buffer.get_height()))
    }

    pub fn image_aspect_ratio(&self) -> Option<f64> {
        self.image_size()
            .map(|(image_width, image_height)| image_width as f64 / image_height as f64)
    }

    pub fn preview_image_buffer_size(&self) -> Option<(i32, i32)> {
        self.preview_image_buffer
            .as_ref()
            .map(|image_buffer| (image_buffer.get_width(), image_buffer.get_height()))
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
                        (start_coord_x as f32 * (image_width as f32 / preview_width as f32)) as i32,
                        (start_coord_y as f32 * (image_height as f32 / preview_height as f32))
                            as i32,
                    ),
                    (
                        (end_coord_x as f32 * (image_width as f32 / preview_width as f32)) as i32,
                        (end_coord_y as f32 * (image_height as f32 / preview_height as f32)) as i32,
                    ),
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn has_unsaved_operations(&self) -> bool {
        !self.operations.is_empty()
    }
}

impl ApplyImageOperation for Image {
    type Result = Self;

    fn apply_operation(mut self, image_operation: &ImageOperation) -> Self::Result {
        if let Some(image_buffer) = self.image_buffer {
            let applied_operation_image_buffer = image_buffer.apply_operation(image_operation);
            if applied_operation_image_buffer.is_some() {
                self.operations.push(*image_operation);
            }
            self.image_buffer = Some(applied_operation_image_buffer.unwrap_or(image_buffer));
        }
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub enum PreviewSize {
    BestFit(i32, i32),
    OriginalSize,
    Resized(f32),
}

impl From<&str> for PreviewSize {
    fn from(value: &str) -> Self {
        match value {
            "preview_fit_screen" => PreviewSize::BestFit(0, 0),
            "preview_10" => PreviewSize::Resized(0.1),
            "preview_25" => PreviewSize::Resized(0.25),
            "preview_33" => PreviewSize::Resized(0.33),
            "preview_50" => PreviewSize::Resized(0.5),
            "preview_66" => PreviewSize::Resized(0.66),
            "preview_75" => PreviewSize::Resized(0.75),
            "preview_100" => PreviewSize::OriginalSize,
            "preview_133" => PreviewSize::Resized(1.33),
            "preview_150" => PreviewSize::Resized(1.5),
            "preview_200" => PreviewSize::Resized(2.0),
            _ => panic!("Cannot create PreviewSize from value: {}", value),
        }
    }
}

impl From<PreviewSize> for String {
    fn from(value: PreviewSize) -> Self {
        match value {
            PreviewSize::BestFit(_, _) => String::from("preview_fit_screen"),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.1) => String::from("preview_10"),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.25) => String::from("preview_25"),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.33) => String::from("preview_33"),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.5) => String::from("preview_50"),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.66) => String::from("preview_66"),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.75) => String::from("preview_75"),
            PreviewSize::OriginalSize => String::from("preview_100"),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 1.33) => String::from("preview_133"),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 1.5) => String::from("preview_150"),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 2.0) => String::from("preview_200"),
            _ => panic!("Cannot create PreviewSize for this value"),
        }
    }
}

impl PreviewSize {
    pub fn smaller(self) -> PreviewSize {
        match self {
            PreviewSize::BestFit(_, _) => PreviewSize::OriginalSize,
            PreviewSize::OriginalSize => PreviewSize::Resized(0.75),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 2.0) => PreviewSize::Resized(1.5),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 1.5) => PreviewSize::Resized(1.33),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 1.33) => PreviewSize::OriginalSize,
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.75) => PreviewSize::Resized(0.66),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.66) => PreviewSize::Resized(0.5),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.5) => PreviewSize::Resized(0.33),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.33) => PreviewSize::Resized(0.25),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.25) => PreviewSize::Resized(0.1),
            PreviewSize::Resized(_) => panic!("Preview size cannot be smaller than 10%"),
        }
    }

    pub fn can_be_smaller(&self) -> bool {
        !matches!(self, PreviewSize::Resized(value) if value <= &0.1)
    }

    pub fn larger(self) -> PreviewSize {
        match self {
            PreviewSize::BestFit(_, _) => PreviewSize::OriginalSize,
            PreviewSize::OriginalSize => PreviewSize::Resized(1.33),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.1) => PreviewSize::Resized(0.25),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.25) => PreviewSize::Resized(0.33),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.33) => PreviewSize::Resized(0.5),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.5) => PreviewSize::Resized(0.66),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.66) => PreviewSize::Resized(0.75),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 0.75) => PreviewSize::OriginalSize,
            PreviewSize::Resized(value) if abs_diff_eq!(value, 1.33) => PreviewSize::Resized(1.5),
            PreviewSize::Resized(value) if abs_diff_eq!(value, 1.5) => PreviewSize::Resized(2.0),
            PreviewSize::Resized(_) => panic!("Preview size cannot be larger than 200%"),
        }
    }

    pub fn can_be_larger(&self) -> bool {
        !matches!(self, PreviewSize::Resized(value) if value >= &2.0)
    }
}
