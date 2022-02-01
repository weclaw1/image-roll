use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use gtk::{
    gio::{self, Cancellable, FileMonitorFlags, FileQueryInfoFlags, FileType},
    prelude::FileExt,
};

pub struct FileList {
    file_list: Vec<gio::FileInfo>,
    current_file: Option<(usize, gio::File)>,
    current_folder: Option<gio::File>,
    current_folder_monitor: Option<gio::FileMonitor>,
}

impl FileList {
    pub fn new(current_file: Option<gio::File>) -> Result<FileList> {
        if let Some(current_file) = current_file {
            let current_folder = current_file.parent().ok_or_else(|| {
                anyhow!(
                    "Couldn't get parent folder for file: {}",
                    current_file.parse_name()
                )
            })?;
            let mut file_list: Vec<gio::FileInfo> = FileList::enumerate_files(&current_folder)?;
            file_list.sort_by_key(|file| file.name().file_name().unwrap().to_str().unwrap().to_owned());
            let current_file_index = file_list
                .iter()
                .position(|file| file.name() == current_file.basename().unwrap_or_default())
                .ok_or_else(|| {
                    anyhow!(
                        "Couldn't find {} in enumerated files",
                        current_file.parse_name()
                    )
                })?;
            let folder_monitor = current_folder
                .monitor_directory::<Cancellable>(FileMonitorFlags::NONE, None)
                .context("Couldn't get monitor for current file directory")?;

            Ok(FileList {
                file_list,
                current_file: Some((current_file_index, current_file)),
                current_folder: Some(current_folder),
                current_folder_monitor: Some(folder_monitor),
            })
        } else {
            Ok(FileList {
                file_list: Vec::new(),
                current_file: None,
                current_folder: None,
                current_folder_monitor: None,
            })
        }
    }

    pub fn refresh(&mut self) -> Result<()> {
        if let Some(current_folder) = &self.current_folder {
            if !current_folder.query_exists::<Cancellable>(None) {
                self.file_list = Vec::new();
                self.current_file = None;
                self.current_folder = None;
                return Ok(());
            }
            self.file_list = FileList::enumerate_files(current_folder)?;
            self.file_list.sort_by_key(|file| file.name().file_name().unwrap().to_str().unwrap().to_owned());

            match &self.current_file {
                Some((_, current_file)) => {
                    let file_index = self.file_list.iter().position(|file| {
                        file.name() == current_file.basename().unwrap_or_default()
                    });
                    if let Some(file_index) = file_index {
                        self.current_file = Some((file_index, self.current_file.take().unwrap().1));
                    } else {
                        self.next();
                    }
                }
                None => self.next(),
            }
        }
        Ok(())
    }

    pub fn next(&mut self) {
        if let Some(current_folder) = &self.current_folder {
            self.current_file = match self.current_file.take() {
                Some((_, _)) if self.file_list.is_empty() => None,
                Some((index, _)) if index + 1 < self.file_list.len() => Some((
                    index + 1,
                    current_folder.child(self.file_list[index + 1].name()),
                )),
                Some((index, _)) if index + 1 >= self.file_list.len() => {
                    Some((0, current_folder.child(self.file_list[0].name())))
                }
                None if !self.file_list.is_empty() => {
                    Some((0, current_folder.child(self.file_list[0].name())))
                }
                _ => None,
            }
        }
    }

    pub fn previous(&mut self) {
        if let Some(current_folder) = &self.current_folder {
            self.current_file = match self.current_file.take() {
                Some((_, _)) if self.file_list.is_empty() => None,
                Some((index, _)) if index as i64 > 0 => Some((
                    index - 1,
                    current_folder.child(self.file_list[index - 1].name()),
                )),
                Some((index, _)) if index as i64 - 1 < 0 => Some((
                    self.file_list.len() - 1,
                    current_folder.child(self.file_list[self.file_list.len() - 1].name()),
                )),
                None if !self.file_list.is_empty() => {
                    Some((0, current_folder.child(self.file_list[0].name())))
                }
                _ => None,
            }
        }
    }

    // pub fn current_folder(&self) -> Option<&gio::File> {
    //     self.current_folder.as_ref()
    // }

    #[allow(dead_code)]
    pub fn current_file(&self) -> Option<&gio::File> {
        self.current_file.as_ref().map(|(_, file)| file)
    }

    pub fn current_file_path(&self) -> Option<PathBuf> {
        self.current_file
            .as_ref()
            .map(|(_, file)| file.path())
            .flatten()
    }

    pub fn len(&self) -> usize {
        self.file_list.len()
    }

    fn enumerate_files(folder: &gio::File) -> Result<Vec<gio::FileInfo>> {
        Ok(folder
            .enumerate_children::<Cancellable>("standard::*", FileQueryInfoFlags::NONE, None)?
            .into_iter()
            .filter_map(|file| file.ok())
            .filter(|file| file.file_type() == FileType::Regular)
            .filter(|file| {
                file.content_type()
                    .filter(|content_type| content_type.to_string().starts_with("image"))
                    .is_some()
            })
            .collect())
    }

    pub fn current_folder_monitor_mut(&mut self) -> Option<&mut gio::FileMonitor> {
        self.current_folder_monitor.as_mut()
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use rand::{Rng, distributions::Alphanumeric};

    use crate::test_utils::TestResources;

    use super::*;

    const TEST_IMAGE: &[u8] = include_bytes!("resources/test/test_image.png");

    #[test]
    fn file_list_contains_image_files() {
        let mut test_resources = TestResources::new("test/file_list_contains_image_files");
        test_resources.add_file("test.png", TEST_IMAGE);
        test_resources.add_file("tes2.png", TEST_IMAGE);

        let file_list = FileList::new(Some(gio::File::for_path(test_resources.file_folder().join("test.png")))).unwrap();

        assert_eq!(2, file_list.len());
    }

    #[test]
    fn file_list_does_not_contain_other_files() {
        let mut test_resources = TestResources::new("test/file_list_does_not_contain_other_files");
        test_resources.add_file("test.png", TEST_IMAGE);
        test_resources.add_file("test2.png", TEST_IMAGE);
        test_resources.add_file("test.txt", "test");

        let file_list = FileList::new(Some(gio::File::for_path(test_resources.file_folder().join("test.png")))).unwrap();

        assert_eq!(2, file_list.len());
    }

    #[test]
    fn file_list_contains_images_without_extension() {
        let mut test_resources = TestResources::new("test/file_list_contains_images_without_extension");
        test_resources.add_file("test", TEST_IMAGE);
        test_resources.add_file("test2", TEST_IMAGE);

        let file_list = FileList::new(Some(gio::File::for_path(test_resources.file_folder().join("test")))).unwrap();

        assert_eq!(2, file_list.len());
    }

    #[test]
    fn file_list_does_not_contain_other_files_without_extension() {
        let mut test_resources = TestResources::new("test/file_list_does_not_contain_other_files_without_extension");
        test_resources.add_file("test", TEST_IMAGE);
        test_resources.add_file("test2", TEST_IMAGE);
        test_resources.add_file("test", TEST_IMAGE);
        test_resources.add_file("testtxt", "test");

        let file_list = FileList::new(Some(gio::File::for_path(test_resources.file_folder().join("test")))).unwrap();

        assert_eq!(2, file_list.len());
    }

    #[test]
    fn file_list_is_in_alphabetical_order() {
        let mut test_resources = TestResources::new("test/file_list_is_in_alphabetical_order");

        let mut random_file_names: Vec<String> = rand::thread_rng().sample_iter(Alphanumeric).map(char::from)
                                                                   .chunks(10)
                                                                   .into_iter()
                                                                   .map(|chunk| chunk.collect::<String>())
                                                                   .take(100)
                                                                   .map(|name| format!("{}.{}", name, "png")).collect();

        random_file_names.iter().for_each(|file_name| test_resources.add_file(file_name, TEST_IMAGE));

        random_file_names.sort();

        let mut file_list = FileList::new(Some(gio::File::for_path(test_resources.file_folder().join(random_file_names.first().unwrap())))).unwrap();

        assert_eq!(100, file_list.len());

        for file_name in random_file_names.iter() {
            assert_eq!(file_name, file_list.current_file().unwrap().basename().unwrap().to_str().unwrap());
            file_list.next();
        }
    }

    #[test]
    fn refresh_file_list_loads_new_images() {
        let mut test_resources = TestResources::new("test/refresh_file_list_loads_new_images");
        test_resources.add_file("test.png", TEST_IMAGE);

        let mut file_list = FileList::new(Some(gio::File::for_path(test_resources.file_folder().join("test.png")))).unwrap();
        assert_eq!(1, file_list.len());

        test_resources.add_file("test2.png", TEST_IMAGE);
        file_list.refresh().unwrap();

        assert_eq!(2, file_list.len());
    }

    #[test]
    fn refresh_file_list_removes_deleted_images() {
        let mut test_resources = TestResources::new("test/refresh_file_list_removes_deleted_images");
        test_resources.add_file("test.png", TEST_IMAGE);
        test_resources.add_file("test2.png", TEST_IMAGE);

        let mut file_list = FileList::new(Some(gio::File::for_path(test_resources.file_folder().join("test.png")))).unwrap();
        assert_eq!(2, file_list.len());

        test_resources.remove_file("test2.png");
        file_list.refresh().unwrap();

        assert_eq!(1, file_list.len());
    }

    #[test]
    fn test_change_to_next_image() {
        let mut empty_file_list = FileList::new(None).unwrap();
        assert!(empty_file_list.current_file().is_none());
        empty_file_list.next();
        assert!(empty_file_list.current_file().is_none());

        let mut test_resources = TestResources::new("test/test_change_to_next_image");
        test_resources.add_file("test1.png", TEST_IMAGE);
        test_resources.add_file("test2.png", TEST_IMAGE);
        test_resources.add_file("test3.png", TEST_IMAGE);

        let mut file_list = FileList::new(Some(gio::File::for_path(test_resources.file_folder().join("test2.png")))).unwrap();

        file_list.next();
        assert_eq!("test3.png", file_list.current_file().unwrap().basename().unwrap().to_str().unwrap());

        file_list.next();
        assert_eq!("test1.png", file_list.current_file().unwrap().basename().unwrap().to_str().unwrap());
    }

    #[test]
    fn test_change_to_previous_image() {
        let mut empty_file_list = FileList::new(None).unwrap();
        assert!(empty_file_list.current_file().is_none());
        empty_file_list.previous();
        assert!(empty_file_list.current_file().is_none());

        let mut test_resources = TestResources::new("test/test_change_to_previous_image");
        test_resources.add_file("test1.png", TEST_IMAGE);
        test_resources.add_file("test2.png", TEST_IMAGE);
        test_resources.add_file("test3.png", TEST_IMAGE);

        let mut file_list = FileList::new(Some(gio::File::for_path(test_resources.file_folder().join("test2.png")))).unwrap();

        file_list.previous();
        assert_eq!("test1.png", file_list.current_file().unwrap().basename().unwrap().to_str().unwrap());

        file_list.previous();
        assert_eq!("test3.png", file_list.current_file().unwrap().basename().unwrap().to_str().unwrap());
    }
}