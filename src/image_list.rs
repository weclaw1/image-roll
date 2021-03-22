use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    path::PathBuf,
};

use crate::image::Image;

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

    pub fn remove(&mut self, key: &PathBuf) -> Option<Image> {
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
            .map(|image_path| self.remove(&image_path))
            .flatten()
    }

    pub fn current_image_mut(&mut self) -> Option<&mut Image> {
        self.current_image_path
            .clone()
            .map(move |image_path| self.images.get_mut(&image_path))
            .flatten()
    }

    pub fn current_image(&self) -> Option<&Image> {
        self.current_image_path
            .as_ref()
            .map(|image_path| self.images.get(image_path))
            .flatten()
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
