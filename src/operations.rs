
use std::path::{Path, PathBuf};
use std::clone::Clone;
use std::fmt;
use std::fmt::Formatter;

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum TraversOperation {
    Continue,
    StopPathRun
}

#[derive(Debug, PartialEq, Clone)]
pub(crate) enum FSOperation {
    Backup(PathBuf),
    Restore { backup: PathBuf, target: PathBuf },
    CreateSymlink { source: PathBuf, target: PathBuf },
    CreateDirectory(PathBuf),
    Delete(PathBuf),
    BreakDirectoryLink(PathBuf) ,
    Nothing{path: PathBuf, cause: String},
}

impl fmt::Display for FSOperation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            FSOperation::Nothing {path, cause} => write!(f, "Nothing"),
            FSOperation::Backup(p) => write!(f, "Backup path {}", p.display()),
            FSOperation::Delete(p) => write!(f, "Delete path {}", p.display()),
            FSOperation::CreateDirectory(p) => write!(f, "Create directory {}", p.display()),
            FSOperation::Restore {backup, target} => write!(f, "Restore path {} as {}", backup.display(), target.display()),
            FSOperation::CreateSymlink{source, target} => write!(f, "Create symlink {} to {}", source.display(), target.display()),
            FSOperation::BreakDirectoryLink(p) => write!(f, "Break directory symlink {}", p.display()),
        }
    }
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

#[test]
fn test_fsoperation_clone() {
    let operation = FSOperation::Backup(PathBuf::from("/some/path"));
    assert_eq!(operation.clone(), operation);
}