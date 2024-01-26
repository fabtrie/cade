use config::FileFormat;
use serde::Deserialize;

#[derive(Deserialize)]
pub enum CacheAccess {
    Read,
    Write,
    ReadWrite,
}

#[derive(Deserialize)]
pub struct CacheConfig {
    pub variant: String,
    pub path: String,
    pub access: CacheAccess,
    #[serde(default = "update_on_hit_default")]
    pub update_on_hit: bool
}

fn update_on_hit_default() -> bool {
    true
}

#[derive(Deserialize)]
pub struct WrapperConfig {
    pub base_dir: Option<String>,
    pub cache: Vec<CacheConfig>,
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