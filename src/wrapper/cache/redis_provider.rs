use std::io;

use redis::{Commands, Expiry, RedisError};

use super::provider::CacheProvider;

pub struct RedisProvider {
    update: bool,
    panic_on_cache_content_mismatch: bool,
    expire: Option<u32>,
    client: redis::Client
}

impl RedisProvider {
    pub fn new(url: &str, update: bool, panic_on_cache_content_mismatch: bool, expire: Option<u32>) -> RedisProvider {
        let client = redis::Client::open(url).unwrap();

        RedisProvider {
            update: update,
            panic_on_cache_content_mismatch: panic_on_cache_content_mismatch,
            expire: expire,
            client: client
        }
    }

    fn get_key(&self, category: Option<&str>, key: &str) -> String {
        match category {
            Some(category) => format!("{}_{}", category, key),
            None => key.to_string()
        }
    }
}

impl CacheProvider for RedisProvider {
    fn get_entry(&self, category: Option<&str>, key: &str) -> io::Result<Vec<u8>> {
        let full_key = self.get_key(category, key);

        let mut con = self.client.get_connection().unwrap();

        let ret: Result<Vec<u8>, RedisError>;
        if self.expire.is_some() {
            ret = con.get_ex(&full_key, Expiry::EX(self.expire.unwrap().try_into().unwrap()));
        } else {
            ret = con.get(&full_key);
        }

        match ret {
            Ok(data) => {
                // retrun err if data is empty
                if data.len() == 0 {
                    Err(io::Error::new(io::ErrorKind::NotFound, "Not found"))
                } else {
                    Ok(data)
                }
            },
            Err(err) => Err(io::Error::new(io::ErrorKind::Other, err))
        }
    }

    fn set_entry(&self, category: Option<&str>, key: &str, value: &Vec<u8>) {
        let full_key = self.get_key(category, key);
        let mut con = self.client.get_connection().unwrap();
        if self.has_entry(category, key) {
            let _:() = con.expire(&full_key, self.expire.unwrap().into()).unwrap();
            
            if self.panic_on_cache_content_mismatch && category != Some("obj") {
                let input_data = self.get_entry(category, key).expect(&format!("Unable to access redis key '{}'!", full_key));
                if input_data != *value {
                    panic!("content of '{}' does not match expected value! (hash collision?)", full_key);
                }
            }
        } else {
            if self.expire.is_some() {
                let _:() = con.set_ex(&full_key, value, self.expire.unwrap().into()).unwrap();
            } else {
                let _:() = con.set(&full_key, value).unwrap();
            }
        }
    }

    fn has_entry(&self, category: Option<&str>, key: &str) -> bool {
        let mut con = self.client.get_connection().unwrap();

        con.exists(self.get_key(category, key)).unwrap()
    }

    fn update(&self) -> bool {
        self.update
    }
}