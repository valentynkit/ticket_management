use std::{path::Path, str::FromStr};

use config::{Config, File};
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Clone, Copy)]
#[serde(try_from = "String")]
enum Environment {
    Local,
    Production,
}

impl Environment {
    const fn as_str(self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Production => "production",
        }
    }
}

impl FromStr for Environment {
    type Err = AppConfigError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "production" => Ok(Self::Production),
            value => Err(AppConfigError::WrongEnv(value.into())),
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = AppConfigError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        value.try_into()
    }
}

impl TryFrom<&'static str> for Environment {
    type Error = AppConfigError;

    fn try_from(value: &'static str) -> Result<Self, Self::Error> {
        Environment::from_str(value)
    }
}

#[derive(Error, Debug)]
pub enum AppConfigError {
    #[error("Environment is empty")]
    EmptyEnv,
    #[error("Environment couldn't be parsed {0}")]
    WrongEnv(String),
    #[error("Could not read Environment from APP_ENVIRONMENT")]
    NoAppEnv,
    #[error("Couldnot load config, inner error: {source}")]
    CouldnotLoadConfig {
        #[source]
        source: config::ConfigError,
    },
}

impl From<config::ConfigError> for AppConfigError {
    fn from(value: config::ConfigError) -> Self {
        Self::CouldnotLoadConfig { source: value }
    }
}
#[derive(Deserialize)]
pub struct AppConfig {
    port: u16,
}

impl AppConfig {
    pub fn load() -> Result<Self, AppConfigError> {
        let base_dir = env!("CARGO_MANIFEST_DIR");
        // TODO: how to get env? before reading settings?
        let env: Environment = std::env::var("APP_ENVIRONMENT")
            .map_err(|_| AppConfigError::NoAppEnv)?
            .try_into()?;

        let extension = ".yaml";
        let mut base_path = Path::new(base_dir).join("base");
        let mut env_path = Path::new(base_dir).join(env.as_str());
        base_path.add_extension(extension);
        env_path.add_extension(extension);

        // TODO:
        let settings = Config::builder()
            .add_source(File::from(base_path))
            .add_source(File::from(env_path))
            .build()?;

        settings
            .try_deserialize::<AppConfig>()
            .map_err(|err| err.into())
    }

    pub(crate) fn port(&self) -> u16 {
        self.port
    }
}
