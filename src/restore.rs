use std::{fs, io, os::unix::fs as unix_fs, path::Path};

use thiserror::Error;

use crate::{
    config::{self, DotConfig},
    types::ProjectPath,
    utils::AbsPath,
    CONFIG_FILE_NAME,
};

#[derive(Error, Debug)]
pub enum RestoreError {
    #[error("IO error while restoring dotfile. Successfully rolled back changes. IO-Error: {0}")]
    IO(#[from] io::Error),
    #[error(
        "IO error while adding dotfile. Unsuccessfully rolled back changes.\n Original Error: {original_error}\n Rollback Error: {rollback_error}"
    )]
    RollbackError {
        original_error: io::Error,
        rollback_error: io::Error,
    },
    #[error("project not initialized")]
    ProjectNotInitialized,
    #[error("coulnd not read dotman config: {0}")]
    ReadConfigError(#[from] config::ReadError),
    #[error("dotfile not found")]
    DotfileNotFound,
    #[error("link occupied")]
    LinkOccupied,
}

fn raw_restore(abs_source: &Path, abs_link: &Path) -> Result<(), io::Error> {
    fs::remove_file(abs_link)?;
    fs::rename(abs_source, abs_link)?;
    // TODO: Remove entry from config
    Ok(())
}

fn rollback_restore(abs_source: &Path, abs_link: &Path) -> Result<(), io::Error> {
    if !abs_link.is_symlink() && !abs_link.exists() {
        unix_fs::symlink(abs_link, abs_source)?;
    }
    Ok(())
}

fn atomic_restore(abs_source: &Path, abs_link: &Path) -> Result<(), RestoreError> {
    let result = raw_restore(abs_source, abs_link);
    if let Err(err) = result {
        if let Err(rollback_error) = rollback_restore(abs_source, abs_link) {
            return Err(RestoreError::RollbackError {
                original_error: err,
                rollback_error,
            });
        }
        return Err(RestoreError::IO(err));
    }
    return Ok(());
}

pub fn restore(project: &ProjectPath, home: &AbsPath) -> Result<(), RestoreError> {
    use RestoreError as E;
    let config_path = project.join(CONFIG_FILE_NAME);
    if !config_path.exists() {
        return Err(E::ProjectNotInitialized);
    }
    let config = DotConfig::from_file(&config_path)?;
    for (source, link) in config.dotfiles.iter() {
        let abs_link = home.join(link);
        let abs_source = project.join(source);
        if !abs_source.exists() {
            return Err(E::DotfileNotFound);
        }
        if !abs_link.is_symlink() && abs_link.exists() {
            return Err(E::LinkOccupied);
        }
    }
    for (source, link) in config.dotfiles.iter() {
        let abs_link = home.join(link);
        let abs_source = project.join(source);
        atomic_restore(&abs_source, &abs_link)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{
        example::{example_complete_setup_from_structure, get_example_structure},
        tests::root_dir,
    };
    use rstest::rstest;

    #[rstest]
    fn basic_restore(root_dir: &PathBuf) {
        let test_dir = AbsPath::new(root_dir.join("basic_restore")).unwrap();
        let f = get_example_structure(&test_dir, &test_dir, &test_dir);
        example_complete_setup_from_structure(&f).unwrap();
        restore(&f.dotfiles, &f.home).expect("restoring to work");
        assert!(!&f.home.join(&f.bashrc.link).is_symlink());
        assert!(&f.home.join(&f.bashrc.link).exists());
        assert!(!&f.dotfiles.join(&f.bashrc.source).exists());
        assert!(!&f.home.join(&f.nvim.link).is_symlink());
        assert!(&f.home.join(&f.nvim.link).exists());
        assert!(!&f.dotfiles.join(&f.nvim.source).exists());
    }
}
