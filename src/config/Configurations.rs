extern crate config;

use std::collections::HashMap;

use config::ConfigError;

use glob::glob;
pub use config::{Config,File,Value};
pub use serde::de::{Deserialize};

#[warn(dead_code)]
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;

pub struct AttestationConfig{
    config : Config,
}

impl AttestationConfig {
    pub fn new()->Self {
        let mut settings = Config::default();
        settings
            .merge(glob("conf/*")
                .unwrap()
                .map(|path| {
                    File::from(path.unwrap())})
                .collect::<Vec<_>>())
            .unwrap();

        AttestationConfig {
            config: settings
        }
    }

    pub fn get<'de, T: Deserialize<'de>>(&self, key: &str) -> ConfigResult<T> {
        self.config.get(key)
    }

    pub fn get_int(&self, key: &str) -> ConfigResult<i64>{
        self.config.get_int(key)
    }

    pub fn get_str(&self, key: &str) -> ConfigResult<String> {
        self.config.get_str(key)
    }

    pub fn get_bool(&self, key: &str) -> ConfigResult<bool> {
        self.config.get_bool(key)
    }

    pub fn get_array(&self, key: &str) -> ConfigResult<Vec<Value>> {
        self.config.get_array(key)
    }

    pub fn get_table(&self, key: &str) -> ConfigResult<HashMap<String,Value>> {
        self.config.get_table(key)
    }

    pub fn get_float(&self, key: &str) -> ConfigResult<f64> {
        self.config.get_float(key)
    }
}

#[cfg(test)]
mod tests {
    use actix_web::test::config;

    use super::AttestationConfig;

    #[test]
    fn test_load() -> () {
        let config = AttestationConfig::new();
        println!("{:?}",config.get_table("debug"));
    }
}