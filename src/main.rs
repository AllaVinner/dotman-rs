use std::ops::Add;
use std::os::unix::fs as unix_fs;
use std::path::Path;
use std::{env, fmt, fs, io};
use std::{env::current_dir, path::PathBuf};

mod arg_parser;
mod config;

#[derive(Debug)]
struct DotRecord {
    link: PathBuf,
    target: PathBuf,
}

enum AddError {
    IO(io::Error),
    SourceNotFound(PathBuf),
    TargetExists(PathBuf),
    ConfigError(String),
    RollbackError((Box<AddError>, io::Error)),
}

impl From<io::Error> for AddError {
    fn from(err: io::Error) -> Self {
        AddError::IO(err)
    }
}

impl fmt::Display for AddError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(err) => write!(f, "IO error: {}", err),
            Self::SourceNotFound(source) => {
                write!(f, "Source path not found: {}", source.display())
            }
            Self::TargetExists(target) => write!(f, "Target already exists: {}", target.display()),
            Self::ConfigError(err) => write!(f, "Something went wrong writing the config {}", err),
            Self::RollbackError((original_err, rollback_err)) => write!(
                f,
                "Rollback Error. System might not be in a valid state. Original error: {}. Rollback error: {}",
                original_err, rollback_err
            ),
        }
    }
}

fn naive_add<S: AsRef<Path>, T: AsRef<Path>>(source: S, target: T) -> Result<(), AddError> {
    fs::rename(&source, &target)?;
    unix_fs::symlink(&target, &source)?;
    // TODO: add record stuff
    return Ok(());
}

fn atomic_add<S: AsRef<Path>, T: AsRef<Path>>(source: S, target: T) -> Result<(), AddError> {
    let source = source.as_ref();
    let target = target.as_ref();
    if !source.exists() {
        return Err(AddError::SourceNotFound(source.to_path_buf()));
    }
    if target.exists() {
        return Err(AddError::TargetExists(target.to_path_buf()));
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    match naive_add(source, target) {
        Ok(_) => (),
        Err(err) => {
            if source.exists() && source.is_symlink() {
                match fs::remove_file(source) {
                    Ok(_) => (),
                    Err(rollback_err) => {
                        return Err(AddError::RollbackError((err.into(), rollback_err)));
                    }
                }
            }
            if target.exists() && !source.exists() {
                match fs::rename(target, source) {
                    Ok(_) => (),
                    Err(rollback_err) => {
                        return Err(AddError::RollbackError((err.into(), rollback_err)));
                    }
                }
            }
            return Err(err);
        }
    }
    return Ok(());
}

fn main_atomic() {
    match atomic_add("./data/x/y", "./data/f") {
        Ok(_) => println!("Success"),
        Err(e) => eprintln!("{}", e),
    }
}

fn normalize_path<P: AsRef<Path>>(p: P) -> PathBuf {
    let p = p.as_ref();
    let home = env::var("HOME").unwrap();
    let cwd = env::current_dir().unwrap();
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

fn main() {
    let cp = normalize_path("./c/b/c");
    println!("{}", cp.display());
    let cp = normalize_path("c/b/c");
    println!("{}", cp.display());
    let cp = normalize_path("~/c/b/c");
    println!("{}", cp.display());
    let cp = normalize_path("~/c/~/d");
    println!("{}", cp.display());
    let cp = normalize_path("/c/d");
    println!("{}", cp.display());
}
