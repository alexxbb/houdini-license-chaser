use crate::app::{LicenseType, APP_NAME};
use iced::futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Write};
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, ConfigError>;

pub fn load_config_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    use std::io::ErrorKind;

    match std::fs::File::open(&path) {
        Ok(file) => serde_json::from_reader(file).map_err(ConfigError::ParseError),
        Err(e) => match e.kind() {
            ErrorKind::NotFound => Err(ConfigError::Missing),
            _ => Err(e.into()),
        },
    }
}

pub fn save_config_file<T: Serialize>(value: T, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let parent = path.parent().unwrap();
    if !parent.exists() {
        std::fs::create_dir_all(parent)?;
    }
    let mut file = std::fs::File::create(path)?;
    serde_json::to_writer_pretty(file, &value)?;
    Ok(())
}
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UserConfig {
    pub hfs: PathBuf,
    pub server_url: String,
}

impl UserConfig {
    const CONFIG_FILE: &'static str = "chaser.json";

    pub fn houdini_executable(&self, lic_type: LicenseType) -> PathBuf {
        let file = match lic_type {
            LicenseType::Core => "houdinicore",
            LicenseType::Fx => "houdinifx",
            LicenseType::Other => "houdini",
        };
        self.hfs.join("bin").join(file)
    }

    pub fn config_file() -> Result<PathBuf> {
        dirs::config_dir()
            .ok_or(ConfigError::Other(
                "Platform config directory not found".to_owned(),
            ))
            .map(|path| path.join(APP_NAME).join(UserConfig::CONFIG_FILE))
    }

    pub fn load() -> Result<UserConfig> {
        load_config_file(&UserConfig::config_file()?)
    }

    pub fn save(&self) -> Result<()> {
        save_config_file(self, UserConfig::config_file()?)
    }

    pub fn is_valid(&self) -> bool {
        !self.server_url.is_empty() && self.hfs.join("bin").exists()
    }
}

#[derive(Debug)]
pub enum ConfigError {
    Missing,
    Io(std::io::Error),
    ParseError(serde_json::Error),
    Other(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Other(s) => f.write_str(s),
            ConfigError::Missing => f.write_str("Config file is missing"),
            ConfigError::ParseError(e) => f.write_fmt(format_args!("Could not load config: {}", e)),
            ConfigError::Io(e) => f.write_fmt(format_args!("IO Error: {}", e)),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<serde_json::Error> for ConfigError {
    fn from(value: serde_json::Error) -> Self {
        ConfigError::ParseError(value)
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(value: std::io::Error) -> Self {
        ConfigError::Io(value)
    }
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AppCache {
    pub window_position: [i32; 2],
}

impl AppCache {
    const CACHE_FILE: &'static str = "cache.json";

    fn cache_file() -> Result<PathBuf> {
        dirs::cache_dir()
            .ok_or(ConfigError::Other(
                "Platform cache directory not found".to_owned(),
            ))
            .map(|path| path.join(APP_NAME).join(AppCache::CACHE_FILE))
    }
    pub fn load() -> Result<AppCache> {
        load_config_file(&AppCache::cache_file()?)
    }

    pub fn save(&self) -> Result<()> {
        save_config_file(self, Self::cache_file()?)
    }

    // pub fn store_windows_pos(&mut self, pos: [i32; 2]) {
    //     self.window_position = pos;
    // }
}
