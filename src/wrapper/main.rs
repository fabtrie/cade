use std::{env, io::{self, Write}, process, path::Path, ffi::OsStr};

use crate::{cache_handler::CacheHandler, cache::cache::Cache};

mod config;
mod cache_handler;
mod compiler;
mod hash;
mod cache;

fn main() {
    let mut args: Vec<String> = env::args().collect();
    // remove name of this binary
    args.remove(0);

    let exe_option = args.get(0);

    let config = config::WrapperConfig::new(&".cade".to_owned());
    let cache: Cache = Cache::new(&config);

    match exe_option {
        Some(exe_path) => {
            let exe =  Path::new(exe_path).file_stem().and_then(OsStr::to_str).expect("could not determine executable");
            let mut cache_handler: Box<dyn CacheHandler>;
            match exe {
                "gcc" |
                "g++" |
                "tricore-gcc" |
                "tricore-g++" |
                "cctc"
                => {
                    cache_handler = Box::new(compiler::compile_handler::Compiler::new(exe, &cache, &config));
                }
                _ => {
                    println!("Unknown exe");
                    std::process::exit(1);
                }
            }

            let restored_from_cache = cache_handler.cache_lookup(&args);

            if let Some(provider_id) = restored_from_cache {
                // dbg!("cache hit");
                if let Some(key) = cache_handler.get_stdout_key() {
                    if let Ok((data, _)) = cache.get_entry(Some("stdout"), key, Some(&provider_id)) {
                        match config.base_dir.as_ref() {
                            Some(base_dir) => {
                                let mut stdout = String::from_utf8(data).unwrap();
                                stdout = stdout.replace("%%%BASE_DIR%%%", base_dir);
                                io::stdout().write_all(&stdout.as_bytes()).unwrap();
                            }
                            None => {
                                io::stdout().write_all(&data).unwrap();
                            }
                        }
                    }
                }
                if let Some(key) = cache_handler.get_stderr_key() {
                    if let Ok((data, _)) = cache.get_entry(Some("stderr"), key, Some(&provider_id)) {
                        match config.base_dir.as_ref() {
                            Some(base_dir) => {
                                let mut stderr = String::from_utf8(data).unwrap();
                                stderr = stderr.replace("%%%BASE_DIR%%%", base_dir);
                                io::stderr().write_all(&stderr.as_bytes()).unwrap();
                            }
                            None => {
                                io::stderr().write_all(&data).unwrap();
                            }
                        }
                    }
                }
            } else{
                // dbg!("cache miss");
                let status = cache_handler.execute(&args);
                match status {
                    Ok(output) => {
                        let mut stdout = String::from_utf8(output.stdout).unwrap();
                        let mut stderr = String::from_utf8(output.stderr).unwrap();

                        io::stdout().write_all(&stdout.as_bytes()).unwrap();
                        io::stderr().write_all(&stderr.as_bytes()).unwrap();
                
                        if !output.status.success() {
                            process::exit(output.status.code().unwrap());
                        }

                        cache_handler.cache_push();

                        if config.base_dir.is_some() {
                            let base_dir = config.base_dir.as_ref().unwrap();
                            stdout = stdout.replace(base_dir, "%%%BASE_DIR%%%");
                            stderr = stderr.replace(base_dir, "%%%BASE_DIR%%%");
                        }

                        // do not cache this call. It may have been recalculated.
                        match cache_handler.get_stdout_key() {
                            Some(key) => {
                                if stdout.len() > 0 {
                                    cache.set_entry(Some("stdout"), key, &stdout.as_bytes().to_vec());
                                }
                            }
                            None => (),
                        }
                        match cache_handler.get_stderr_key() {
                            Some(key) => {
                                if stderr.len() > 0 {
                                    cache.set_entry(Some("stderr"), key, &stderr.as_bytes().to_vec());
                                }
                            }
                            None => (),
                        }
                    },
                    Err(_) => {
                        panic!("Could not execute handler");
                    }
                }
            }
        }
        None => {
            println!("No exe");
        }
    }
    
}
