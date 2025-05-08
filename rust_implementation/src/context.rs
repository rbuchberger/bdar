use std::path::PathBuf;
use std::{env, fs};

use serde::Deserialize;

use crate::Result;

#[derive(Debug, Deserialize, Clone)]
pub struct Repo {
    pub name: String,
    pub source_dir: String,
    pub exclude_globs: Option<Vec<String>>,
    // media_size: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    repos: Vec<Repo>,
}

#[derive(Debug)]
pub struct Context {
    // pub config: Config,
    pub repo: Repo,
    // pub repo_name: String,
    pub db_path: PathBuf,
}

impl Context {
    pub fn new(repo_name: Option<String>) -> Result<Self> {
        let config = Self::load_config_file()?;
        let repo = Self::get_repo_config(&config, repo_name)?.clone();
        // let repo_name = repo.name.clone();
        let db_path = Self::data_dir().join(format!("{}.sqlite", &repo.name));

        return Ok(Self {
            // config,
            repo,
            // repo_name,
            db_path,
        });
    }

    fn config_dir() -> PathBuf {
        match env::var("BDAR_CONFIG_DIR") {
            Ok(v) => return PathBuf::from(v),
            _ => [
                env::var("XDG_CONFIG_HOME").unwrap_or("~/.config".into()),
                "bdar".into(),
            ]
            .iter()
            .collect(),
        }
    }

    fn data_dir() -> PathBuf {
        match env::var("BDAR_DATA_DIR") {
            Ok(v) => return PathBuf::from(v),
            _ => [
                env::var("XDG_DATA_HOME").unwrap_or("~/.local/share".into()),
                "bdar".into(),
            ]
            .iter()
            .collect(),
        }
    }

    fn load_config_file() -> Result<Config> {
        let path = Self::config_dir().join("config.yml");

        let raw_text = fs::read_to_string(&path)?;

        let config: Config = serde_yml::from_str(&raw_text)?;

        return Ok(config);
    }

    fn get_repo_config<'a>(config: &'a Config, name: Option<String>) -> Result<&'a Repo> {
        match (config.repos.len(), name) {
            (0, _) => Err("No repos configured. Please configure one.".into()),
            (1, None) => config
                .repos
                .first()
                .ok_or("I have no idea how this happened".into()),
            (_, None) => Err("You must specify which repo we are working with.".into()),
            (_, Some(name)) => config
                .repos
                .iter()
                .find(|&r| r.name == name)
                .ok_or_else(|| format!("Repo named {} not found in config!", name).into()),
        }
    }
}
