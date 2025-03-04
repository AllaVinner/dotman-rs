use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub struct AbsPath(PathBuf);

#[derive(Error, Debug, Clone, PartialEq)]
#[error("{0} must be absolute")]
pub struct AbsPathError(String);
impl AbsPath {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, AbsPathError> {
        let path = resolve_path(path);
        if path.is_absolute() {
            Ok(Self(path.into()))
        } else {
            Err(AbsPathError(path.to_string_lossy().to_string()))
        }
    }
}

impl AsRef<Path> for AbsPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

pub fn resolve_path<P: AsRef<Path>>(path: P) -> PathBuf {
    let path = path.as_ref();
    let mut path_buff = PathBuf::new();
    use std::path::Component as C;
    for component in path.components() {
        match component {
            C::CurDir => (),
            C::ParentDir => {
                path_buff.pop();
            }
            C::Normal(c) => path_buff.push(c),
            C::RootDir => {
                path_buff = PathBuf::from("/");
            }
            C::Prefix(_) => {
                path_buff = PathBuf::from("/");
            }
        }
    }
    return path_buff;
}

pub fn normalize_path<P: AsRef<Path>, H: AsRef<Path>, W: AsRef<Path>>(
    p: P,
    home: H,
    cwd: W,
) -> PathBuf {
    let p = p.as_ref();
    let home = home.as_ref();
    let cwd = cwd.as_ref();
    use std::path::Component as C;
    let mut comp_iter = p.components();
    let base_path = match comp_iter.next() {
        None => return PathBuf::new(),
        Some(c) => match c {
            C::CurDir => cwd.into(),
            C::ParentDir => match cwd.to_path_buf().parent() {
                Some(pb) => pb.into(),
                None => PathBuf::from("/"),
            },
            C::Normal(c) => {
                if c == "~" {
                    home.into()
                } else {
                    cwd.join(c)
                }
            }
            C::RootDir => PathBuf::from("/"),
            C::Prefix(_) => PathBuf::from("/"),
        },
    };
    let end_path: PathBuf = comp_iter.collect();
    let path = base_path.join(end_path);
    return resolve_path(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_path() {
        let home = "/h";
        let cwd = "/h/w";
        let p = "p";
        let expected = "/h/w/p";
        assert_eq!(normalize_path(p, home, cwd).to_str().unwrap(), expected);
        let p = "p/q/";
        let expected = "/h/w/p/q";
        assert_eq!(normalize_path(p, home, cwd).to_str().unwrap(), expected);
        let p = "p/q/../x";
        let expected = "/h/w/p/x";
        assert_eq!(normalize_path(p, home, cwd).to_str().unwrap(), expected);
        let p = "p/q/../.././../y";
        let expected = "/h/y";
        assert_eq!(normalize_path(p, home, cwd).to_str().unwrap(), expected);
        let p = "~/a";
        let expected = "/h/a";
        assert_eq!(normalize_path(p, home, cwd).to_str().unwrap(), expected);
        let p = "/a";
        let expected = "/a";
        assert_eq!(normalize_path(p, home, cwd).to_str().unwrap(), expected);
    }

    #[test]
    fn test_resolve_path() {
        assert_eq!(resolve_path("a/b/c"), PathBuf::from("a/b/c"));
        assert_eq!(resolve_path("a/b/.."), PathBuf::from("a"));
        assert_eq!(resolve_path("a/b/../c"), PathBuf::from("a/c"));
        assert_eq!(resolve_path("a/b/.././c"), PathBuf::from("a/c"));
        assert_eq!(resolve_path("a/b/../../c"), PathBuf::from("c"));
        assert_eq!(resolve_path("/a/b/c"), PathBuf::from("/a/b/c"));
        assert_eq!(resolve_path("/a/b/.."), PathBuf::from("/a"));
        assert_eq!(resolve_path("/a/b/../c"), PathBuf::from("/a/c"));
        assert_eq!(resolve_path("/a/b/.././c"), PathBuf::from("/a/c"));
        assert_eq!(resolve_path("/a/b/../../c"), PathBuf::from("/c"));
        assert_eq!(resolve_path("./a/b/c"), PathBuf::from("a/b/c"));
        assert_eq!(resolve_path("./a/b/.."), PathBuf::from("a"));
        assert_eq!(resolve_path("./a/b/../c"), PathBuf::from("a/c"));
        assert_eq!(resolve_path("./a/b/.././c"), PathBuf::from("a/c"));
        assert_eq!(resolve_path("./a/b/../../c"), PathBuf::from("c"));
        assert_eq!(resolve_path("~/a/b/../../c"), PathBuf::from("~/c"));
    }
}
