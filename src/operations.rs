
use std::path::{Path, PathBuf};
use std::clone::Clone;
use std::fmt;
use std::fmt::Formatter;
use fileutils::print_path;

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
    BreakDirectoryLink(PathBuf),
    Nothing { path: PathBuf, cause: String },
    Compound { op1: &FSOperation, op2: &FSOperation },
}

impl fmt::Display for FSOperation {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            FSOperation::Nothing {path, cause} => write!(f, "Nothing on {} : {}", print_path(f.as_path()), cause),
            FSOperation::Backup(p) => write!(f, "Backup path {}", print_path(p.as_path())),
            FSOperation::Delete(p) => write!(f, "Delete path {}", print_path(p.as_path())),
            FSOperation::CreateDirectory(p) => write!(f, "Create directory {}",print_path(p.as_path())),
            FSOperation::Restore {backup, target} => write!(f, "Restore path {} as {}", print_path(backup.as_path()), print_path(target.as_path())),
            FSOperation::CreateSymlink{source, target} => write!(f, "Create symlink {} to {}", print_path(source.as_path()), print_path(target.as_path())),
            FSOperation::BreakDirectoryLink(p) => write!(f, "Break directory symlink {}", print_path(p.as_path())),
            FSOperation::Compound {op1, op2} => write!(f, "{} then {}", op1.display(), op2.display()),
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