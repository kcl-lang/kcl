use crate::langserver;
use crate::util;
use kclvm_error::Position;
use std::collections::HashMap;

// Record all occurrences of the name in a file
#[derive(Default)]
pub struct FileWordMap {
    file_name: String,
    word_map: HashMap<String, Vec<Position>>,
}

impl FileWordMap {
    pub fn new(file_name: String) -> Self {
        Self {
            file_name,
            word_map: HashMap::new(),
        }
    }

    // Clear records
    pub fn clear(&mut self) {
        self.word_map.clear();
    }

    // insert an occurrence of a name
    pub fn insert(&mut self, name: String, pos: Position) {
        self.word_map.entry(name).or_insert(Vec::new()).push(pos);
    }

    // build the record map
    // if text is missing, it will be read from the file system based on the filename
    pub fn build(&mut self, text: Option<String>) {
        self.clear();
        let text = text.unwrap_or(langserver::read_file(&self.file_name).unwrap());
        let lines: Vec<&str> = text.lines().collect();
        for (li, line) in lines.into_iter().enumerate() {
            let words = langserver::line_to_words(line.to_string());
            words.iter().for_each(|x| {
                self.word_map
                    .entry(x.word.clone())
                    .or_insert(Vec::new())
                    .push(Position {
                        filename: self.file_name.clone(),
                        line: li as u64,
                        column: Some(x.startpos as u64),
                    })
            });
        }
    }

    // return all occurrence of a name
    pub fn get(&self, name: &String) -> Option<&Vec<Position>> {
        self.word_map.get(name)
    }
}

// Record all occurrences of the name in workspace
pub struct WorkSpaceWordMap {
    path: String,
    file_map: HashMap<String, FileWordMap>,
}

impl WorkSpaceWordMap {
    pub fn new(path: String) -> Self {
        Self {
            path,
            file_map: HashMap::new(),
        }
    }

    // when user edit a file, the filemap of this file need to rebuild
    pub fn change_file(&mut self, file_name: String, text: String) {
        self.file_map
            .entry(file_name.clone())
            .or_insert(FileWordMap::new(file_name))
            .build(Some(text));
    }

    // when user add a file, the workspacemap will add a new filemap for it
    pub fn create_file(&mut self, file_name: String) {
        self.file_map
            .entry(file_name.clone())
            .or_insert(FileWordMap::new(file_name))
            .clear();
    }

    // when user delete a file, the workspacemap will remove the old filemap of it
    pub fn delete_file(&mut self, file_name: String) {
        self.file_map.remove(&file_name);
    }

    // when user rename a file, the workspacemap will remove the old filemap of it and build a new filemap for it
    pub fn rename_file(&mut self, old_name: String, new_name: String) {
        self.delete_file(old_name);
        self.create_file(new_name.clone());
        self.file_map.get_mut(&new_name).unwrap().build(None);
    }

    // build & maintain the record map for each file under the path
    pub fn build(&mut self) {
        //TODO may use some cache from other component?
        let files = util::get_kcl_files(&self.path, true);
        match files {
            Ok(files) => {
                for file in files.into_iter() {
                    self.file_map
                        .insert(file.clone(), FileWordMap::new(file.clone()));
                    self.file_map.get_mut(&file).unwrap().build(None);
                }
            }
            Err(_) => {}
        }
    }

    // return all occurrence of a name in the workspace
    pub fn get(self, name: &String) -> Option<Vec<Position>> {
        let mut words = Vec::new();
        for (_, mp) in self.file_map.iter() {
            match mp.get(name) {
                Some(file_words) => {
                    // words.extend(file_words.into_iter());
                    words.extend_from_slice(file_words);
                }
                None => {}
            }
        }
        Some(words)
    }
}
