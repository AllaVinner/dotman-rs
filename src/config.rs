use std::{collections::BTreeMap, fs, io, path::Path};

use thiserror::Error;

use serde::{Deserialize, Serialize};
use toml;

use crate::types::{LinkPath, TargetPath};

type DotItems = BTreeMap<TargetPath, LinkPath>;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DotConfig {
    pub dot_items: DotItems,
}

#[derive(Error, Debug)]
pub enum WriteError {
    #[error("Could not serialize dotfile config due to: {0}")]
    SerializationError(#[from] toml::ser::Error),
    #[error("Could not write dotfile config due to: {0}")]
    WriteError(#[from] io::Error),
}

#[derive(Debug, Error)]
pub enum ReadError {
    #[error("Could not read dotfile config due to: {0}")]
    ReadError(#[from] io::Error),
    #[error("Could not deserialize dotfile config due to: {0}")]
    DeSerializationError(#[from] toml::de::Error),
}

impl DotConfig {
    pub fn new() -> Self {
        Self {
            dot_items: DotItems::new(),
        }
    }
    pub fn to_string(&self) -> Result<String, toml::ser::Error> {
        toml::to_string_pretty(self)
    }

    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), WriteError> {
        let config_str = self.to_string()?;
        fs::write(path.as_ref(), config_str)?;
        return Ok(());
    }

    pub fn from_file<P: AsRef<Path>>(config_path: P) -> Result<Self, ReadError> {
        let toml_content = fs::read_to_string(config_path)?;
        let config: DotConfig = toml::from_str(&toml_content)?;
        return Ok(config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let config = DotConfig {
            dot_items: DotItems::from([
                (
                    TargetPath::try_from("B").unwrap(),
                    LinkPath::try_from("a/b/c").unwrap(),
                ),
                (
                    TargetPath::try_from("A").unwrap(),
                    LinkPath::try_from("a").unwrap(),
                ),
                (
                    TargetPath::try_from("a/b").unwrap(),
                    LinkPath::try_from("a/b").unwrap(),
                ),
            ]),
        };
        let expected_str = r#"[dot_items]
A = "~/a"
B = "~/a/b/c"
"a/b" = "~/a/b"
"#;
        let actual = toml::to_string(&config).unwrap();
        println!("{}", actual);
        assert_eq!(actual, expected_str);
    }

    #[test]
    fn test_deserialize() {
        let toml_content = r#"[dot_items]
A = "a"
B = "~/a/b/c"
"a/b" = "~/a/b"
"#;
        let expected_config = DotConfig {
            dot_items: DotItems::from([
                (
                    TargetPath::try_from("B").unwrap(),
                    LinkPath::try_from("a/b/c").unwrap(),
                ),
                (
                    TargetPath::try_from("A").unwrap(),
                    LinkPath::try_from("a").unwrap(),
                ),
                (
                    TargetPath::try_from("a/b").unwrap(),
                    LinkPath::try_from("a/b").unwrap(),
                ),
            ]),
        };
        let actual: DotConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(actual, expected_config);
    }
}
