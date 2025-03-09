use std::fs::{self, create_dir_all};
use std::{
    env::{self, current_dir},
    io,
    path::Path,
};

use crate::{
    add, init,
    types::{LinkPath, ProjectPath, SourcePath},
    utils::{normalize_path, AbsPath},
    HOME_ENV,
};

#[derive(Debug)]
pub struct ExampleDotfile {
    pub link: LinkPath,
    pub source: SourcePath,
}

#[derive(Debug)]
pub struct ExampleStructure {
    pub home: AbsPath,
    pub dotfiles: ProjectPath,
    pub nvim: ExampleDotfile,
    pub bashrc: ExampleDotfile,
}

pub fn get_example_structure<P: AsRef<Path>, H: AsRef<Path>, W: AsRef<Path>>(
    base_dir: P,
    home: H,
    cwd: W,
) -> ExampleStructure {
    let base_dir = AbsPath::new(normalize_path(base_dir, &home, &cwd)).expect("");
    let home = AbsPath::new(home).expect("");
    let project = ProjectPath::new(base_dir.join("dotfiles")).expect("");
    let bashrc_link =
        LinkPath::new(base_dir.join("bashrc").strip_prefix(&home).unwrap()).expect("");
    let bashrc_source = SourcePath::new("bashrc").expect("");
    let nvim_link =
        LinkPath::new(base_dir.join("config/nvim").strip_prefix(&home).unwrap()).expect("");
    let nvim_source = SourcePath::new("nvim").expect("");
    ExampleStructure {
        home,
        dotfiles: project,
        nvim: ExampleDotfile {
            link: nvim_link,
            source: nvim_source,
        },
        bashrc: ExampleDotfile {
            link: bashrc_link,
            source: bashrc_source,
        },
    }
}

pub fn example_new_user<P: AsRef<Path>>(base_dir: P) -> io::Result<()> {
    let home = AbsPath::new(env::var(HOME_ENV).expect("Home var not set.")).expect("");
    let cwd = current_dir().expect("There is a current dir.");
    let f = get_example_structure(base_dir, &home, &cwd);
    example_new_user_from_structure(&f)
}

pub fn example_new_user_from_structure(f: &ExampleStructure) -> io::Result<()> {
    create_dir_all(&f.dotfiles)?;
    create_dir_all(&f.home.join(&f.nvim.link))?;
    fs::write(&f.home.join(&f.bashrc.link), "basrc content")?;
    fs::write(
        &f.home.join(&f.nvim.link).join("init.lua"),
        "init dot lua content",
    )?;
    Ok(())
}

pub fn example_new_machine<P: AsRef<Path>>(base_dir: P) -> io::Result<()> {
    let base_dir = base_dir.as_ref();
    let home = AbsPath::new(env::var(HOME_ENV).expect("Home var not set.")).expect("");
    let cwd = current_dir().expect("There is a current dir.");
    let f = get_example_structure(base_dir, &home, &cwd);
    example_new_machine_from_structure(&f)?;
    Ok(())
}

pub fn example_new_machine_from_structure(f: &ExampleStructure) -> io::Result<()> {
    example_new_user_from_structure(f)?;
    init::init_project(&f.dotfiles).expect("A");
    add::add(&f.home, &f.bashrc.link, &f.dotfiles, &f.bashrc.source).expect("B");
    add::add(&f.home, &f.nvim.link, &f.dotfiles, &f.nvim.source).expect("C");
    fs::remove_file(&f.home.join(&f.bashrc.link))?;
    fs::remove_dir_all(&f.home.join(&f.nvim.link).parent().unwrap())?;
    Ok(())
}

pub fn example_new_dotfile_from_structure(f: &ExampleStructure) -> io::Result<()> {
    example_new_user_from_structure(f)?;
    init::init_project(&f.dotfiles).expect("A");
    fs::rename(&f.home.join(&f.nvim.link), &f.dotfiles.join(&f.nvim.source))?;
    fs::rename(
        &f.home.join(&f.bashrc.link),
        &f.dotfiles.join(&f.bashrc.source),
    )?;
    Ok(())
}

pub fn example_new_dotfile<P: AsRef<Path>>(base_dir: P) -> io::Result<()> {
    let base_dir = base_dir.as_ref();
    let home = AbsPath::new(env::var(HOME_ENV).expect("Home var not set.")).expect("");
    let cwd = current_dir().expect("There is a current dir.");
    let f = get_example_structure(base_dir, &home, &cwd);
    example_new_dotfile_from_structure(&f)?;
    Ok(())
}
