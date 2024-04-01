use std::io;

pub trait CacheProvider {
    fn get_entry(&self, category: Option<&str>, key: &str) -> io::Result<Vec<u8>>;

    fn set_entry(&self, category: Option<&str>, key: &str, value: &Vec<u8>);

    fn has_entry(&self, category: Option<&str>, key: &str) -> bool;

    fn update(&self) -> bool;
}