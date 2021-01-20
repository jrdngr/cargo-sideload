use serde::{Deserialize, Serialize};
use std::{collections::HashMap, convert::TryFrom, fmt::Display, str::FromStr};

pub const CONFIG_FILE_DIR: &str = "cargo-sideload";
pub const CONFIG_FILE_NAME: &str = "config.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Config {
    pub default_registry: Option<String>,
    pub registries: HashMap<String, RegistryConfig>,
}

impl Config {
    pub fn load() -> anyhow::Result<Option<Self>> {
        match dirs::config_dir() {
            Some(config_dir) => {
                let config_path = config_dir.join(CONFIG_FILE_DIR).join(CONFIG_FILE_NAME);
                if config_path.is_file() {
                    let bytes = std::fs::read(config_path)?;
                    let config: Config = toml::from_slice(&bytes)?;
                    Ok(Some(config))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistryConfig {
    pub headers: Vec<Header>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "&str")]
#[serde(into = "String")]
pub struct Header {
    pub name: String,
    pub value: String,
}

impl FromStr for Header {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let split: Vec<&str> = s.splitn(2, ':').collect();

        if split.len() != 2 {
            anyhow::bail!("Invalid auth header format. Expected `[Header-Name]: [Header Value]`");
        }

        let name = split[0].to_string();
        let value = split[1].trim_start().to_string();

        Ok(Self { name, value })
    }
}

impl TryFrom<&str> for Header {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Header::from_str(value)
    }
}

impl Display for Header {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.name, self.value)
    }
}

impl From<Header> for String {
    fn from(header: Header) -> Self {
        header.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header() {
        let header = Header::from_str("Authorization: Bearer abcd1234").unwrap();
        assert_eq!(header.name, "Authorization");
        assert_eq!(header.value, "Bearer abcd1234");

        assert!(Header::from_str("Authorization").is_err());
        assert!(Header::from_str("").is_err());
    }

    #[test]
    fn test_config() {
        let config_str = r#"
            default_registry = "test_registry"
    
            [registries.test_registry]
            headers = [ "Authorization: Blah abcd1234" ] 
            
            [registries.other_registry]
            headers = [ 
                    "PRIVATE-KEY: abcdef",
                    "Some-Other-Header: And its value",
            ]
        "#;

        let config: Config = toml::from_str(config_str).unwrap();

        assert_eq!(config.default_registry, Some("test_registry".to_string()));

        let test_registry_config = config.registries.get("test_registry").unwrap();

        let first_header = &test_registry_config.headers[0];
        assert_eq!(first_header.name, "Authorization");
        assert_eq!(first_header.value, "Blah abcd1234");

        let other_registry_config = config.registries.get("other_registry").unwrap();
        let header = &other_registry_config.headers[0];
        assert_eq!(header.name, "PRIVATE-KEY");
        assert_eq!(header.value, "abcdef");

        let second_header = &other_registry_config.headers[1];
        assert_eq!(second_header.name, "Some-Other-Header");
        assert_eq!(second_header.value, "And its value");
    }
}
