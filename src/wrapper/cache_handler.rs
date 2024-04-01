use std::{io, process::{Command, Output}};

pub trait CacheHandler {
    fn cache_lookup(&mut self, args: &Vec<String>) -> bool;

    fn cache_push(&mut self);

    fn get_stdout_key(&self) -> Option<&String>;
    fn get_stderr_key(&self) -> Option<&String>;

    fn execute(&mut self, args: &Vec<String>) -> io::Result<Output> {
        let status = Command::new(&args[0])
            .args(&args[1..])
            .output();

        self.execute_callback(&status);

        status
    }

    fn execute_callback(&mut self, _result: &io::Result<Output>) {}
}

