use std::{
    fs::{self, create_dir, create_dir_all},
    io,
    path::{Path, PathBuf},
};

pub fn setup_new_user<P: AsRef<Path>>(base_dir: P) -> io::Result<()> {
    let base_dir = base_dir.as_ref();
    create_dir_all(base_dir)?;
    create_dir(base_dir.join("dotfiles"))?;
    create_dir(base_dir.join("config"))?;
    create_dir(base_dir.join("config/nvim"))?;
    fs::write(base_dir.join("bashrc"), "basrc content")?;
    fs::write(
        base_dir.join("config/nvim/init.lua"),
        "init dot lua content",
    )?;
    Ok(())
}
