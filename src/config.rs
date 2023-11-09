use anyhow::Result;
use anyhow::{Context, Error};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct UserConfig {
    pub hfs: PathBuf,
}

#[derive(Default, Serialize, Deserialize, Debug, Clone)]
pub struct AppCache {
    pub window_position: [i32; 2],
}

impl AppCache {
    const APP_NAME: &'static str = "houdini.license.chaser";
    const CACHE_FILE: &'static str = "cache.json";

    fn cache_file() -> Result<PathBuf> {
        dirs::cache_dir()
            .ok_or(Error::msg("Platform cache directory not found"))
            .map(|path| path.join(AppCache::APP_NAME).join(AppCache::CACHE_FILE))
    }
    pub fn load() -> Result<AppCache> {
        let reader =
            std::fs::File::open(Self::cache_file()?).context("Could not open cache file")?;
        serde_json::from_reader(reader).context("Error loading cache.json")
    }

    pub fn save(&self) -> Result<()> {
        let cache_file = Self::cache_file()?;
        let parent = cache_file.parent().unwrap();
        if !parent.exists() {
            std::fs::create_dir(parent).context("Could not create app cache dir")?;
        }
        let mut file =
            std::fs::File::create(cache_file).context("Could not create app cache file")?;
        serde_json::to_writer_pretty(file, self).context("Could not serialize cache")?;
        Ok(())
    }

    pub fn store_windows_pos(&mut self, pos: [i32; 2]) {
        self.window_position = pos;
    }
}

impl UserConfig {
    pub fn load() -> Result<UserConfig> {
        Ok(UserConfig {
            hfs: Default::default(),
        })
    }

    pub fn save(&self) -> Result<()> {
        todo!()
    }
}
