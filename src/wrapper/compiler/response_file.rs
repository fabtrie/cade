use std::{io::{BufReader, BufRead}, fs::File};

pub struct Parser {
    pub args: Vec<String>,
}

impl Parser {
    pub fn new(response_file: &str) -> Parser {
        let args = BufReader::new(File::open(response_file).unwrap()).lines().map(|l| l.unwrap()).collect();
        Parser {
            args: args,
        }
    }
}