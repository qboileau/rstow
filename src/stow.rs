use quicli::prelude::*;
use im::vector::*;
use failure::Error;

use std::io;
use std::result::Result;
use std::path::{Path, PathBuf};
use std::collections::LinkedList;

use fileutils::*;
use errors::*;
use operations::*;

pub(crate) fn stow_path<'a>(source_path: &'a Path,
                            target_path: &'a Path,
                            force: bool,
                            backup: bool,
                            operations: &'a mut Vector<FSOperation>) -> Result<TraversOperation, AppError> {

    let target_is_directory = source_path.is_dir();
    let target_exist = target_path.exists();
    let target_is_symlink = is_symlink(target_path);
    let is_valid_symlink = check_symlink(target_path, source_path);


    debug!("Stow {} -> {}", source_path.display(), target_path.display());
    trace!("Flags: (Force:{}, Backup:{}) Target state: (Dir:{}, Exist:{}, Symlink:{} to {:?}, Valid symlink:{})",
           force,
           backup,
           target_is_directory,
           target_exist,
           target_is_symlink,
           get_symlink_target(target_path),
           is_valid_symlink
    );

    let stop_if_directory = || -> Result<TraversOperation, AppError> {
        if target_is_directory {
            Ok(TraversOperation::StopPathRun)
        } else {
            Ok(TraversOperation::Continue)
        }
    };

    let symlink_operation = FSOperation::CreateSymlink { source: source_path.to_path_buf(), target: target_path.to_path_buf() };

    match (target_exist, target_is_symlink, target_is_directory, force) {
        (true, true, _, _) => {
            //A symbolic link already exist
            if is_valid_symlink {
                //ignore target exist if it's already the good symlink
                debug!("Valid symlink {} already exist, nothing to do", target_path.display());
                operations.push_back(FSOperation::Nothing{ path: target_path.to_path_buf(), cause: "Valid symbolic link".to_owned() });
                stop_if_directory()
            } else {
                if target_is_directory {
                    if force {
                        debug!("Invalid symlink {} already exist on directory. Replace by a physical directory and rebuild child links.", target_path.display());
                        operations.push_back(FSOperation::BreakDirectoryLink(target_path.to_path_buf()));
                        Ok(TraversOperation::Continue)
                    } else {
                        debug!("Error: Invalid symlink {} already exist on directory.", target_path.display());
                        Err(AppError::StowPathError {
                            source: ErrorPath::from(source_path),
                            target: ErrorPath::from(target_path),
                            cause: "Target directory already exist as a symlink somewhere else. Not supported yet".to_string()
                        })
                    }
                } else {
                    if force {
                        debug!("Invalid symlink {} already exist on file. Override because of force flag.", target_path.display());
                        log!(Level::Warn, "Path symlink {} already exist and will be override", target_path.display());
                        operations.push_back(FSOperation::Delete(target_path.to_path_buf()));
                        operations.push_back(symlink_operation);
                        stop_if_directory()
                    } else {
                        debug!("Error: Invalid symlink {} already exist on file without force flag.", target_path.display());
                        Err(AppError::StowPathError {
                            source: ErrorPath::from(source_path),
                            target: ErrorPath::from(target_path),
                            cause: "Target file already exist as a symlink somewhere else. Try with -f force flag to override symlink".to_string()
                        })
                    }
                }
            }
        }
        (true, false, false, true) => {
            debug!("Target file {} already exist. Check force and backup flag states before create symlink.", target_path.display());
            // A real file already exist and force flag is set
            if backup {
                operations.push_back(FSOperation::Backup(target_path.to_path_buf()));
            } else {
                log!(Level::Warn, "Path {} already exist and will be override !", target_path.display());
                operations.push_back(FSOperation::Delete(target_path.to_path_buf()));
            }
            operations.push_back(symlink_operation);
            Ok(TraversOperation::Continue)
        }
        (true, false, false, false) => {
            debug!("Error: Invalid target file {} already exist without force flag.", target_path.display());
            // A real file already exist but force flag is not set => ERROR
            Err(AppError::StowPathError {
                source: ErrorPath::from(source_path),
                target: ErrorPath::from(target_path),
                cause: "Target file already physically exist. Set -f flag to force override".to_string()
            })
        }
        (true, false, true, _) => {
            debug!("Target directory {} exist. Continue on children", target_path.display());
            //break for existing directory
            Ok(TraversOperation::Continue)
        }
        (false, _, _, _) => {
            debug!("Target file {} not exist. Create symlink.", target_path.display());
            operations.push_back(symlink_operation);
            stop_if_directory()
        }
    }
}

#[cfg(test)]
mod test_stow {
    use super::*;
    use test_utils::*;
    use std::borrow::BorrowMut;
    use std::fs::*;

    const FORCE: bool = true;
    const BACKUP: bool = true;
    const NO_FORCE: bool = false;
    const NO_BACKUP: bool = false;

    #[test]
    fn test_file() {
        with_test_directories("test_file".as_ref(),|source: &PathBuf, target: &PathBuf| {
            let source_file = add_file_to("file.txt", source.as_path()).unwrap();
            let target_file = target.join("file.txt");

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_file.as_path(), target_file.as_path(), NO_FORCE, NO_BACKUP, operations.borrow_mut());

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::Continue);

            let mut iter = operations.iter();
            let value = iter.next().unwrap();
            assert_eq!(value, &FSOperation::CreateSymlink { source: source_file, target: target_file });
            assert_eq!(iter.next(), None);
        });
    }

    #[test]
    fn test_directory() {
        with_test_directories("test_directory".as_ref(), |source: &PathBuf, target: &PathBuf| {
            let source_dir = add_directory_to("subDir", source.as_path()).unwrap();
            let target_dir = target.join("subDir");

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_dir.as_path(), target_dir.as_path(), NO_FORCE, NO_BACKUP, operations.borrow_mut());

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::StopPathRun);

            let mut iter = operations.iter();
            let value = iter.next().unwrap();
            assert_eq!(value, &FSOperation::CreateSymlink { source: source_dir, target: target_dir });
            assert_eq!(iter.next(), None);
        });
    }

    #[test]
    fn test_existing_file() {
        with_test_directories("test_existing_file".as_ref(), |source: &PathBuf, target: &PathBuf| {
            let source_file = add_file_to("file.txt", source.as_path()).unwrap();
            let target_file = add_file_to("file.txt", target.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_file.as_path(), target_file.as_path(), NO_FORCE, NO_BACKUP, operations.borrow_mut());

            // return an error
            assert!(result.is_err());
            assert!(operations.is_empty());
        });
    }

    #[test]
    fn test_existing_directory() {
        with_test_directories("test_existing_directory".as_ref(),|source: &PathBuf, target: &PathBuf| {
            let source_file = add_directory_to("subDir", source.as_path()).unwrap();
            let target_file = add_directory_to("subDir", target.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_file.as_path(), target_file.as_path(), NO_FORCE, NO_BACKUP, operations.borrow_mut());

            // return an error
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::Continue);
            assert!(operations.is_empty());
        });
    }

    #[test]
    fn test_existing_file_with_force() {
        with_test_directories("test_existing_file_with_force".as_ref(), |source: &PathBuf, target: &PathBuf| {
            let source_file = add_file_to("file.txt", source.as_path()).unwrap();
            let target_file = add_file_to("file.txt", target.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_file.as_path(), target_file.as_path(), FORCE, NO_BACKUP, operations.borrow_mut());

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::Continue);

            // Delete then Symlink
            let mut iter = operations.iter();
            assert_eq!(iter.next().unwrap(), &FSOperation::Delete(target_file.to_path_buf()));
            assert_eq!(iter.next().unwrap(), &FSOperation::CreateSymlink { source: source_file, target: target_file });
            assert_eq!(iter.next(), None);
        });
    }

    #[test]
    fn test_existing_directory_with_force() {
        with_test_directories("test_existing_directory_with_force".as_ref(),|source: &PathBuf, target: &PathBuf| {
            let source_file = add_directory_to("subDir", source.as_path()).unwrap();
            let target_file = add_directory_to("subDir", target.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_file.as_path(), target_file.as_path(), FORCE, NO_BACKUP, operations.borrow_mut());

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::Continue);
            assert!(operations.is_empty());
        });
    }

    #[test]
    fn test_existing_file_with_force_backup() {
        with_test_directories("test_existing_file_with_force_backup".as_ref(), |source: &PathBuf, target: &PathBuf| {
            let source_file = add_file_to("file.txt", source.as_path()).unwrap();
            let target_file = add_file_to("file.txt", target.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_file.as_path(), target_file.as_path(), FORCE, BACKUP, operations.borrow_mut());

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::Continue);

            // Backup then Symlink
            let mut iter = operations.iter();
            assert_eq!(iter.next().unwrap(), &FSOperation::Backup(target_file.to_path_buf()));
            assert_eq!(iter.next().unwrap(), &FSOperation::CreateSymlink { source: source_file, target: target_file });
            assert_eq!(iter.next(), None);
        });
    }

    #[test]
    fn test_existing_directory_with_force_backup() {
        with_test_directories("test_existing_directory_with_force_backup".as_ref(),|source: &PathBuf, target: &PathBuf| {
            let source_file = add_directory_to("subDir", source.as_path()).unwrap();
            let target_file = add_directory_to("subDir", target.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_file.as_path(), target_file.as_path(), FORCE, BACKUP, operations.borrow_mut());

            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::Continue);
            assert!(operations.is_empty());
        });
    }

    #[test]
    fn test_existing_valid_link_file() {
        with_test_directories("test_existing_valid_link_file".as_ref(), |source: &PathBuf, target: &PathBuf| {
            let source_file = add_file_to("file.txt", source.as_path()).unwrap();
            let target_file = target.join("file.txt");
            create_symlink(source_file.as_path(), target_file.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_file.as_path(), target_file.as_path(), NO_FORCE, NO_BACKUP, operations.borrow_mut());

            // nothing to do, continue traversing
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::Continue);

            let mut iter = operations.iter();
            assert_eq!(iter.next().unwrap(), &FSOperation::Nothing{ path: target_file.to_path_buf(), cause: "Valid symbolic link".to_owned() });
            assert_eq!(iter.next(), None);
        });
    }

    #[test]
    fn test_existing_valid_link_directory() {
        with_test_directories("test_existing_valid_link_directory".as_ref(),|source: &PathBuf, target: &PathBuf| {
            let source_file = add_directory_to("subDir", source.as_path()).unwrap();
            let target_file = target.join("subDir");
            create_symlink(source_file.as_path(), target_file.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_file.as_path(), target_file.as_path(), NO_FORCE, NO_BACKUP, operations.borrow_mut());

            // return stop directory traversing
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::StopPathRun);

            let mut iter = operations.iter();
            assert_eq!(iter.next().unwrap(), &FSOperation::Nothing{ path: target_file.to_path_buf(), cause: "Valid symbolic link".to_owned() });
            assert_eq!(iter.next(), None);
        });
    }

    #[test]
    fn test_existing_invalid_link_file() {
        with_test_directories("test_existing_invalid_link_file".as_ref(), |source: &PathBuf, target: &PathBuf| {
            let source_file = add_file_to("file.txt", source.as_path()).unwrap();
            let target_file = target.join("file.txt");

            let other_source_dir: PathBuf = source.parent().unwrap().to_path_buf().join("somewhere");
            create_dir_all(other_source_dir.as_path()).unwrap();
            let other_source = add_file_to("file.txt", other_source_dir.as_path()).unwrap();
            create_symlink(other_source.as_path(), target_file.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_file.as_path(), target_file.as_path(), NO_FORCE, NO_BACKUP, operations.borrow_mut());

            // nothing to do, continue traversing
            assert!(result.is_err());
            assert!(operations.is_empty());
        });
    }

    #[test]
    fn test_existing_invalid_link_directory() {
        with_test_directories("test_existing_invalid_link_directory".as_ref(),|source: &PathBuf, target: &PathBuf| {
            let source_file = add_directory_to("subDir", source.as_path()).unwrap();
            let target_file = target.join("subDir");

            let other_source_dir: PathBuf = source.parent().unwrap().to_path_buf().join("somewhere");
            create_dir_all(other_source_dir.as_path()).unwrap();
            create_symlink(other_source_dir.as_path(), target_file.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_file.as_path(), target_file.as_path(), NO_FORCE, NO_BACKUP, operations.borrow_mut());

            // return stop directory traversing
            assert!(result.is_err());
            assert!(operations.is_empty());
        });
    }

    #[test]
    fn test_existing_invalid_link_file_with_force() {
        with_test_directories("test_existing_invalid_link_file_with_force".as_ref(), |source: &PathBuf, target: &PathBuf| {
            let source_file = add_file_to("file.txt", source.as_path()).unwrap();
            let target_file = target.join("file.txt");

            let other_source_dir: PathBuf = source.parent().unwrap().to_path_buf().join("somewhere");
            create_dir_all(other_source_dir.as_path()).unwrap();
            let other_source = add_file_to("file.txt", other_source_dir.as_path()).unwrap();
            create_symlink(other_source.as_path(), target_file.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_file.as_path(), target_file.as_path(), FORCE, NO_BACKUP, operations.borrow_mut());

            // nothing to do, continue traversing
            assert!(result.is_ok());

            // Delete old symlink then Symlink to new source
            let mut iter = operations.iter();
            assert_eq!(iter.next().unwrap(), &FSOperation::Delete(target_file.to_path_buf()));
            assert_eq!(iter.next().unwrap(), &FSOperation::CreateSymlink { source: source_file, target: target_file });
            assert_eq!(iter.next(), None);
        });
    }

    #[test]
    fn test_existing_invalid_link_directory_with_force() {
        with_test_directories("test_existing_invalid_link_directory_with_force".as_ref(),|source: &PathBuf, target: &PathBuf| {
            let source_file = add_directory_to("subDir", source.as_path()).unwrap();
            let target_file = target.join("subDir");

            let other_source_dir: PathBuf = source.parent().unwrap().to_path_buf().join("somewhere");
            create_dir_all(other_source_dir.as_path()).unwrap();
            create_symlink(other_source_dir.as_path(), target_file.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = stow_path(source_file.as_path(), target_file.as_path(), FORCE, NO_BACKUP, operations.borrow_mut());

            assert!(result.is_ok());

            let mut iter = operations.iter();
            assert_eq!(iter.next().unwrap(), &FSOperation::BreakDirectoryLink(target_file.to_path_buf()));
            assert_eq!(iter.next(), None);
        });
    }
}
