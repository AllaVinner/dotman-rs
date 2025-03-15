use std::env::current_dir;
use std::error::Error;
use std::path::Path;
use std::{env, io};

use clap::Parser;
use types::{LinkPath, ProjectPath, SourcePath};
use utils::{normalize_path, AbsPath};

mod add;
mod cli;
mod config;
mod example;
mod init;
mod restore;
mod setup;
mod status;
mod types;
mod update;
mod utils;

const HOME_ENV: &str = if cfg!(test) { "TEST_HOME" } else { "HOME" };
const CONFIG_FILE_NAME: &str = ".dotman.toml";

fn run_command(command: cli::Commands) -> Result<(), Box<dyn Error>> {
    let home = env::var(HOME_ENV).expect("Home var not set.");
    let cwd = current_dir().expect("There is a current dir.");
    match command {
        cli::Commands::Init(cmd_args) => {
            let project = ProjectPath::new(normalize_path(cmd_args.project, &home, &cwd))?;
            init::init_project(&project)?;
        }
        cli::Commands::Example(sa) => setup_project(sa.base_dir, sa.example)?,
        cli::Commands::Add(sa) => {
            let target = match sa.target {
                Some(t) => t,
                None => sa
                    .source
                    .file_name()
                    .expect("source to not be an empty path")
                    .into(),
            };
            let home = AbsPath::new(home)?;
            let link = LinkPath::new(normalize_path(sa.source, &home, &cwd).strip_prefix(&home)?)?;
            let project = ProjectPath::new(normalize_path(sa.project, &home, &cwd))?;
            let target = SourcePath::new(target)?;
            add::add(&home, &link, &project, &target)?;
        }
        cli::Commands::Setup(args) => {
            let home = AbsPath::new(home)?;
            let project = ProjectPath::new(normalize_path(args.project, &home, &cwd))?;
            match args.dotfile {
                None => setup::setup_project(&project, &home)?,
                Some(d) => {
                    let dotfile = SourcePath::new(d)?;
                    setup::setup_dotfile(&project, &dotfile, &home)?;
                }
            }
        }
        cli::Commands::Status(args) => {
            let home = AbsPath::new(home)?;
            if args.recursive {
                let base_dir = AbsPath::new(normalize_path(args.project, &home, &cwd))?;
                let projects = utils::find_dotman_projects(&base_dir);
                for project in projects {
                    status::project_summary(&project, &home)?;
                    println!("");
                }
            } else {
                let project = ProjectPath::new(normalize_path(args.project, &home, &cwd))?;
                status::project_summary(&project, &home)?;
            }
        }
        cli::Commands::Update(args) => {
            let home = AbsPath::new(home)?;
            let link = LinkPath::new(normalize_path(args.link, &home, &cwd).strip_prefix(&home)?)?;
            let dotfile = SourcePath::new(args.dotfile)?;
            let project = ProjectPath::new(normalize_path(args.project, &home, &cwd))?;
            update::update(&home, &link, &dotfile, &project)?;
        }
        cli::Commands::Restore(args) => {
            let home = AbsPath::new(home)?;
            let project = ProjectPath::new(normalize_path(args.project, &home, &cwd))?;
            match args.dotfile {
                None => restore::restore(&project, &home)?,
                Some(d) => {
                    restore::restore(&project, &home)?;
                }
            }
        }
    }
    Ok(())
}

fn setup_project<P: AsRef<Path>>(base_dir: P, setup_type: cli::Examples) -> Result<(), io::Error> {
    match setup_type {
        cli::Examples::NewUser => example::example_new_user(base_dir)?,
        cli::Examples::NewMachine => example::example_new_machine(base_dir)?,
        cli::Examples::NewDotfile => example::example_new_dotfile(base_dir)?,
        cli::Examples::CompleteSetup => example::example_complete_setup(base_dir)?,
    }
    Ok(())
}

fn main_cli() {
    let args = cli::CLI::parse();
    match run_command(args.clone().command.unwrap()) {
        Err(e) => {
            eprintln!("error: {}", e);
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
        fs::create_dir,
        path::PathBuf,
    };

    use super::*;
    use rstest::fixture;

    const TEST_BASE_DIR_ENV: &str = "TEST_BASE_DIR";

    #[fixture]
    #[once]
    pub fn root_dir() -> PathBuf {
        let test_base_dir = if let Ok(test_base_dir) = env::var(TEST_BASE_DIR_ENV) {
            let home = env::var("HOME").expect("Home env var set.");
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
