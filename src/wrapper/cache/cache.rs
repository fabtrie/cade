use std::{path::Path, io};
use crate::config;
use cade::common::compression::zstd;

use super::{provider::CacheProvider, file_provider::FileCacheProvider, redis_provider::RedisProvider};

/// This is a handler for caching.
/// its purpose is to abstract the caching part from the rest of the logic
pub struct Cache {
    providers: Vec<Box<dyn CacheProvider + 'static>>
}

impl Cache {
    pub fn new(config: &config::WrapperConfig) -> Option<Cache> {
        let mut providers:Vec<Box<dyn CacheProvider + 'static>> = Vec::new();
        let mut id = 0u32;
        for cache_config in &config.cache {
            match cache_config {
                config::CacheConfig::filesystem(filesystem_config) => {
                    let path = Path::new(&filesystem_config.path);
                    let provider = FileCacheProvider::new(id.to_string(), path, filesystem_config.update_on_hit, config.panic_on_cache_content_mismatch, filesystem_config.test_if_update_is_required, config.debug);
                    providers.push(Box::new(provider));
                },
                config::CacheConfig::redis(redis_config) => {
                    let provider = RedisProvider::new(id.to_string(), &redis_config.url, redis_config.update_on_hit, config.panic_on_cache_content_mismatch, redis_config.expire, redis_config.test_if_update_is_required);
                    providers.push(Box::new(provider));
                }
            }
            id += 1;
        }

        if providers.len() == 0 {
            return None;
        }
        Some(Cache {
            providers: providers
        })
    }

    pub fn get_entry(&self, category: Option<&str>, key: &str, provider_id: Option<&str>) -> io::Result<(Vec<u8>,&str)> {
        for provider in self.providers.iter() {
            if let Some(id) = provider_id {
                if id != provider.get_id() {
                    continue;
                }
            }
            let ret = provider.get_entry(category, key);
            if ret.is_ok() {
                let data = ret.unwrap();
                let id = provider.get_id();
                
                for provider2 in self.providers.iter() {
                    if id != provider2.get_id() && provider2.update() && (!provider2.test_if_update_is_required() || !provider2.has_entry(category, key)) {
                        provider2.set_entry(category, key, &data);
                    }
                }

                return Ok((zstd::decompress(&data), provider.get_id()));
            }
        }
        Err(io::Error::new(io::ErrorKind::NotFound, "Not found"))
    }

    fn update_all_entry(&self, category: Option<&str>, key: &str, data: &Vec<u8>) {
        // update in all caches
        for provider in self.providers.iter() {
            if provider.update() {
                provider.set_entry(category, key, &data);
            }
        }
    }

    pub fn set_entry(&self, category: Option<&str>, key: &str, data: &Vec<u8>) {
        let compressed_data = zstd::compress(&data);
        self.update_all_entry(category, key, &compressed_data);
    }
}