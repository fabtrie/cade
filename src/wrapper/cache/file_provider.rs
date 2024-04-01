use std::path::{Path, PathBuf};
use std::{fs, io};

use super::provider::CacheProvider;


pub struct FileCacheProvider {
    path: PathBuf,
    update: bool,
    panic_on_cache_content_mismatch: bool
}

impl FileCacheProvider {
    pub fn new(path: &Path, update: bool, panic_on_cache_content_mismatch: bool) -> FileCacheProvider {
        if !path.exists() {
            fs::create_dir_all(path).expect(&format!("Unable to create directory '{}'!", path.to_str().unwrap()));
        }
        FileCacheProvider {
            path: path.to_path_buf(),
            update: update,
            panic_on_cache_content_mismatch: panic_on_cache_content_mismatch
        }
    }

    fn get_path(&self, category: Option<&str>, key: &str) -> PathBuf {
        match category {
            Some(category) => self.path.join(category).join(key),
            None => self.path.join(key),
        }
    }
}

impl CacheProvider for FileCacheProvider {

    fn get_entry(&self, category: Option<&str>, key: &str) -> io::Result<Vec<u8>> {
        fs::read(self.get_path(category, key))
    }

    fn set_entry(&self, category: Option<&str>, key: &str, value: &Vec<u8>) {
        let path = self.get_path(category, key);
        if self.panic_on_cache_content_mismatch && path.exists() && category != Some("obj") {
            let input_data = std::fs::read(&path).expect(&format!("Unable to read input file '{}'!", path.to_str().unwrap()));
            if input_data != *value {
                panic!("content of '{}' does not match expected value! (hash collision?)", path.to_str().unwrap());
            }
        } else {
            if path.parent().is_some() {
                fs::create_dir_all(path.parent().unwrap()).expect(&format!("Unable to create directory '{}'!", path.parent().unwrap().to_str().unwrap()));
            }
            fs::write(path, value).unwrap();
        }
    }

    fn has_entry(&self, category: Option<&str>, key: &str) -> bool {
        let path = self.get_path(category, key);
        path.exists()
    }

    fn update(&self) -> bool {
        self.update
    }
}