use std::collections::{BTreeSet, HashMap};
use std::path::PathBuf;
use colored::Colorize;
use crate::io::File;

pub fn file_parity<'a>(
    files: Vec<(PathBuf, Vec<File>)>
) -> (Vec<Vec<File>>, Vec<String>) {
    let mut errors = Vec::new();
    let mut reference_content: BTreeSet<PathBuf> = BTreeSet::new();
    for (folder, files) in &files {
        for file in files {
            reference_content.insert(
                pathdiff::diff_paths(file.path(), &folder).unwrap()
            );
        }
    }

    let mut files_of_type: HashMap<PathBuf, Vec<File>> = HashMap::new();
    for (folder, files) in files {
        let mut expected = reference_content.clone();
        for file in files {
            let file_name = pathdiff::diff_paths(file.path(), &folder).unwrap();
            files_of_type.entry(file_name.clone()).or_insert(Vec::new()).push(file);
            expected.remove(&file_name);
        }
        for not_found in expected {
            errors.push(format!(
                "[{}] File \"{}\" not found in folder \"{}\"",
                "NOT FOUND".yellow(),
                not_found.to_str().unwrap().green(),
                folder.to_str().unwrap().green(),
            ));
        }
    }

    (files_of_type.into_iter().map(|(_, files)| files).collect(), errors)
}
