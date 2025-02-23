use std::{
    error::Error,
    fmt::{self, Display},
    fs::create_dir_all,
    io,
    path::Path,
};

use crate::{config, CONFIG_FILE_NAME};

#[derive(Debug)]
pub enum InitError {
    ConfigFileExists,
    EnsureFoldersError(io::Error),
    WriteError(config::WriteError),
}

impl Error for InitError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            InitError::WriteError(e) => Some(e),
            _ => None,
        }
    }
}

impl Display for InitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ConfigFileExists => write!(f, "Project already initialized"),
            Self::EnsureFoldersError(e) => write!(f, "{}", e),
            Self::WriteError(e) => write!(f, "{}", e),
        }
    }
}

pub fn init_project<P: AsRef<Path>>(project: P) -> Result<(), InitError> {
    use InitError as E;
    let project = project.as_ref();
    let config_path = project.join(CONFIG_FILE_NAME);
    if config_path.exists() {
        return Err(E::ConfigFileExists);
    }
    create_dir_all(project).map_err(|op| E::EnsureFoldersError(op))?;
    let config = config::DotConfig::new();
    config.write(&config_path).map_err(|e| E::WriteError(e))?;
    return Ok(());
}

#[cfg(test)]
mod tests {
    use std::{fs::create_dir, path::PathBuf};

    use crate::tests::root_dir;
    use rstest::rstest;

    use super::*;

    #[rstest]
    fn basic_init(root_dir: &PathBuf) {
        let test_dir = root_dir.join("basic_init");
        create_dir(&test_dir).expect("Could not create `test_dir`.");
        println!("{}", test_dir.to_str().unwrap());
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
        let test_dir = root_dir.join("basic_deep_init");
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
