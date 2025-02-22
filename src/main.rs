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

#[derive(Debug)]
enum InitError {
    ConfigFileExists,
    EnsureFoldersError(io::Error),
    WriteError(config::WriteError),
}

impl Error for InitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InitError::WriteError(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigFileExists => write!(f, "Project already initialized"),
            Self::EnsureFoldersError(e) => write!(f, "{}", e),
            Self::WriteError(e) => write!(f, "{}", e),
        }
    }
}

fn init_project<P: AsRef<Path>>(project: P) -> Result<(), InitError> {
    use InitError as E;
    let project = project.as_ref();
    let config_path = project.join(CONFIG_FILE_NAME);
    if config_path.exists() {
        return Err(E::ConfigFileExists);
    }
    create_dir_all(project).map_err(|op| E::EnsureFoldersError(op))?;
    let config = config::DotConfig::new();
    config.write(&config_path).map_err(|e| E::WriteError(e))?;
    return Ok(());
}

fn main_init() {
    match init_project("project") {
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
            init_project(cmd_args.project.unwrap_or(".".into()))?;
        }
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
        Ok(o) => (),
    };
}

fn main() {
    main_cli();
}

#[cfg(test)]
mod tests {
    use super::*;

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
