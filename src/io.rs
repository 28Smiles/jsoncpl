use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

fn file_tree(
    base_path: &PathBuf,
    path: &PathBuf,
    mut files: Vec<File>
) -> Vec<File> {
    let directory = path.read_dir().expect(
        &*format!(
                "\"{}\" is not a directory",
                path.as_os_str().to_str().unwrap()
        )
    );
    for entry in directory {
        let entry = entry.unwrap();
        let path = entry.path();

        if path.is_dir() {
            files = file_tree(base_path, &path, files);
        } else {
            files.push(File(path));
        }
    }

    files
}

pub fn read_folders(
    folders: &Vec<PathBuf>
) -> Vec<(PathBuf, Vec<File>)> {
    let mut lang_content: Vec<(PathBuf, Vec<File>)> = Vec::new();
    for path in folders {
        let content = file_tree(
            path,
            path,
            Vec::new()
        );
        lang_content.push((path.clone(), content));
    }

    lang_content
}

#[derive(Clone)]
pub struct File(PathBuf);
pub struct LoadedFile(PathBuf, String);

impl File {
    pub fn load(self) -> LoadedFile {
        LoadedFile(self.0.clone(), fs::read_to_string(&self.0).unwrap())
    }

    pub fn path(&self) -> &PathBuf {
        &self.0
    }
}

impl LoadedFile {
    pub fn content(&self) -> &String {
        &self.1
    }

    pub fn path(&self) -> &PathBuf {
        &self.0
    }
}

impl PartialEq<Self> for File {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(other.path())
    }
}

impl Eq for File {
}

impl Hash for File {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl PartialEq<Self> for LoadedFile {
    fn eq(&self, other: &Self) -> bool {
        self.0.eq(other.path())
    }
}

impl Eq for LoadedFile {
}

impl Hash for LoadedFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}
