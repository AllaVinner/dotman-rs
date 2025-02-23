use std::path::{Path, PathBuf};

pub fn normalize_path<P: AsRef<Path>, H: AsRef<Path>, W: AsRef<Path>>(
    p: P,
    home: H,
    cwd: W,
) -> PathBuf {
    let p = p.as_ref();
    let home = home.as_ref();
    let cwd = cwd.as_ref();
    let mut path_buff = PathBuf::new();
    use std::path::Component as C;
    if let Some(first) = p.components().next() {
        match first {
            C::Normal(c) => {
                if c.to_str().unwrap() != "~" {
                    path_buff = cwd.clone().into();
                }
            }
            C::CurDir => {
                path_buff = cwd.clone().into();
            }
            _ => (),
        }
    }
    for component in p.components() {
        match component {
            C::CurDir => (),
            C::ParentDir => {
                path_buff.pop();
            }
            C::Normal(c) => {
                if c.to_str().unwrap() == "~" {
                    path_buff = home.clone().into();
                } else {
                    path_buff.push(c);
                }
            }
            // TODO: Don't support this
            C::RootDir => {
                path_buff = PathBuf::from("/");
            }
            C::Prefix(_) => (),
        }
    }
    return path_buff;
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
}
