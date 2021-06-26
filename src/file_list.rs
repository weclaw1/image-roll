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
            let file_list: Vec<gio::FileInfo> = FileList::enumerate_files(&current_folder)?;
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
            self.file_list = FileList::enumerate_files(&current_folder)?;

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

    // pub fn current_file(&self) -> Option<&gio::File> {
    //     self.current_file.as_ref().map(|(_, file)| file)
    // }

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
            .enumerate_children::<Cancellable>("", FileQueryInfoFlags::NONE, None)?
            .into_iter()
            .filter_map(|file| file.ok())
            .filter(|file| file.file_type() == FileType::Regular)
            .filter(|file| {
                gio::content_type_guess(file.name().to_str(), &[])
                    .0
                    .starts_with("image")
            })
            .collect())
    }

    pub fn current_folder_monitor_mut(&mut self) -> Option<&mut gio::FileMonitor> {
        self.current_folder_monitor.as_mut()
    }
}
