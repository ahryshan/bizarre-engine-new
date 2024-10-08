use std::{env, sync::LazyLock};

use serde::Deserialize;
use thiserror::Error;
use toml::Table;

static CONFIG: LazyLock<Table> = LazyLock::new(init_config);

fn init_config() -> Table {
    let config_path = env::var("BE_CONFIG_PATH").unwrap_or(String::from("be_config.toml"));
    std::fs::read_to_string(config_path)
        .unwrap()
        .parse()
        .unwrap()
}

pub trait ConfigSection: for<'a> Deserialize<'a> + Default {
    fn section_name() -> &'static str;
}

#[derive(Debug, Clone, Error)]
pub enum ConfigError {
    #[error("Failed to parse config: {0}")]
    FailedToParse(#[from] toml::de::Error),
}

type ConfigResult<T> = Result<T, ConfigError>;

pub fn get_config() -> Table {
    CONFIG.clone()
}

pub fn get_config_section<C: ConfigSection>() -> ConfigResult<C> {
    if let Some(value) = CONFIG.get(C::section_name()) {
        value.clone().try_into().map_err(|err| err.into())
    } else {
        Ok(Default::default())
    }
}
