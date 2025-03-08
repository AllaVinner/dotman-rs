use crate::{config, types::ProjectPath, CONFIG_FILE_NAME};
use std::{fs::create_dir_all, io};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum InitError {
    #[error("project already initialized")]
    ConfigFileExists,
    #[error("could not ensure project folder path: {0}")]
    EnsureFoldersError(#[from] io::Error),
    #[error("could not write dotman config: {0}")]
    WriteError(#[from] config::WriteError),
}

pub fn init_project(project: &ProjectPath) -> Result<(), InitError> {
    use InitError as E;
    let config_path = project.join(CONFIG_FILE_NAME);
    if config_path.exists() {
        return Err(E::ConfigFileExists);
    }
    create_dir_all(project)?;
    let config = config::DotConfig::new();
    config.write(&config_path)?;
    return Ok(());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::root_dir;
    use rstest::rstest;
    use std::{fs::create_dir, path::PathBuf};

    #[rstest]
    fn basic_init(root_dir: &PathBuf) {
        let test_dir = ProjectPath::new(root_dir.join("basic_init")).unwrap();
        create_dir(&test_dir).expect("Could not create `test_dir`.");
        let config = test_dir.join(CONFIG_FILE_NAME);
        let _ = create_dir(&test_dir);
        assert!(init_project(&test_dir).is_ok());
        match config::DotConfig::from_file(config) {
            Ok(c) => assert_eq!(c, config::DotConfig::new()),
            Err(e) => assert!(false, "{}", e),
        }
    }

    #[rstest]
    fn basic_deep_init(root_dir: &PathBuf) {
        let test_dir = ProjectPath::new(root_dir.join("basic_deep_init")).unwrap();
        assert!(!test_dir.exists());
        let config = test_dir.join(CONFIG_FILE_NAME);
        let _ = create_dir(&test_dir);
        assert!(init_project(&test_dir).is_ok());
        assert!(test_dir.exists());
        match config::DotConfig::from_file(config) {
            Ok(c) => assert_eq!(c, config::DotConfig::new()),
            Err(e) => assert!(false, "{}", e),
        }
    }
}
