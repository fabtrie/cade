use std::{fs, io, path::Path, process::Output, str};

use crate::{cache_handler::CacheHandler, hash::Hasher, cache::cache::Cache, config};

use super::{response_file, dep_parser::{self, DepParser}, gcc, tasking};


pub trait CompilerTrait {
    fn get_name(&self) -> String;

    fn get_resp_file_prefix(&self) -> Vec<String>;

    fn get_dep_file_prefix(&self) -> Vec<String>;
}


pub struct Compiler<'a> {
    specific: Box<dyn CompilerTrait>,
    parsed_args: CompilerArgs,
    cache: Option<&'a Cache>,
    total_hash: Option<String>,
    dep_hash:Option<String>,
    config: &'a config::WrapperConfig
}

struct CompilerArgs {
    processed_args: Vec<String>,
    dep_file: Option<String>,
    out_file: Option<String>,
    source_file: Option<String>,
}

impl<'a> CacheHandler for Compiler<'a> {

    fn cache_lookup(&mut self, args: &Vec<String>) -> Option<String> {
        self.parsed_args = self.parse_args(args);

        if let Some(cache) = self.cache {
            if self.parsed_args.dep_file.is_some() && self.parsed_args.out_file.is_some() && self.parsed_args.source_file.is_some() {
                let source_file = self.parsed_args.source_file.as_ref().unwrap();

                let source_data = match fs::read(source_file) {
                    Ok(data) => data,
                    Err(_) => { println!("Could not read source file {}.", source_file); std::process::exit(1); }
                };

                let mut dep_hasher = Hasher::new();
                dep_hasher.update(&source_data);
                self.update_hash(&mut dep_hasher);
                self.dep_hash = Some(dep_hasher.finalize());

                let dep_file = self.parsed_args.dep_file.as_ref().unwrap();

                let dep_file_result = self.get_dep_file(cache);

                match dep_file_result {
                    Ok(dep_file_data) => {
                        // an entry for the source file exists in the cache and we could restore its dependency file
                        let mut dep_str = String::from_utf8(dep_file_data).unwrap();

                        if self.config.base_dir.is_some() {
                            // in case base dir is set, replace placeholder with actual base dir
                            let base_dir = self.config.base_dir.as_ref().unwrap();
                            dep_str = dep_str.replace("%%%BASE_DIR%%%", base_dir);
                        }
                        // write dep file to disk
                        fs::write(dep_file, &dep_str).unwrap();

                        // parse dep file to create hash of all dependencies
                        let dep = dep_parser::DepParser::new(&dep_str);

                        if let Some(hash) = self._get_object_hash(&dep).ok() {
                            self.total_hash = Some(hash);
                        } else {
                            // error creating the hash (e.g. error reading dep file)
                            return None;
                        }

                        let obj_result = cache.get_entry(Some("obj"), self.total_hash.as_ref().unwrap(), None);
                        match obj_result {
                            Ok((obj_data, provider_id)) => {
                                // cache hit. Source file and all dependencies match.
                                // write object file to disk
                                fs::write(self.parsed_args.out_file.as_ref().unwrap(), obj_data).unwrap();
                                return Some(provider_id.to_string());
                            },
                            // cache miss. Source file matches but dependencies not.
                            Err(_) => ()
                        }
                    },
                    // cache miss. C-File was updated.
                    Err(_) => ()
                };
            }
        }

        None
    }

    fn execute_callback(&mut self, _result: &io::Result<Output>) {
        // nothing to do
    }

    fn cache_push(&mut self) {
        if let Some(cache) = self.cache {
            if self.total_hash.is_none() {
                let dep_file = self.parsed_args.dep_file.as_ref().unwrap();
                match fs::read_to_string(dep_file) {
                    Ok(dep_str) => {
                        let dep = dep_parser::DepParser::new(&dep_str);
                        match self._get_object_hash(&dep) {
                            Ok(hash) => self.total_hash = Some(hash),
                            Err(err) => { 
                                println!("Error creating hash for object file: {}", err);
                                std::process::exit(1);
                            }
                        }

                        let mut dep_file_str = dep.get_dep_file_string();

                        // replace base_dir with placeholder before caching
                        // this is required to be able to set the propper base dir on cache load
                        if self.config.base_dir.is_some() {
                            let base_dir = self.config.base_dir.as_ref().unwrap();
                            dep_file_str = dep_file_str.replace(base_dir, "%%%BASE_DIR%%%");
                        }
                        
                        self.set_dep_file(&dep_file_str.as_bytes().to_vec(), cache);
                    },
                    Err(_) => {
                        println!("Could not read dep file {}.", dep_file);
                        std::process::exit(1);
                    }
                }
            }
            
            let obj_data = fs::read(self.parsed_args.out_file.as_ref().unwrap()).expect("Unable to read output file!");
            let hash = self.total_hash.as_ref().unwrap();
            cache.set_entry(Some("obj"), hash, &obj_data);
        }
    }

    fn get_stdout_key(&self) -> Option<&String> {
        self.total_hash.as_ref()
    }

    fn get_stderr_key(&self) -> Option<&String> {
        self.total_hash.as_ref()
    }

    fn resolve_tmpl(&self, tmpl: &str) -> String {
        let path_str = self.parsed_args.out_file.as_ref().unwrap();
        let path = Path::new(path_str);
        tmpl
        .replace("{obj_folder}", path.parent().unwrap().to_str().unwrap())
        .replace("{obj_path}", path_str)
    }

}

fn handle_path_arg(arg: &str, prefix:&str, next_arg: &Option<&String>) -> (bool, String) {
    let mut path =arg.strip_prefix(prefix).unwrap();
    let mut skip_next = false;
    if path.len() == 0 {
        path = &next_arg.unwrap();
        skip_next = true;
    }

    return (skip_next, path.to_owned());
}

impl<'a> Compiler<'a> {
    pub fn new(exe_name:&str, cache: Option<&'a Cache>, config: &'a config::WrapperConfig) -> Compiler<'a> {
        // Compiler{name: compiler.get_name()}
        let compiler: Box<dyn CompilerTrait>;
        match exe_name {
            "gcc" |
            "g++" |
            "tricore-gcc" |
            "tricore-g++" => { compiler = Box::new(gcc::Gcc{}); },
            "cctc" => { compiler = Box::new(tasking::Tasking{}); },
            _ => {
                println!("Unknown compiler");
                std::process::exit(1);
            }
        }
        Compiler{
            specific: compiler,
            parsed_args: CompilerArgs{processed_args: Vec::new(), dep_file: None, out_file: None, source_file: None},
            cache: cache,
            total_hash: None,
            dep_hash: None,
            config: config
        }
    }

    #[allow(dead_code)]
    pub fn get_name(&self) -> String {
        self.specific.get_name()
    }

    fn parse_args(&self, args: &Vec<String>) -> CompilerArgs {
        let mut dep_file = None;
        let mut out_file = None;
        let mut source_file = None;
        
        let mut full_args = Vec::new();
        let mut skip_next = false;
        'arg_loop: for (i, arg) in args.iter().enumerate() {
            if skip_next {
                skip_next = false;
                continue;
            }
            for resp_file_prefix in self.specific.get_resp_file_prefix() {
                if arg.starts_with(&resp_file_prefix) {
                    let file_path: &str;
                    if arg == &resp_file_prefix {
                        // args.iter().nth(i+1).unwrap()
                        file_path = &args[i+1];
                        skip_next = true;
                    } else {
                        file_path = arg.strip_prefix(&resp_file_prefix).unwrap();
                    }
                    let resp_parser = response_file::Parser::new(&file_path);
                    let resp_file_args = self.parse_args(&resp_parser.args);
                    full_args.extend(resp_file_args.processed_args);
    
                    if dep_file == None { dep_file = resp_file_args.dep_file; }
                    if out_file == None { out_file = resp_file_args.out_file; }
                    if source_file == None { source_file = resp_file_args.source_file; }

                    continue 'arg_loop;
                }
            }
            for dep_file_prefix in self.specific.get_dep_file_prefix() {
                if arg.starts_with(&dep_file_prefix) {
                    let processed_arg;
                    (skip_next, processed_arg) = handle_path_arg(arg, &dep_file_prefix, &args.get(i+1));
                    full_args.push(dep_file_prefix.to_owned() + &processed_arg);
                    dep_file = Some(processed_arg);
                    
                    continue 'arg_loop;
                }
            } if arg.starts_with("-I") || arg.starts_with("-c") || arg.starts_with("-o") {
                let prefix = arg.get(0..2).unwrap();
                let processed_arg;
                (skip_next, processed_arg) = handle_path_arg(arg, prefix, &args.get(i+1));
                full_args.push(prefix.to_owned() + &processed_arg);

                if prefix == "-c" {
                    source_file = Some(processed_arg);
                } else if prefix == "-o" {
                    out_file = Some(processed_arg);
                }
            } else {
                full_args.push(arg.to_owned());
            }
        }

        if let Some(base_dir) = self.config.base_dir.as_ref() {
            for arg in &mut full_args {
                *arg = arg.replace(base_dir, "");
            }
        }
        
        CompilerArgs {
            processed_args: full_args,
            dep_file: dep_file,
            out_file: out_file,
            source_file: source_file,
        }
    }
    
    fn get_dep_file(&self, cache: &Cache) -> io::Result<Vec<u8>> {
        cache.get_entry(Some("dep"), &self.dep_hash.as_ref().unwrap(), None).and_then(|(dep_data, _)| {
            Ok(dep_data)
        })
    }

    pub fn set_dep_file(&self, data: &Vec<u8>, cache: &Cache) {
        cache.set_entry(Some("dep"), &self.dep_hash.as_ref().unwrap(), &data);

        if self.config.debug {
            fs::write(self.parsed_args.out_file.as_ref().unwrap().to_owned() + ".cade_dep", data).unwrap();
        }
    }

    pub fn update_hash(&self, hasher: &mut Hasher) {
        let mut all_args = self.parsed_args.processed_args.join("").to_string();
        if let Some(base_dir) = self.config.base_dir.as_ref() {
            all_args = all_args.replace(base_dir, "");
        }
        if self.config.debug {
            println!("all_args: {}", all_args);
        }

        hasher.update(all_args.as_bytes());
    }

    fn _get_object_hash(&self, dep: &DepParser) -> Result<String, Box<dyn std::error::Error + 'static>> {
        // Hash an input incrementally.
        let mut hasher = Hasher::new();
        self.update_hash(&mut hasher);
        dep.update_hash(&mut hasher)?;
        let hash = hasher.finalize();
        Ok(hash)
    }
}

