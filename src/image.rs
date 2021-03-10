use std::path::Path;

use approx::abs_diff_eq;
use gdk_pixbuf::{InterpType, Pixbuf};

use crate::image_operation::{ApplyImageOperation, ImageOperation};

pub struct Image {
    image_buffer: Option<Pixbuf>,
    preview_image_buffer: Option<Pixbuf>,
    operations: Vec<ImageOperation>,
}

impl Image {
    pub fn load<P: AsRef<Path>>(path: P) -> Image {
        let image_buffer = Pixbuf::from_file(path).expect("Couldn't load image");
        let preview_image_buffer = image_buffer.clone();
        Image {
            image_buffer: Some(image_buffer),
            preview_image_buffer: Some(preview_image_buffer),
            operations: Vec::new(),
        }
    }

    pub fn reload<P: AsRef<Path>>(self, path: P) -> Image {
        let mut image_buffer = Pixbuf::from_file(path).expect("Couldn't load image");
        image_buffer = self
            .operations
            .iter()
            .fold(image_buffer, |image, operation| {
                image.apply_operation(operation)
            });
        let preview_image_buffer = image_buffer.clone();
        Image {
            image_buffer: Some(image_buffer),
            preview_image_buffer: Some(preview_image_buffer),
            operations: self.operations,
        }
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

    pub fn preview_image_buffer_size(&self) -> Option<(i32, i32)> {
        self.preview_image_buffer
            .as_ref()
            .map(|image_buffer| (image_buffer.get_width(), image_buffer.get_height()))
    }
}

impl ApplyImageOperation for Image {
    fn apply_operation(mut self, image_operation: &ImageOperation) -> Image {
        self.image_buffer = self
            .image_buffer
            .map(|buffer| buffer.apply_operation(image_operation));
        self.operations.push(*image_operation);
        self
    }
}

#[derive(Clone, Copy)]
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
