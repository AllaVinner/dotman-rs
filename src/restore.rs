use std::{fs, io, os::unix::fs as unix_fs, path::Path};

use crate::{
    config::{DotConfig, ReadError},
    types::{ProjectPath, SourcePath},
    utils::AbsPath,
    CONFIG_FILE_NAME,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RestoreError {
    #[error("project not initialized")]
    ProjectNotInitialized,
    #[error("could not find dotfile")]
    DotfileNotFound,
    #[error("could not read project config")]
    ConfigReadError(#[from] ReadError),
    #[error("dotfile not recorded in config")]
    DotfileNotRecorded,
    #[error("link path is already occupied")]
    LinkOccupied,
    #[error("error while restoring source: {0}")]
    RestoreError(#[from] io::Error),
}

fn atomic_restore(link_source: &Path, link_target: &Path) -> Result<(), io::Error> {
    if let Some(parent) = link_source.parent() {
        fs::create_dir_all(parent)?;
    }
    unix_fs::symlink(&link_target, &link_source)?;
    Ok(())
}

fn restore_source(
    project: &ProjectPath,
    source: &SourcePath,
    home: &AbsPath,
) -> Result<(), RestoreError> {
    use RestoreError as E;
    let config_path = project.join(CONFIG_FILE_NAME);
    if !config_path.exists() {
        return Err(E::ProjectNotInitialized);
    }
    let abs_source = project.join(source);
    if !abs_source.exists() {
        return Err(E::DotfileNotFound);
    }
    let config = DotConfig::from_file(config_path)?;
    let link = match config.dotfiles.get(source) {
        Some(v) => v,
        None => return Err(E::DotfileNotRecorded),
    };
    let abs_link = home.join(link);
    if abs_link.is_symlink() || abs_link.exists() {
        return Err(E::LinkOccupied);
    }
    atomic_restore(&abs_link, &abs_source)?;
    Ok(())
}

pub fn restore_project(project: &ProjectPath, home: &AbsPath) -> Result<(), RestoreError> {
    use RestoreError as E;
    let config_path = project.join(CONFIG_FILE_NAME);
    if !config_path.exists() {
        return Err(E::ProjectNotInitialized);
    }
    let config = DotConfig::from_file(config_path)?;
    for (source, link) in config.dotfiles.iter() {
        let abs_link = home.join(link);
        let abs_source = project.join(source);
        if !abs_source.exists() {
            return Err(E::DotfileNotFound);
        }
        if abs_link.is_symlink() || abs_link.exists() {
            return Err(E::LinkOccupied);
        }
    }
    for (source, link) in config.dotfiles.iter() {
        let abs_link = home.join(link);
        let abs_source = project.join(source);
        atomic_restore(&abs_link, &abs_source)?;
    }
    Ok(())
}

pub fn restore_dotfile(
    project: &ProjectPath,
    source: &SourcePath,
    home: &AbsPath,
) -> Result<(), RestoreError> {
    restore_source(project, source, home)
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::{
        setup::{get_setup_structure, setup_new_machine, setup_new_machine_from_structure},
        tests::root_dir,
    };
    use rstest::rstest;

    #[rstest]
    fn basic_restore(root_dir: &PathBuf) {
        let test_dir = AbsPath::new(root_dir.join("basic_restore")).unwrap();
        let f = get_setup_structure(&test_dir, &test_dir, &test_dir);
        setup_new_machine_from_structure(&f).unwrap();
        restore_project(&f.dotfiles, &f.home).unwrap();

        let toml_content = r#"[dotfiles]
bashrc = "~/bashrc"
nvim = "~/config/nvim"
"#;

        let expected_config: DotConfig = toml::from_str(toml_content).unwrap();
        let actual_config = DotConfig::from_file(&f.dotfiles.join(CONFIG_FILE_NAME)).unwrap();
        assert_eq!(actual_config, expected_config);
    }
}
