use std::{
    collections::HashMap,
    ops::{Index, IndexMut},
    path::{Path, PathBuf},
};

use gtk::{
    gio::{Cancellable, File},
    prelude::FileExt,
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

    pub fn current_image_path(&self) -> Option<PathBuf> {
        self.current_image_path.clone()
    }

    pub fn save_current_image(&mut self, filename: Option<PathBuf>) -> Result<()> {
        let (filename, clear_buffer) = if let Some(filename) = filename {
            (filename, false)
        } else {
            (self.current_image_path.clone().unwrap(), true)
        };

        let current_image = self
            .current_image_mut()
            .ok_or_else(|| anyhow!("Couldn't load current image"))?;

        current_image.save(filename, clear_buffer)?;
        Ok(())
    }

    pub fn delete_current_image(&mut self) -> Result<String> {
        let current_image_path = self.current_image_path.as_ref().unwrap();
        let current_image_file = File::for_path(current_image_path);
        let current_image_name = current_image_file.parse_name();
        current_image_file.trash::<Cancellable>(None)?;
        self.images.remove(current_image_path);
        Ok(current_image_name.to_string())
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
