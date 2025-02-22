use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

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
}

#[derive(Args, Debug, Clone)]
pub struct InitArgs {
    /// Directory to init dotman project in
    pub project: Option<PathBuf>,
}
