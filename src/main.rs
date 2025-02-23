use std::env::current_dir;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::{env, fmt, io};

use clap::Parser;

mod add;
mod arg_parser;
mod cli;
mod config;
mod init;
mod setup;
mod utils;

const HOME_ENV: &str = if cfg!(test) { "TEST_HOME" } else { "HOME" };
const CONFIG_FILE_NAME: &str = ".dotman";

// #[derive(Debug)]
// struct DotRecord {
//     link: PathBuf,
//     target: PathBuf,
// }

fn main_add() {
    let home = env::var(HOME_ENV).unwrap();
    let cwd = env::current_dir().unwrap();
    let source = utils::normalize_path("data/x/y", &home, &cwd);
    let target = utils::normalize_path("data/f", &home, &cwd);
    let source_from_home = source.strip_prefix(home).unwrap();
    match add::add(source, target) {
        Ok(_) => println!("Success"),
        Err(e) => eprintln!("{}", e),
    }
}

fn run_command(command: cli::Commands) -> Result<(), Box<dyn Error>> {
    let home = env::var(HOME_ENV).expect("Home var not set.");
    let cwd = current_dir().expect("There is a current dir.");
    match command {
        cli::Commands::Init(cmd_args) => {
            init::init_project(cmd_args.project)?;
        }
        cli::Commands::Setup(sa) => setup_project(sa.base_dir, sa.setup_type)?,
        cli::Commands::Add(sa) => add::add(
            utils::normalize_path(sa.source, &home, &cwd),
            utils::normalize_path(sa.target, &home, &cwd),
        )?,
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
    use std::{
        env::{current_dir, temp_dir},
        fs::{self, create_dir},
    };

    use super::*;
    use rstest::{fixture, rstest};

    const TEST_BASE_DIR_ENV: &str = "TEST_BASE_DIR";

    #[fixture]
    #[once]
    pub fn root_dir() -> PathBuf {
        let test_base_dir = if let Ok(test_base_dir) = env::var(TEST_BASE_DIR_ENV) {
            let home = env::var("HOME").expect("Home var not set.");
            let cwd = current_dir().expect("There is a current dir.");
            utils::normalize_path(test_base_dir, home, cwd)
        } else {
            let time_format = "%Y-%m-%d_%H-%M-%S";
            let current_time = chrono::offset::Local::now();
            let current_time_str = format!("{}", current_time.format(time_format));
            let base_name = "dotman-rs-test_";
            temp_dir().join(base_name.to_owned() + &current_time_str)
        };
        if !test_base_dir.exists() {
            create_dir(&test_base_dir).expect("Could not create test root directory.");
        }
        return test_base_dir;
    }

    #[test]
    fn test_env_var_processing() {
        assert_eq!(HOME_ENV, "TEST_HOME");
    }
}
