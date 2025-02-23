use std::{
    error::Error,
    fmt, fs, io,
    os::unix::fs as unix_fs,
    path::{Path, PathBuf},
};

#[derive(Debug)]
pub enum AddError {
    IO(io::Error),
    SourceNotFound(PathBuf),
    TargetExists(PathBuf),
    RollbackError((io::Error, io::Error)),
}

impl From<io::Error> for AddError {
    fn from(err: io::Error) -> Self {
        AddError::IO(err)
    }
}

impl fmt::Display for AddError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::IO(err) => write!(f, "IO error: {}", err),
            Self::SourceNotFound(source) => {
                write!(f, "Source path not found: {}", source.display())
            }
            Self::TargetExists(target) => write!(f, "Target already exists: {}", target.display()),
            Self::RollbackError((original_err, rollback_err)) => write!(
                f,
                "Rollback Error. System might not be in a valid state. Original error: {}. Rollback error: {}",
                original_err, rollback_err
            ),
        }
    }
}

impl Error for AddError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::RollbackError(e) => Some(&e.0),
            _ => None,
        }
    }
}

fn naive_add<S: AsRef<Path>, T: AsRef<Path>>(source: S, target: T) -> Result<(), io::Error> {
    fs::rename(&source, &target)?;
    unix_fs::symlink(&target, &source)?;
    // TODO: add record stuff
    return Ok(());
}

fn rollback_add<S: AsRef<Path>, T: AsRef<Path>>(source: S, target: T) -> Result<(), io::Error> {
    let source = source.as_ref();
    let target = target.as_ref();
    if source.exists() && source.is_symlink() {
        fs::remove_file(source)?;
    }
    if target.exists() && !source.exists() {
        fs::rename(target, source)?;
    }
    Ok(())
}

fn atomic_add<S: AsRef<Path>, T: AsRef<Path>>(source: S, target: T) -> Result<(), AddError> {
    let source = source.as_ref();
    let target = target.as_ref();
    if !source.exists() {
        return Err(AddError::SourceNotFound(source.to_path_buf()));
    }
    if target.exists() {
        return Err(AddError::TargetExists(target.to_path_buf()));
    }
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    let result = naive_add(source, target);
    if let Err(err) = result {
        if let Err(rollback_error) = rollback_add(source, target) {
            return Err(AddError::RollbackError((err, rollback_error)));
        }
        return Err(AddError::IO(err));
    }
    return Ok(());
}

pub fn add<S: AsRef<Path>, T: AsRef<Path>>(source: S, target: T) -> Result<(), AddError> {
    atomic_add(source, target)
}

#[cfg(test)]
mod tests {
    use rstest::rstest;
    use std::fs::create_dir;

    use super::*;
    use crate::setup::setup_new_user;
    use crate::tests::root_dir;

    #[rstest]
    fn basic_add(root_dir: &PathBuf) {
        let test_dir = root_dir.join("basic_add");
        create_dir(&test_dir).expect("Could not create test directory.");
        let dotfiles = test_dir.join("dotfiles");
        let bashrc_source = test_dir.join("bashrc");
        let bashrc_target = dotfiles.join("bashrc");
        let nvim_source = test_dir.join("config/nvim");
        let nvim_target = dotfiles.join("nvim");
        setup_new_user(test_dir).expect("Could not setup folder structure.");
        add(&bashrc_source, &bashrc_target).expect("Could not add bashrc to target.");
        assert!(&bashrc_source.is_symlink());
        assert!(&bashrc_target.exists());

        assert!(&!nvim_source.is_symlink());
        assert!(&nvim_source.exists());
        assert!(&!nvim_target.exists());
        assert!(&!nvim_target.join("init.lua").exists());
        add(&nvim_source, &nvim_target).expect("Could not add bashrc to target.");
        assert!(&nvim_source.is_symlink());
        assert!(&nvim_target.exists());
        assert!(&nvim_target.join("init.lua").exists());
    }
}
