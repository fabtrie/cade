use crate::cache_handler::CacheHandler;

pub struct NoCacheHandler;

impl CacheHandler for NoCacheHandler {
    fn cache_lookup(&mut self, _args: &Vec<String>) -> Option<String> {
        None
    }

    fn cache_push(&mut self) {
    }

    fn get_stdout_key(&self) -> Option<&String> {
        None
    }

    fn get_stderr_key(&self) -> Option<&String> {
        None
    }
}