use gio::{Cancellable, FileExt, FileQueryInfoFlags, FileType};

pub struct FileList {
    file_list: Vec<gio::FileInfo>,
    current_file: Option<(usize, gio::File)>,
    current_folder: Option<gio::File>,
}

impl FileList {
    pub fn new(current_file: Option<gio::File>) -> FileList {
        if let Some(current_file) = current_file {
            let current_folder = current_file.get_parent().unwrap();
            let file_list: Vec<gio::FileInfo> = FileList::enumerate_files(&current_folder);
            let current_file_index = file_list
                .iter()
                .position(|file| file.get_name() == current_file.get_basename())
                .unwrap();

            FileList {
                file_list,
                current_file: Some((current_file_index, current_file)),
                current_folder: Some(current_folder),
            }
        } else {
            FileList {
                file_list: Vec::new(),
                current_file: None,
                current_folder: None,
            }
        }
    }

    pub fn refresh(&mut self) {
        if let Some(current_folder) = &self.current_folder {
            if !current_folder.query_exists::<Cancellable>(None) {
                self.file_list = Vec::new();
                self.current_file = None;
                self.current_folder = None;
                return;
            }
            self.file_list = FileList::enumerate_files(&current_folder);

            match &self.current_file {
                Some((_, current_file)) => {
                    let file_index = self
                        .file_list
                        .iter()
                        .position(|file| file.get_name() == current_file.get_basename());
                    if let Some(file_index) = file_index {
                        self.current_file = Some((file_index, self.current_file.take().unwrap().1));
                    } else {
                        self.next();
                    }
                }
                None => self.next(),
            }
        }
    }

    pub fn next(&mut self) {
        if let Some(current_folder) = &self.current_folder {
            self.current_file = match self.current_file.take() {
                Some((_, _)) if self.file_list.is_empty() => None,
                Some((index, _)) if index + 1 < self.file_list.len() => Some((
                    index + 1,
                    current_folder
                        .get_child(self.file_list[index + 1].get_name().unwrap())
                        .unwrap(),
                )),
                Some((index, _)) if index + 1 >= self.file_list.len() => Some((
                    0,
                    current_folder
                        .get_child(self.file_list[0].get_name().unwrap())
                        .unwrap(),
                )),
                None if !self.file_list.is_empty() => Some((
                    0,
                    current_folder
                        .get_child(self.file_list[0].get_name().unwrap())
                        .unwrap(),
                )),
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
                    current_folder
                        .get_child(self.file_list[index - 1].get_name().unwrap())
                        .unwrap(),
                )),
                Some((index, _)) if index as i64 - 1 < 0 => Some((
                    self.file_list.len() - 1,
                    current_folder
                        .get_child(self.file_list[self.file_list.len() - 1].get_name().unwrap())
                        .unwrap(),
                )),
                None if !self.file_list.is_empty() => Some((
                    0,
                    current_folder
                        .get_child(self.file_list[0].get_name().unwrap())
                        .unwrap(),
                )),
                _ => None,
            }
        }
    }

    pub fn current_folder(&self) -> Option<&gio::File> {
        self.current_folder.as_ref()
    }

    pub fn current_file(&self) -> Option<&gio::File> {
        self.current_file.as_ref().map(|(_, file)| file)
    }

    pub fn len(&self) -> usize {
        self.file_list.len()
    }

    fn enumerate_files(folder: &gio::File) -> Vec<gio::FileInfo> {
        folder
            .enumerate_children::<Cancellable>("", FileQueryInfoFlags::NONE, None)
            .unwrap()
            .into_iter()
            .filter_map(|file| file.ok())
            .filter(|file| file.get_file_type() == FileType::Regular)
            .filter(|file| {
                gio::content_type_guess(file.get_name().unwrap().to_str(), &[])
                    .0
                    .starts_with("image")
            })
            .collect()
    }
}
