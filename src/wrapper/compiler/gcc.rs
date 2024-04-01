use super::compile_handler::CompilerTrait;

pub struct Gcc;

impl CompilerTrait for Gcc {
    fn get_name(&self) -> String {
        String::from("gcc")
    }

    fn get_resp_file_prefix(&self) -> Vec<String> {
        vec!["@".to_owned()]
    }

    fn get_dep_file_prefix(&self) -> Vec<String> {
        vec!["-MF".to_owned()]
    }
}