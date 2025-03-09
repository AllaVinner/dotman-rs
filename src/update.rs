use std::{fs, io, os::unix::fs as unix_fs, path::Path};

use thiserror::Error;

use crate::{
    config::{self, DotConfig},
    types::{LinkPath, ProjectPath, SourcePath},
    utils::AbsPath,
    CONFIG_FILE_NAME,
};

#[derive(Error, Debug)]
pub enum UpdateError {
    #[error("project not initialized")]
    ProjectNotInitialized,
    #[error("dotfile not found")]
    DotfileNotFound,
    #[error("link path already occupired")]
    LinkOccupied,
    #[error("coulnd not read dotman config: {0}")]
    ReadConfigError(#[from] config::ReadError),
    #[error("Could not serialize config: {0}")]
    ConfigSerializationError(#[from] toml::ser::Error),
    #[error("error while linking - rollback successfull - error {0}")]
    IO(io::Error),
    #[error(
        "error while linking - and while rolling back - error {original_error} - rollback {rollback_error}"
    )]
    RollbackError {
        original_error: io::Error,
        rollback_error: io::Error,
    },
}

fn raw_update(
    link: &Path,
    full_source: &Path,
    config_path: &Path,
    config_content: &str,
) -> Result<(), io::Error> {
    unix_fs::symlink(&full_source, &link)?;
    fs::write(config_path, config_content)?;
    return Ok(());
}

fn rollback_update(link: &Path) -> Result<(), io::Error> {
    if link.exists() && link.is_symlink() {
        fs::remove_file(link)?;
    }
    Ok(())
}

fn atomic_update(
    link: &Path,
    source: &Path,
    config: &Path,
    config_content: &str,
) -> Result<(), UpdateError> {
    let result = raw_update(link, source, config, config_content);
    if let Err(err) = result {
        if let Err(rollback_error) = rollback_update(link) {
            return Err(UpdateError::RollbackError {
                original_error: err,
                rollback_error,
            });
        }
        return Err(UpdateError::IO(err));
    }
    return Ok(());
}

pub fn update(
    home: &AbsPath,
    link: &LinkPath,
    source: &SourcePath,
    project: &ProjectPath,
) -> Result<(), UpdateError> {
    use UpdateError as E;
    let config_path = project.join(CONFIG_FILE_NAME);
    if !config_path.exists() {
        return Err(E::ProjectNotInitialized);
    }
    let abs_source = project.join(source);
    if !abs_source.exists() {
        return Err(E::DotfileNotFound);
    }
    let abs_link = home.join(link);
    if abs_link.is_symlink() || abs_link.exists() {
        return Err(E::LinkOccupied);
    }
    let mut config = DotConfig::from_file(&config_path)?;
    let _ = config.dotfiles.insert(source.clone(), link.clone());
    let config_content = config.to_string()?;
    atomic_update(&abs_link, &abs_source, &config_path, &config_content)?;
    Ok(())
}
