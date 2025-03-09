use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};

#[derive(Parser, Debug, Clone)]
#[command(version, about, long_about = None)]
pub struct CLI {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// Initiate project
    Init(InitArgs),
    /// Setup example directories
    Example(ExampleArgs),
    Add(AddArgs),
    Setup(SetupArgs),
    Status(StatusArgs),
    Update(UpdateArgs),
}

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    /// Directory to init dotman project in
    #[arg(default_value = ".")]
    pub project: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct ExampleArgs {
    /// Setup for main
    pub example: Examples,
    /// Base Directory
    #[arg(default_value = ".")]
    pub base_dir: PathBuf,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Examples {
    NewUser,
    NewMachine,
    NewDotfile,
}

#[derive(Args, Debug, Clone)]
pub struct AddArgs {
    pub source: PathBuf,
    pub project: PathBuf,
    #[arg(short, long)]
    pub target: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct SetupArgs {
    #[arg(default_value = ".")]
    pub project: PathBuf,
    #[arg(short, long)]
    pub dotfile: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct StatusArgs {
    #[arg(default_value = ".")]
    pub project: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct UpdateArgs {
    pub dotfile: PathBuf,
    pub link: PathBuf,
    #[arg(default_value = ".")]
    pub project: PathBuf,
}
