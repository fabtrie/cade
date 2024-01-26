use std::{path::Path, io};
use crate::{compression::zstd, wrapper_config};

use super::{provider::CacheProvider, file_provider::FileCacheProvider};

/// This is a handler for caching.
/// its purpose is to abstract the caching part from the rest of the logic
pub struct Cache {
    providers: Vec<Box<dyn CacheProvider + 'static>>,
}

impl Cache {
    pub fn new(cache_configs: &Vec<wrapper_config::CacheConfig>) -> Cache {
        let mut providers:Vec<Box<dyn CacheProvider + 'static>> = Vec::new();
        for config in cache_configs {
            match config.variant.as_str() {
                "filesystem" => {
                    let path = Path::new(&config.path);
                    let provider = FileCacheProvider::new(path, config.update_on_hit);
                    providers.push(Box::new(provider));},
                _ => panic!("Unknown cache provider: {}", config.variant),
            }
        }
        Cache {
            providers: providers
        }
    }

    pub fn get_entry(&self, category: Option<&str>, key: &str) -> io::Result<Vec<u8>> {
        for provider in self.providers.iter() {
            let ret = provider.get_entry(category, key);
            if ret.is_ok() {
                let data = ret.unwrap();
                
                for provider in self.providers.iter() {
                    if provider.update() {
                        provider.set_entry(category, key, &data);
                    }
                }

                return Ok(zstd::decompress(&data));
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "Not found"))
    }

    fn update_all_entry(&self, category: Option<&str>, key: &str, data: &Vec<u8>) {
        // update in all caches
        for provider in self.providers.iter() {
            provider.set_entry(category, key, &data);
        }
    }

    pub fn set_entry(&self, category: Option<&str>, key: &str, data: &Vec<u8>) {
        let compressed_data = zstd::compress(&data);
        self.update_all_entry(category, key, &compressed_data);
    }


}