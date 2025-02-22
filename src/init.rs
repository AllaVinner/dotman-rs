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
