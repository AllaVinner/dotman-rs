use std::{
    collections::BTreeMap,
    error::Error,
    fmt::Display,
    fs, io,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use toml;

type Target = PathBuf;
type Link = PathBuf;
type Records<K, V> = BTreeMap<K, V>;

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DotConfig {
    records: Records<Target, Link>,
}

#[derive(Debug)]
pub enum WriteError {
    SerializationError(toml::ser::Error),
    WriteError(io::Error),
}

impl Display for WriteError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializationError(e) => write!(f, "Could not serialize config: {}", e),
            Self::WriteError(e) => write!(f, "{}", e),
        }
    }
}

impl Error for WriteError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::SerializationError(e) => Some(e),
            Self::WriteError(e) => Some(e),
        }
    }
}

#[derive(Debug)]
pub enum ReadError {
    ReadError(io::Error),
    DeSerializationError(toml::de::Error),
}

impl Display for ReadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::DeSerializationError(e) => write!(f, "Could not deserialize config: {}", e),
            Self::ReadError(e) => write!(f, "{}", e),
        }
    }
}

impl Error for ReadError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::DeSerializationError(e) => Some(e),
            Self::ReadError(e) => Some(e),
        }
    }
}

impl DotConfig {
    pub fn new() -> Self {
        Self {
            records: Records::new(),
        }
    }

    pub fn write<P: AsRef<Path>>(&self, path: P) -> Result<(), WriteError> {
        use WriteError as E;
        let config_str = toml::to_string_pretty(self).map_err(|e| E::SerializationError(e))?;
        fs::write(path.as_ref(), config_str).map_err(|e| E::WriteError(e))?;
        return Ok(());
    }

    pub fn from_file<P: AsRef<Path>>(config_path: P) -> Result<Self, ReadError> {
        use ReadError as E;
        let toml_content = fs::read_to_string(config_path).map_err(|e| E::ReadError(e))?;
        let config: DotConfig =
            toml::from_str(&toml_content).map_err(|e| E::DeSerializationError(e))?;
        return Ok(config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let config = DotConfig {
            records: Records::from([
                (PathBuf::from("B"), PathBuf::from("~/a/b/c")),
                (PathBuf::from("A"), PathBuf::from("a")),
                (PathBuf::from("a/b"), PathBuf::from("~/a/b/")),
            ]),
        };
        let expected_str = r#"[records]
A = "a"
B = "~/a/b/c"
"a/b" = "~/a/b/"
"#;
        let actual = toml::to_string(&config).unwrap();
        println!("{}", actual);
        assert_eq!(actual, expected_str);
    }

    #[test]
    fn test_deserialize() {
        let toml_content = r#"[records]
A = "a"
B = "~/a/b/c"
"a/b" = "~/a/b"
"#;
        let expected_config = DotConfig {
            records: Records::from([
                (PathBuf::from("B"), PathBuf::from("~/a/b/c")),
                (PathBuf::from("A"), PathBuf::from("a")),
                (PathBuf::from("a/b"), PathBuf::from("~/a/b/")),
            ]),
        };
        let actual: DotConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(actual, expected_config);
    }
}
