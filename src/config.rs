use anyhow::Result;
use anyhow::{Context, Error};
use iced::futures::TryStreamExt;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

const APP_NAME: &'static str = "houdini.license.chaser";

pub fn load_config_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let reader = std::fs::File::open(&path)
        .context(format!("Could not open file: {}", path.to_string_lossy()))?;
    serde_json::from_reader(reader).context(format!("Error loading {}", path.to_string_lossy()))
}

pub fn save_config_file<T: Serialize>(value: T, path: impl AsRef<Path>) -> Result<()> {
    let path = path.as_ref();
    let parent = path.parent().unwrap();
    if !parent.exists() {
        std::fs::create_dir_all(parent).context("Could not create app cache dir")?;
    }
    let mut file = std::fs::File::create(path).context("Could not create app cache file")?;
    serde_json::to_writer_pretty(file, &value).context("Could not serialize cache")?;
    Ok(())
}
#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UserConfig {
    pub hfs: PathBuf,
}

impl UserConfig {
    const CONFIG_FILE: &'static str = "chaser.json";

    fn config_file() -> Result<PathBuf> {
        dirs::config_dir()
            .ok_or(Error::msg("Platform config directory not found"))
            .map(|path| path.join(APP_NAME).join(UserConfig::CONFIG_FILE))
    }

    pub fn load() -> Result<UserConfig> {
        load_config_file(&UserConfig::config_file()?)
    }

    pub fn save(&self) -> Result<()> {
        save_config_file(self, UserConfig::config_file()?)
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
            .ok_or(Error::msg("Platform cache directory not found"))
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
