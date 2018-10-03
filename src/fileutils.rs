
use quicli::prelude::*;

use std::io;
use std::io::{Error, ErrorKind};
use std::fs::{self};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::os::unix::fs::symlink;

pub(crate) fn create_symlink(source_path: &Path, target_path: &Path) -> io::Result<()> {
    if cfg!(target_family = "unix") {
        info!("create symbolic link {} -> {}", source_path.display(), target_path.display());
        symlink(source_path, target_path)
    } else {
        Err(Error::new(ErrorKind::Other, "OS not supported"))
    }
}

pub(crate) fn build_backup_path(path: &Path) ->io::Result<PathBuf> {
    let file_name = path.file_name()
        .and_then(|x: &OsStr| x.to_str())
        .expect("Unable to get filename");

    let parent_path = path.parent().expect("Unable to get parent directory");
    Ok(parent_path.join("backup-".to_owned()+file_name))
}

pub(crate) fn backup_path(path: &Path) -> io::Result<()> {
    let backup_path = build_backup_path(path)?;

    info!("backup {} into {}", path.display(), backup_path.as_path().display());
    fs::rename(path, backup_path.as_path())
}

pub(crate) fn restore_path(backup: &Path, target: &Path) -> io::Result<()> {
    info!("restore backup {} into {}", backup.display(), target.display());
    fs::rename(backup, target)
}

pub(crate) fn delete_path(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        info!("delete directory recursively {}", path.display());
        fs::remove_dir_all(path)
    } else {
        info!("delete file {}", path.display());
        fs::remove_file(path)
    }
}

pub(crate) fn is_symlink(path: &Path) -> bool {
    match path.symlink_metadata() {
        Ok(data) => data.file_type().is_symlink(),
        Err(_e) => false
    }
}

pub(crate) fn check_symlink(symlink_path: &Path, valid_dest: &Path) -> bool {
    match get_symlink_target(symlink_path) {
        Some(target) => valid_dest.eq(target.as_path()),
        None => false
    }
}

pub(crate) fn get_symlink_target(symlink_path: &Path) -> Option<PathBuf> {
    if is_symlink(symlink_path) {
        let error = format!("Unable to find absolute path of symlink target {}", symlink_path.display());
        let absolute_target = symlink_path.canonicalize().expect(error.as_str());
        Some(absolute_target.to_owned())
    } else {
        None
    }
}

pub(crate) fn break_directory_link(directory: &Path) -> io::Result<()> {
    let target = get_symlink_target(directory).unwrap();

    // replace symlink with real directory file
    delete_path(directory)?;
    create_dir(directory)?;

    let source_paths = fs::read_dir(target.as_path())?;
    for src_dir_entry in source_paths {
        let source_child = src_dir_entry.unwrap().path();
        let target_child = target.join(source_child.as_path().file_name().expect("Unable to get path filename"));
        create_symlink(source_child.as_path(), target_child.as_path())?;
    }

    Ok(())
}