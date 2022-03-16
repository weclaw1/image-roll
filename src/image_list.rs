use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    path::{Path, PathBuf},
};

use crate::image::Image;

use anyhow::{anyhow, Result};

pub struct ImageList {
    images: HashMap<PathBuf, Image>,
    current_image_path: Option<PathBuf>,
}

impl ImageList {
    pub fn new() -> Self {
        Self {
            images: HashMap::new(),
            current_image_path: None,
        }
    }

    pub fn remove(&mut self, key: &Path) -> Option<Image> {
        self.images.remove(key)
    }

    pub fn insert(&mut self, key: PathBuf, value: Image) {
        self.images.insert(key, value);
    }

    pub fn set_current_image_path(&mut self, current_image_path: Option<PathBuf>) {
        self.current_image_path = current_image_path;
    }

    // pub fn current_image(&self) -> Option<&Image> {
    //     self.current_image_path.as_ref().map(|image_path| self.images.get(image_path)).flatten()
    // }

    pub fn remove_current_image(&mut self) -> Option<Image> {
        self.current_image_path
            .clone()
            .and_then(|image_path| self.remove(&image_path))
    }

    pub fn current_image_mut(&mut self) -> Option<&mut Image> {
        self.current_image_path
            .clone()
            .and_then(move |image_path| self.images.get_mut(&image_path))
    }

    pub fn current_image(&self) -> Option<&Image> {
        self.current_image_path
            .as_ref()
            .and_then(|image_path| self.images.get(image_path))
    }

    pub fn current_image_path(&self) -> Option<PathBuf> {
        self.current_image_path.clone()
    }

    pub fn save_current_image(&mut self, filename: Option<PathBuf>) -> Result<()> {
        let (filename, clear_operations) = if let Some(filename) = filename {
            (filename, false)
        } else {
            (
                self.current_image_path
                    .clone()
                    .ok_or_else(|| anyhow!("Current image path is not set"))?,
                true,
            )
        };

        let current_image = self
            .current_image_mut()
            .ok_or_else(|| anyhow!("Couldn't load current image"))?;

        current_image.save(filename, clear_operations)?;
        Ok(())
    }

    pub fn copy_current_image(&self, clipboard: &gtk::Clipboard) {
        if let Some(current_image) = self.current_image() {
            if let Some(buffer) = current_image.current_image_buffer() {
                clipboard.set_image(buffer);
            }
        }
    }
}

impl Index<&PathBuf> for ImageList {
    type Output = Image;

    fn index(&self, index: &PathBuf) -> &Self::Output {
        &self.images[index]
    }
}

impl IndexMut<&PathBuf> for ImageList {
    fn index_mut(&mut self, index: &PathBuf) -> &mut Self::Output {
        self.images.get_mut(index).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        image_operation::{ApplyImageOperation, ImageOperation},
        test_utils::TestResources,
    };

    use super::*;

    const TEST_IMAGE: &[u8] = include_bytes!("resources/test/test_image.png");

    #[test]
    fn save_current_image_overwrites_image_at_current_image_path_when_filename_is_set_to_none() {
        let mut test_resources = TestResources::new("test/save_current_image_overwrites_image_at_current_image_path_when_filename_is_set_to_none");
        test_resources.add_file("test.png", TEST_IMAGE);

        let image_path = test_resources.file_folder().join("test.png");

        let creation_date = std::fs::File::open(&image_path)
            .unwrap()
            .metadata()
            .unwrap()
            .modified()
            .unwrap();

        let image = Image::load(&image_path).unwrap();

        let mut image_list = ImageList::new();
        image_list.insert(image_path.clone(), image);
        image_list.set_current_image_path(Some(image_path.clone()));
        image_list.save_current_image(None).unwrap();

        let modification_date = std::fs::File::open(&image_path)
            .unwrap()
            .metadata()
            .unwrap()
            .modified()
            .unwrap();
        assert!(modification_date > creation_date);
    }

    #[test]
    fn save_current_image_creates_a_new_image_when_filename_is_set() {
        let mut test_resources =
            TestResources::new("test/save_current_image_creates_a_new_image_when_filename_is_set");
        test_resources.add_file("test.png", TEST_IMAGE);

        let image_path = test_resources.file_folder().join("test.png");
        let image = Image::load(&image_path).unwrap();

        let mut image_list = ImageList::new();
        image_list.insert(image_path.clone(), image);
        image_list.set_current_image_path(Some(image_path.clone()));

        let new_image_path = test_resources.file_folder().join("test2.png");
        image_list
            .save_current_image(Some(new_image_path.clone()))
            .unwrap();

        assert!(std::fs::File::open(new_image_path).is_ok());
    }

    #[test]
    fn save_current_image_clears_image_operations_when_filename_is_set_to_none() {
        let mut test_resources = TestResources::new(
            "test/save_current_image_clears_image_operations_when_filename_is_set_to_none",
        );
        test_resources.add_file("test.png", TEST_IMAGE);

        let image_path = test_resources.file_folder().join("test.png");

        let mut image = Image::load(&image_path).unwrap();
        image = image.apply_operation(&ImageOperation::Resize((10, 10)));

        let mut image_list = ImageList::new();
        image_list.insert(image_path.clone(), image);
        image_list.set_current_image_path(Some(image_path.clone()));

        assert!(image_list.current_image().unwrap().has_operations());

        image_list.save_current_image(None).unwrap();

        assert!(!image_list.current_image().unwrap().has_operations());
    }

    #[test]
    fn save_current_image_does_not_clear_image_operations_when_filename_is_set() {
        let mut test_resources = TestResources::new(
            "test/save_current_image_does_not_clear_image_operations_when_filename_is_set",
        );
        test_resources.add_file("test.png", TEST_IMAGE);

        let image_path = test_resources.file_folder().join("test.png");

        let mut image = Image::load(&image_path).unwrap();
        image = image.apply_operation(&ImageOperation::Resize((10, 10)));

        let mut image_list = ImageList::new();
        image_list.insert(image_path.clone(), image);
        image_list.set_current_image_path(Some(image_path.clone()));

        assert!(image_list.current_image().unwrap().has_operations());

        image_list
            .save_current_image(Some(test_resources.file_folder().join("test2.png")))
            .unwrap();

        assert!(image_list.current_image().unwrap().has_operations());
    }
}
