use super::compile_handler::CompilerTrait;

pub struct Tasking;

impl CompilerTrait for Tasking {
    fn get_name(&self) -> String {
        String::from("tasking")
    }

    fn get_resp_file_prefix(&self) -> Vec<String> {
        vec!["--option-file=".to_owned(), "-f".to_owned()]
    }

    fn get_dep_file_prefix(&self) -> Vec<String> {
        vec!["--dep-file=".to_owned()]
    }
}