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
    /// Initialize dotman project
    Init(InitArgs),
    /// Add dotfile to dotman project
    Add(AddArgs),
    /// Setup dotfiles from dotman project
    Setup(SetupArgs),
    /// Show status of dotman project
    Status(StatusArgs),
    /// Update links in dotman project
    Update(UpdateArgs),
    /// Create example file structure
    Example(ExampleArgs),
}

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    /// Directory to init dotman project in
    #[arg(default_value = ".")]
    pub project: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct ExampleArgs {
    /// Example Type
    pub example: Examples,
    /// Directory to create exmample file structure
    #[arg(default_value = ".")]
    pub base_dir: PathBuf,
}

#[derive(ValueEnum, Debug, Clone, Copy)]
pub enum Examples {
    /// Existing dotfiles and empty dotfiles folder
    NewUser,
    /// Existing dotman project but no links
    NewMachine,
    /// Existing dotman project with existing non-recorded dotfile
    NewDotfile,
}

#[derive(Args, Debug, Clone)]
pub struct AddArgs {
    /// Dotfile to add
    pub source: PathBuf,
    /// Dotman project to add to
    #[arg(default_value = ".")]
    pub project: PathBuf,
    /// Name of moved dotfile, defaults to dotfile name.
    #[arg(short, long)]
    pub target: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct SetupArgs {
    /// Project to setup
    #[arg(default_value = ".")]
    pub project: PathBuf,
    /// Dotfile to setup, defaults to all dotfiles in project
    #[arg(short, long)]
    pub dotfile: Option<PathBuf>,
}

#[derive(Args, Debug, Clone)]
pub struct StatusArgs {
    /// Project to show status of
    #[arg(default_value = ".")]
    pub project: PathBuf,
}

#[derive(Args, Debug, Clone)]
pub struct UpdateArgs {
    /// Dofile to update
    pub dotfile: PathBuf,
    /// New link path of dotfile
    pub link: PathBuf,
    /// Project of dotfile
    #[arg(default_value = ".")]
    pub project: PathBuf,
}
