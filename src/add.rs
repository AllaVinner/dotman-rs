use crate::{
    config::{self, DotConfig},
    types::{LinkPath, ProjectPath, TargetPath},
    utils::AbsPath,
    CONFIG_FILE_NAME,
};
use std::{
    fs, io,
    os::unix::fs as unix_fs,
    path::{Path, PathBuf},
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AddError {
    #[error("IO error while adding dotfile. Successfully rolled back changes. Error: {0}")]
    IO(#[from] io::Error),
    #[error(
        "IO error while adding dotfile. Unsuccessfully rolled back changes. Original Error: {original_error}. Rollback Error: {rollback_error}"
    )]
    RollbackError {
        original_error: io::Error,
        rollback_error: io::Error,
    },
    #[error("source {0} not found")]
    SourceNotFound(PathBuf),
    #[error("target {0} already exists")]
    TargetExists(PathBuf),
    #[error("project {0} not initialized")]
    ProjectNotInitialized(PathBuf),
    #[error("coulnd not read dotman config: {0}")]
    ReadConfigError(#[from] config::ReadError),
    #[error("Could not serialize config: {0}")]
    ConfigSerializationError(#[from] toml::ser::Error),
    #[error("dotfile record already present in dotman config")]
    DotfileRecordExists(PathBuf),
}

fn raw_add(
    source: &Path,
    full_target: &Path,
    config_path: &Path,
    config_content: &str,
) -> Result<(), io::Error> {
    fs::rename(source, full_target)?;
    unix_fs::symlink(&full_target, &source)?;
    fs::write(config_path, config_content)?;
    return Ok(());
}

fn rollback_add(source: &Path, target: &Path) -> Result<(), io::Error> {
    if source.exists() && source.is_symlink() {
        fs::remove_file(source)?;
    }
    if target.exists() && !source.exists() {
        fs::rename(target, source)?;
    }
    Ok(())
}

fn atomic_add(
    source: &Path,
    target: &Path,
    config: &Path,
    config_content: &str,
) -> Result<(), AddError> {
    let result = raw_add(source, target, config, config_content);
    if let Err(err) = result {
        if let Err(rollback_error) = rollback_add(source, target) {
            return Err(AddError::RollbackError {
                original_error: err,
                rollback_error,
            });
        }
        return Err(AddError::IO(err));
    }
    return Ok(());
}

fn add_home_dotfile(
    home: &AbsPath,
    link: &LinkPath,
    project: &ProjectPath,
    target: &TargetPath,
) -> Result<(), AddError> {
    let abs_source = home.join(link);
    let abs_target = project.join(target);
    if !abs_source.exists() {
        return Err(AddError::SourceNotFound(abs_source));
    }
    if abs_target.exists() {
        return Err(AddError::TargetExists(abs_target));
    }
    let abs_config = project.join(CONFIG_FILE_NAME);
    if !abs_config.exists() {
        return Err(AddError::ProjectNotInitialized(abs_config));
    }
    let mut config = DotConfig::from_file(&abs_config)?;
    if config.dotfiles.contains_key(target) {
        return Err(AddError::DotfileRecordExists(target.to_path_buf()));
    }
    if let Some(parent) = abs_target.parent() {
        fs::create_dir_all(parent)?;
    }
    let _ = config.dotfiles.insert(target.clone(), link.clone());
    let config_content = config.to_string()?;
    atomic_add(&abs_source, &abs_target, &abs_config, &config_content)?;
    Ok(())
}

pub fn add(
    home: &AbsPath,
    link: &LinkPath,
    project: &ProjectPath,
    target: &TargetPath,
) -> Result<(), AddError> {
    add_home_dotfile(home, link, project, target)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use std::fs::create_dir;

    use super::*;
    use crate::init;
    use crate::setup::setup_new_user;
    use crate::tests::root_dir;

    #[rstest]
    fn basic_add(root_dir: &PathBuf) {
        let test_dir = AbsPath::new(root_dir.join("basic_add")).unwrap();
        create_dir(&test_dir).expect("Could not create test directory.");
        let dotfiles = ProjectPath::new(test_dir.join("dotfiles")).unwrap();
        let bashrc_source = LinkPath::new("bashrc").unwrap();
        let bashrc_target = TargetPath::new("bashrc").unwrap();
        let nvim_source = LinkPath::new("config/nvim").unwrap();
        let nvim_target = TargetPath::new("nvim").unwrap();
        setup_new_user(&test_dir).expect("Could not setup folder structure.");
        init::init_project(&dotfiles).unwrap();
        add(&test_dir, &bashrc_source, &dotfiles, &bashrc_target)
            .expect("Could not add bashrc to target.");
        assert!(&test_dir.join(&bashrc_source).is_symlink());
        assert!(&dotfiles.join(&bashrc_target).exists());
        assert!(!test_dir.join(&nvim_source).is_symlink());
        assert!(test_dir.join(&nvim_source).exists());
        assert!(!dotfiles.join(&nvim_target).exists());
        assert!(!dotfiles.join(&nvim_target).join("init.lua").exists());
        add(&test_dir, &nvim_source, &dotfiles, &nvim_target)
            .expect("Could not add bashrc to target.");
        assert!(test_dir.join(&nvim_source).is_symlink());
        assert!(dotfiles.join(&nvim_target).exists());
        assert!(dotfiles.join(&nvim_target).join("init.lua").exists());
    }
}
