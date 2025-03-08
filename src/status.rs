use std::{collections::BTreeMap, fs};

use crate::{
    config::{self, DotConfig},
    types::{ProjectPath, SourcePath},
    utils::AbsPath,
    CONFIG_FILE_NAME,
};

type StatusSummary = BTreeMap<SourcePath, String>;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum StatusError {
    #[error("project not initialized")]
    ProjectNotInitialilzed,
    #[error("coulnd not read dotman config: {0}")]
    ReadConfigError(#[from] config::ReadError),
}

pub fn project_summary(project: &ProjectPath, home: &AbsPath) -> Result<(), StatusError> {
    use StatusError as E;
    let abs_config = project.join(CONFIG_FILE_NAME);
    if !abs_config.exists() {
        return Err(E::ProjectNotInitialilzed);
    }
    let config = DotConfig::from_file(&abs_config)?;
    for (source, link) in config.dotfiles.iter() {
        let mut missings = vec![];
        if !project.join(source).exists() {
            missings.push("source");
        }
        if !home.join(link).is_symlink() {
            missings.push("link");
        }
        if fs::read_link(home.join(link)).map_or(false, |p| p != project.join(source)) {
            missings.push("link");
        }
        let status_msg = if missings.len() == 0 {
            "Complete".to_string()
        } else if missings.len() == 1 {
            "Missing ".to_string() + missings[0]
        } else {
            missings
                .iter()
                .enumerate()
                .fold(String::from("Missing "), |mut s, (i, v)| {
                    if i == 0 {
                        s.push_str(v);
                    } else if i + 1 == missings.len() {
                        s.push_str(", and ");
                        s.push_str(v);
                    } else {
                        s.push_str(", ");
                        s.push_str(v);
                    }
                    s
                })
        };
        let path_str = source
            .to_str()
            .expect("path to be able to be converted to string");
        println!("{}", format!("{path_str}: {status_msg}"));
    }
    Ok(())
}
