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

        let cause = {
            if !target_exist { "Target not found" }
            else if !target_is_symlink { "Target not a symlink" }
            else if !is_valid_symlink { "Target symlink invalid" }
            else { "unknown" }
        };

        // do nothing if target doesn't exist, is a real file or is not a symlink valid
        operations.push_back(FSOperation::Nothing { path: target_path.to_path_buf(), cause: cause.to_owned()});
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

#[cfg(test)]
mod test_unstow {
    use super::*;
    use test_utils::*;
    use std::borrow::BorrowMut;
    use std::fs::*;

    #[test]
    fn test_valid_link_file() {
        with_test_directories("unstow_test_valid_link_file".as_ref(), |source: &PathBuf, target: &PathBuf| {
            let source_file = add_file_to("file.txt", source.as_path()).unwrap();
            let target_file = target.join("file.txt");
            create_symlink(source_file.as_path(), target_file.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = unstow_path(source_file.as_path(), target_file.as_path(),  operations.borrow_mut());

            // nothing to do, continue traversing
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::Continue);

            let mut iter = operations.iter();
            assert_eq!(iter.next().unwrap(), &FSOperation::Delete(target_file.to_path_buf()));
            assert_eq!(iter.next(), None);
        });
    }

    #[test]
    fn test_valid_link_file_with_backup() {
        with_test_directories("unstow_test_valid_link_file_with_backup".as_ref(), |source: &PathBuf, target: &PathBuf| {
            let source_file = add_file_to("file.txt", source.as_path()).unwrap();
            let target_file = target.join("file.txt");
            let backup_target_file = add_file_to("backup-file.txt", target.as_path()).unwrap();

            create_symlink(source_file.as_path(), target_file.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = unstow_path(source_file.as_path(), target_file.as_path(),  operations.borrow_mut());

            // nothing to do, continue traversing
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::Continue);

            let mut iter = operations.iter();
            assert_eq!(iter.next().unwrap(), &FSOperation::Delete(target_file.to_path_buf()));
            assert_eq!(iter.next().unwrap(), &FSOperation::Restore {backup: backup_target_file.to_path_buf(), target: target_file.to_path_buf()});
            assert_eq!(iter.next(), None);
        });
    }

    #[test]
    fn test_unvalid_link_file() {
        with_test_directories("unstow_test_unvalid_link_file".as_ref(), |source: &PathBuf, target: &PathBuf| {
            let source_file = add_file_to("file.txt", source.as_path()).unwrap();
            let target_file = target.join("file.txt");

            let other_source_dir: PathBuf = source.parent().unwrap().to_path_buf().join("somewhere");
            create_dir_all(other_source_dir.as_path()).unwrap();
            let other_source = add_file_to("file.txt", other_source_dir.as_path()).unwrap();
            create_symlink(other_source.as_path(), target_file.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = unstow_path(source_file.as_path(), target_file.as_path(),  operations.borrow_mut());

            // nothing to do, continue traversing
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::Continue);

            let mut iter = operations.iter();
            assert_eq!(iter.next().unwrap(), &FSOperation::Nothing { path: target_file.to_path_buf(), cause: "Target symlink invalid".to_owned() });
            assert_eq!(iter.next(), None);
        });
    }

    #[test]
    fn test_valid_link_directory() {
        with_test_directories("unstow_test_valid_link_directory".as_ref(), |source: &PathBuf, target: &PathBuf| {
            let source_file = add_directory_to("subdir", source.as_path()).unwrap();
            let target_file = target.join("subdir");
            create_symlink(source_file.as_path(), target_file.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = unstow_path(source_file.as_path(), target_file.as_path(),  operations.borrow_mut());

            // nothing to do, continue traversing
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::Continue);

            let mut iter = operations.iter();
            assert_eq!(iter.next().unwrap(), &FSOperation::Delete(target_file.to_path_buf()));
            assert_eq!(iter.next(), None);
        });
    }

    #[test]
    fn test_valid_link_directory_with_backup() {
        with_test_directories("unstow_test_valid_link_directory_with_backup".as_ref(), |source: &PathBuf, target: &PathBuf| {
            let source_file = add_directory_to("subdir", source.as_path()).unwrap();
            let target_file = target.join("subdir");
            let backup_target_file = add_directory_to("backup-subdir", target.as_path()).unwrap();

            create_symlink(source_file.as_path(), target_file.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = unstow_path(source_file.as_path(), target_file.as_path(),  operations.borrow_mut());

            // nothing to do, continue traversing
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::Continue);

            let mut iter = operations.iter();
            assert_eq!(iter.next().unwrap(), &FSOperation::Delete(target_file.to_path_buf()));
            assert_eq!(iter.next().unwrap(), &FSOperation::Restore {backup: backup_target_file.to_path_buf(), target: target_file.to_path_buf()});
            assert_eq!(iter.next(), None);
        });
    }

    #[test]
    fn test_unvalid_link_directory() {
        with_test_directories("unstow_test_unvalid_link_directory".as_ref(), |source: &PathBuf, target: &PathBuf| {
            let source_file = add_directory_to("subdir", source.as_path()).unwrap();
            let target_file = target.join("subdir");

            let other_source_dir: PathBuf = source.parent().unwrap().to_path_buf().join("somewhere");
            create_dir_all(other_source_dir.as_path()).unwrap();
            let other_source = add_directory_to("subdir", other_source_dir.as_path()).unwrap();
            create_symlink(other_source.as_path(), target_file.as_path()).unwrap();

            let mut operations: Vector<FSOperation> = Vector::new();
            let result = unstow_path(source_file.as_path(), target_file.as_path(),  operations.borrow_mut());

            // nothing to do, continue traversing
            assert!(result.is_ok());
            assert_eq!(result.unwrap(), TraversOperation::Continue);

            let mut iter = operations.iter();
            assert_eq!(iter.next().unwrap(), &FSOperation::Nothing { path: target_file.to_path_buf(), cause: "Target symlink invalid".to_owned() });
            assert_eq!(iter.next(), None);
        });
    }

}