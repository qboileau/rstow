
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq)]
pub enum TraversOperation {
    Continue,
    StopPathRun
}

#[derive(Debug, PartialEq)]
pub enum FSOperation {
    Backup(PathBuf),
    Restore { backup: PathBuf, target: PathBuf },
    CreateSymlink { source: PathBuf, target: PathBuf },
    Delete(PathBuf)
}

#[test]
fn test_fsoperation_equals() {
    //test Backup
    assert_eq!(FSOperation::Backup(PathBuf::from("/some/path")), FSOperation::Backup(PathBuf::from("/some/path")));

    //test Delete
    assert_eq!(FSOperation::Delete(PathBuf::from("/some/path")), FSOperation::Delete(PathBuf::from("/some/path")));

    //test Restore
    assert_eq!(FSOperation::Restore { backup: PathBuf::from("/some/path1"), target: PathBuf::from("/target/path1") },
               FSOperation::Restore { backup: PathBuf::from("/some/path1"), target: PathBuf::from("/target/path1") });

    //test CreateSymlink
    assert_eq!(FSOperation::CreateSymlink { source: PathBuf::from("/source/path1"), target: PathBuf::from("/target/path1") },
               FSOperation::CreateSymlink { source: PathBuf::from("/source/path1"), target: PathBuf::from("/target/path1") });
}

#[test]
#[should_panic]
fn test_fsoperation_backup_not_equals() {
    assert_eq!(FSOperation::Backup(PathBuf::from("/some/path")), FSOperation::Backup(PathBuf::from("/other/path")))
}

#[test]
#[should_panic]
fn test_fsoperation_delete_not_equals() {
    assert_eq!(FSOperation::Delete(PathBuf::from("/some/path")), FSOperation::Delete(PathBuf::from("/other/path")))
}

#[test]
#[should_panic]
fn test_fsoperation_restore_not_equals() {
    assert_eq!(FSOperation::Restore { backup: PathBuf::from("/some/path1"), target: PathBuf::from("/target/path1") },
               FSOperation::Restore { backup: PathBuf::from("/some/path1"), target: PathBuf::from("/other/target/path1") });
}

#[test]
#[should_panic]
fn test_fsoperation_symlink_not_equals() {
    assert_eq!(FSOperation::CreateSymlink { source: PathBuf::from("/some/path1"), target: PathBuf::from("/target/path1") },
               FSOperation::CreateSymlink { source: PathBuf::from("/different/source/path1"), target: PathBuf::from("/target/path1") });
}
