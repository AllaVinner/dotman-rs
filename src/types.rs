use core::fmt;
use derive_more::{AsRef, Deref};
use serde::{
    de::{self, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use std::{
    ops::Deref,
    path::{Path, PathBuf},
};
use thiserror::Error;

use crate::utils::resolve_path;

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, PartialOrd, Ord, Deref, AsRef)]
pub struct ProjectPath(PathBuf);

#[derive(Error, Debug, Clone, PartialEq)]
#[error("path is not absolute")]
pub struct ProjectPathError;
impl ProjectPath {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, ProjectPathError> {
        let path = resolve_path(path);
        if path.is_absolute() {
            Ok(Self(path.into()))
        } else {
            Err(ProjectPathError)
        }
    }
}

impl AsRef<Path> for ProjectPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize, PartialOrd, Ord, Deref, AsRef)]
pub struct SourcePath(PathBuf);

#[derive(Error, Debug, Clone, PartialEq)]
#[error("path is absolute")]
pub struct SourcePathError;
impl SourcePath {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, SourcePathError> {
        let path = resolve_path(path);
        if path.is_absolute() {
            Err(SourcePathError)
        } else {
            Ok(Self(path.into()))
        }
    }
}

impl AsRef<Path> for SourcePath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl TryFrom<&str> for SourcePath {
    type Error = SourcePathError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Deref, AsRef)]
pub struct LinkPath(PathBuf);

#[derive(Error, Debug, Clone, PartialEq)]
#[error("path is not absolute")]
pub struct LinkPathError;
impl LinkPath {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, LinkPathError> {
        let path = resolve_path(path);
        if path.is_absolute() {
            Err(LinkPathError)
        } else {
            Ok(Self(path.into()))
        }
    }
}

impl AsRef<Path> for LinkPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl TryFrom<&str> for LinkPath {
    type Error = LinkPathError;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl Serialize for LinkPath {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let path_str = if self.0.is_absolute() {
            self.0.to_string_lossy().into_owned()
        } else {
            format!("~/{}", self.0.to_string_lossy())
        };
        serializer.serialize_str(&path_str)
    }
}

// Custom deserialization logic
impl<'de> Deserialize<'de> for LinkPath {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct MyPathVisitor;

        impl<'de> Visitor<'de> for MyPathVisitor {
            type Value = LinkPath;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a path")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                // Handle paths with tilde prefix
                if value.starts_with('~') {
                    Ok(LinkPath::new(PathBuf::from(&value[2..])).unwrap())
                } else {
                    Ok(LinkPath::new(PathBuf::from(value)).unwrap())
                }
            }
        }
        deserializer.deserialize_str(MyPathVisitor)
    }
}
