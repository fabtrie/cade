use std::{env, ffi::OsStr, fs::OpenOptions, io::{self, Write}, path::Path, process};

use config::LogConfig;

use crate::{cache_handler::CacheHandler, cache::cache::Cache};

mod config;
mod cache_handler;
mod nocache_handler;
mod compiler;
mod hash;
mod cache;

fn write(data: &[u8], is_stdout: bool, log_config: &Option<LogConfig>, cache_handler: &dyn CacheHandler) {
    if is_stdout {
        io::stdout().write_all(data).unwrap();
    } else {
        io::stderr().write_all(data).unwrap();
    }
    if let Some(log_config) = log_config {
        let log_config = if is_stdout {
            &log_config.stdout
        } else {
            &log_config.stderr
        };

        if let Some(log_config) = log_config {
            let path = Path::new(&log_config.path);
            
            OpenOptions::new().append(log_config.append).write(!log_config.append).create(true).open(cache_handler.resolve_tmpl(path.to_str().unwrap())).unwrap().write_all(data).unwrap();
        }
    }
}

fn main() {
    let mut args: Vec<String> = env::args().collect();
    // remove name of this binary
    args.remove(0);

    let exe_option = args.get(0);

    let config = config::WrapperConfig::new(&".cade".to_owned());
    let cache = Cache::new(&config);

    match exe_option {
        Some(exe_path) => {
            let mut cache_handler: Box<dyn CacheHandler>;
            let exe =  Path::new(exe_path).file_stem().and_then(OsStr::to_str).expect("could not determine executable");
            match exe {
                "gcc" |
                "g++" |
                "tricore-gcc" |
                "tricore-g++" |
                "cctc"
                => {
                    cache_handler = Box::new(compiler::compile_handler::Compiler::new(exe, cache.as_ref(), &config));
                }
                _ => {
                    match cache.as_ref() {
                        Some(_) => {
                            println!("disable cache for '{}' or implement a cache handler.", exe);
                            std::process::exit(1);
                        }
                        None => {
                            cache_handler = Box::new(nocache_handler::NoCacheHandler);
                        }
                    }
                }
            }

            let restored_from_cache = cache_handler.cache_lookup(&args);

            if let Some(provider_id) = restored_from_cache {
                // dbg!("cache hit");
                for i in 0..=1 {
                    let key;
                    let category;
                    if i == 0 {
                        key = cache_handler.get_stdout_key();
                        category = "stdout";
                    } else {
                        key = cache_handler.get_stderr_key();
                        category = "stderr";
                    }

                    if let Some(key) = key {
                        if let Ok((data, _)) = cache.as_ref().unwrap().get_entry(Some(category), key, Some(&provider_id)) {
                            match config.base_dir.as_ref() {
                                Some(base_dir) => {
                                    let mut output = String::from_utf8(data).unwrap();
                                    output = output.replace("%%%BASE_DIR%%%", base_dir);
                                    write(&output.as_bytes(), i == 0, &config.log, cache_handler.as_ref());
                                }
                                None => {
                                    write(&data, i == 0, &config.log, cache_handler.as_ref());
                                }
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
                        write(&stdout.as_bytes(), true, &config.log, cache_handler.as_ref());
                        write(&stderr.as_bytes(), false, &config.log, cache_handler.as_ref());
                
                        if !output.status.success() {
                            process::exit(output.status.code().unwrap());
                        }

                        if let Some(cache) = cache.as_ref() {

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
