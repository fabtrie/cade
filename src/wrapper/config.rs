use config::FileFormat;
use serde::Deserialize;

#[derive(Deserialize)]
pub enum CacheAccess {
    Read,
    Write,
    ReadWrite,
}

#[derive(Deserialize)]
pub struct FilesystemConfig {
    pub path: String,
    pub access: CacheAccess,
    #[serde(default = "bool_true_default")]
    pub update_on_hit: bool,
    #[serde(default = "bool_true_default")]
    pub test_if_update_is_required: bool
}

#[derive(Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub expire: Option<u32>,
    pub access: CacheAccess,
    #[serde(default = "bool_true_default")]
    pub update_on_hit: bool,
    #[serde(default = "bool_true_default")]
    pub test_if_update_is_required: bool
}

#[derive(Deserialize)]
#[allow(non_camel_case_types)]
pub enum CacheConfig {
    filesystem(FilesystemConfig),
    redis(RedisConfig)
}

fn bool_true_default() -> bool {
    true
}

#[derive(Deserialize)]
pub struct WrapperConfig {
    pub base_dir: Option<String>,
    pub cache: Vec<CacheConfig>,
    #[serde(default = "debug_default")]
    pub debug: bool,
    #[serde(default = "panic_on_cache_content_mismatch_default")]
    pub panic_on_cache_content_mismatch: bool,
}

fn debug_default() -> bool {
    false
}

fn panic_on_cache_content_mismatch_default() -> bool {
    false
}

impl WrapperConfig {
    pub fn new(file_path: &String) -> Self {
        let config = config::Config::builder()
            .add_source(
                config::Environment::with_prefix("CADE")
                    .try_parsing(true)
                    .separator("_")
                    .list_separator(" "),
            )
            .add_source(config::File::new(&file_path, FileFormat::Json))
            .set_default("debug", false).unwrap()
            .build()
            .unwrap();

        let app: WrapperConfig = config.try_deserialize().unwrap();

        app
    }
}