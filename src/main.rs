use std::error::Error;
use std::fmt::Display;
use std::fs::create_dir_all;
use std::os::unix::fs as unix_fs;
use std::path::Path;
use std::{env, fmt, fs, io};
use std::{env::current_dir, path::PathBuf};

use clap::Parser;

mod arg_parser;
mod cli;
mod config;
mod init;
mod setup;

const HOME_ENV: &str = if cfg!(test) { "TEST_HOME" } else { "HOME" };
const CONFIG_FILE_NAME: &str = ".dotman";

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
    RollbackError((io::Error, io::Error)),
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

fn naive_add<S: AsRef<Path>, T: AsRef<Path>>(source: S, target: T) -> Result<(), io::Error> {
    fs::rename(&source, &target)?;
    unix_fs::symlink(&target, &source)?;
    // TODO: add record stuff
    return Ok(());
}

fn rollback_add<S: AsRef<Path>, T: AsRef<Path>>(source: S, target: T) -> Result<(), io::Error> {
    let source = source.as_ref();
    let target = target.as_ref();
    if source.exists() && source.is_symlink() {
        fs::remove_file(source)?;
    }
    if target.exists() && !source.exists() {
        fs::rename(target, source)?;
    }
    Ok(())
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
    let result = naive_add(source, target);
    if let Err(err) = result {
        if let Err(rollback_error) = rollback_add(source, target) {
            return Err(AddError::RollbackError((err, rollback_error)));
        }
        return Err(AddError::IO(err));
    }
    return Ok(());
}

fn normalize_path<P: AsRef<Path>, H: AsRef<Path>, W: AsRef<Path>>(
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

fn main_init() {
    match init::init_project("project") {
        Ok(()) => println!("Success create project"),
        Err(e) => eprintln!("Error creating project: {:?}", e),
    }
}

fn main_add() {
    let home = env::var(HOME_ENV).unwrap();
    let cwd = env::current_dir().unwrap();
    let source = normalize_path("data/x/y", &home, &cwd);
    let target = normalize_path("data/f", &home, &cwd);
    let source_from_home = source.strip_prefix(home).unwrap();
    match atomic_add(source, target) {
        Ok(_) => println!("Success"),
        Err(e) => eprintln!("{}", e),
    }
}

fn run_command(command: cli::Commands) -> Result<(), Box<dyn Error>> {
    match command {
        cli::Commands::Init(cmd_args) => {
            init::init_project(cmd_args.project)?;
        }
        cli::Commands::Setup(sa) => setup_project(sa.base_dir, sa.setup_type)?,
    }
    Ok(())
}

fn setup_project<P: AsRef<Path>>(base_dir: P, setup_type: cli::SetupType) -> Result<(), io::Error> {
    match setup_type {
        cli::SetupType::NewUser => setup::setup_new_user(base_dir)?,
    }
    Ok(())
}

fn main_cli() {
    let args = cli::CLI::parse();
    match run_command(args.clone().command.unwrap()) {
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
        Ok(_) => (),
    };
}

fn main() {
    main_cli();
}

#[cfg(test)]
mod tests {

    use chrono;
    use std::env::temp_dir;

    use super::*;
    use rstest::{fixture, rstest};

    #[fixture]
    #[once]
    pub fn root_dir() -> PathBuf {
        let time_format = "%Y-%m-%d_%H-%M-%S";
        let current_time = chrono::offset::Local::now();
        let current_time_str = format!("{}", current_time.format(time_format));
        let base_name = "dotman-rs-test_";
        return temp_dir().join(base_name.to_owned() + &current_time_str);
    }

    #[test]
    fn test_env_var_processing() {
        assert_eq!(HOME_ENV, "TEST_HOME");
    }

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
