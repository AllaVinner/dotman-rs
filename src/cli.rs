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
    Setup(SetupArgs),
}

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    /// Directory to init dotman project in
    #[arg(default_value = ".")]
    pub project: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct SetupArgs {
    /// Setup for main
    pub setup_type: SetupType,
    /// Base Directory
    #[arg(default_value = ".")]
    pub base_dir: PathBuf,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum SetupType {
    NewUser,
}
