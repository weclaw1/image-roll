use std::path::{Path, PathBuf};

pub struct TestResources {
    file_folder: PathBuf,
}

impl TestResources {
    pub fn new<P: AsRef<Path>>(file_folder: P) -> Self {
        std::fs::create_dir_all(&file_folder).unwrap();
        Self {
            file_folder: file_folder.as_ref().to_path_buf(),
        }
    }

    pub fn add_file<T: AsRef<str>, C: AsRef<[u8]>>(&mut self, file_name: T, contents: C) {
        std::fs::write(self.file_folder.join(file_name.as_ref()), contents).unwrap();
    }

    pub fn remove_file<T: AsRef<str>>(&mut self, file_name: T) {
        std::fs::remove_file(self.file_folder.join(file_name.as_ref())).unwrap();
    }

    pub fn file_folder(&self) -> &Path {
        self.file_folder.as_path()
    }
}

impl Drop for TestResources {
    fn drop(&mut self) {
        std::fs::remove_dir_all(self.file_folder.as_path()).unwrap();
    }
}
