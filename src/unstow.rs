use im::vector::*;
use failure::Error;

use std::result::Result;
use std::path::{Path, PathBuf};
use std::collections::LinkedList;

use fileutils::*;
use errors::*;
use operations::FSOperation;
use operations::TraversOperation;

pub(crate) fn unstow_path<'a>(source_path: &'a Path, target_path: &'a Path, operations: &'a mut Vector<FSOperation>) -> Result<TraversOperation, AppError> {
    let target_is_directory = source_path.is_dir();
    let target_exist = target_path.exists();
    let target_is_symlink = is_symlink(target_path);
    let is_valid_symlink = check_symlink(target_path, source_path);
    let backup_path = build_backup_path(target_path).unwrap();
    let backup_exist = backup_path.exists();

    if !target_exist || !target_is_symlink || !is_valid_symlink {
        // do nothing if target doesn't exist, is a real file or is not a symlink valid
        return Ok(TraversOperation::Continue);
    }

    //remove symlink
    operations.push_back(FSOperation::Delete(target_path.to_path_buf()));

    //restore backup if exist
    if backup_exist {
        operations.push_back(FSOperation::Restore { backup: backup_path.to_owned(), target: target_path.to_path_buf()});
    }
    Ok(TraversOperation::Continue)
}