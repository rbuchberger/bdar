use std::path::PathBuf;
use std::{env, fs};

use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct Repo {
    pub name: String,
    pub source_dir: String,
    // media_size: String,
    // exclude_dirs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    repos: Vec<Repo>,
}

#[derive(Debug)]
pub struct Context {
    pub config: Config,
    pub repo: Repo,
    pub repo_name: String,
    pub db_path: PathBuf,
}

impl Context {
    pub fn new(repo_name: Option<String>) -> Self {
        let config = Self::load_config_file();
        let repo = Self::get_repo_config(&config, repo_name).clone();
        let repo_name = repo.name.clone();
        let db_path = Self::data_dir().join(format!("{}.sqlite", &repo_name));

        return Self {
            config,
            repo,
            repo_name,
            db_path,
        };
    }

    fn config_dir() -> PathBuf {
        if let Ok(v) = env::var("BDAR_CONFIG_DIR") {
            return PathBuf::from(v);
        }

        return [
            env::var("XDG_CONFIG_HOME").unwrap_or(String::from("~/.config")),
            String::from("bdar"),
        ]
        .iter()
        .collect();
    }

    fn data_dir() -> PathBuf {
        if let Ok(v) = env::var("BDAR_DATA_DIR") {
            return PathBuf::from(v);
        }

        return [
            env::var("XDG_DATA_HOME").unwrap_or(String::from("~/.local/share")),
            String::from("bdar"),
        ]
        .iter()
        .collect();
    }

    fn load_config_file() -> Config {
        let path = Self::config_dir().join("config.yml");

        let raw_text = fs::read_to_string(&path).expect(&format!(
            "Config file not found at {}.",
            &path.to_string_lossy()
        ));

        let config: Config = serde_yml::from_str(&raw_text).unwrap();

        return config;
    }

    fn get_repo_config<'a>(config: &'a Config, name: Option<String>) -> &'a Repo {
        match (config.repos.len(), name) {
            (0, _) => panic!("No repos configured. Please configure one."),
            (1, None) => return &config.repos.first().unwrap(),
            (_, None) => panic!("You must specify which repo we are working with."),
            (_, Some(name)) => {
                let repo = config
                    .repos
                    .iter()
                    .find(|&r| r.name == name)
                    .expect(&format!("Repo named {} not found in config!", name));

                return &repo;
            }
        }
    }
}
