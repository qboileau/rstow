#![allow(unused)]

#[macro_use] extern crate quicli;
use quicli::prelude::*;

use std::io;
use std::io::{Error, ErrorKind};
use std::fs::{self};
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::os::unix::fs::symlink;
use std::collections::LinkedList;
use std::borrow::BorrowMut;
use std::borrow::Borrow;

/// Like stow but simpler and with more crabs
#[derive(Debug, StructOpt)]
struct Cli {
    // Source directory
    /// Source directory
    #[structopt(long = "source", short = "s", default_value = "./")]
    source: String,
    // Target directory
    /// Target directory
    #[structopt(long = "target", short = "t")]
    target: String,
    /// Force override files on target using a symlink
    #[structopt(long = "force", short = "f")]
    force: bool,
    /// Create a backup of the file before override it with a symlink
    #[structopt(long = "backup", short = "b")]
    backup: bool,
    /// Dry run rstaw (this will do not affect files and logs what should be done)
    #[structopt(long = "dryrun", short = "d")]
    dryrun: bool,
    // Quick and easy logging setup you get for free with quicli
    #[structopt(flatten)]
    verbosity: Verbosity,
}

enum TraversOperation {
    Continue,
    StopPathRun
}

enum FSOperation {
    Backup(PathBuf),
    CreateSymlink { source: PathBuf , target: PathBuf },
    Delete(PathBuf)
}

main!(|args: Cli, log_level: verbosity| {
    let dryrun = &args.dryrun;
    let force = &args.force;
    let backup = &args.backup;

    let source = fs::canonicalize(&args.source).expect("Unresolved absolute source path");
    let target = fs::canonicalize(&args.target).expect("Unresolved absolute target path");

    info!("Stow from Source {:?} to target {:?}", source.display(), target.display());
    visit_sync(source.as_path(), target.as_path(), *dryrun, *force, *backup).expect("An error occurred when visiting directories") ;

});

fn visit_sync(source: &Path, target: &Path, dryrun: bool, force: bool, backup: bool)-> io::Result<()> {
    let source_paths = fs::read_dir(source).unwrap();

    let mut operations: LinkedList<FSOperation> = LinkedList::new();
    for src_dir_entry in source_paths {
        let path = src_dir_entry.unwrap().path();

        let target_file_path = target.join(path.as_path().file_name().expect("Unable to get path filename"));

        //handle
        let result = stow_path(path.as_path(), target_file_path.as_path(), force, backup, operations.borrow_mut());

        match result {
            Ok(TraversOperation::StopPathRun) => (),
            Ok(TraversOperation::Continue) =>
                if path.as_path().is_dir() {
                    visit_sync(path.as_path(), target_file_path.as_path(), dryrun, force, backup);
                },
            Err(e) => error!("{}", e)
        }
    }

    if dryrun {
        dryrun_interpreter(operations.borrow());
    } else {
        fs_effect_interpreter(operations.borrow());
    }
    Ok(())
}

fn stow_path<'a>(source_path: &'a Path, target_path: &'a Path, force: bool, backup: bool, operations: &'a mut LinkedList<FSOperation>) -> io::Result<TraversOperation> {
    let target_is_directory = source_path.is_dir();
    let target_exist = target_path.exists();
    let target_is_symlink = is_symlink(target_path);
    let valid_symlink = check_symlink(target_path, source_path);

    let stop_if_directory = || -> io::Result<TraversOperation> {
        if target_is_directory {
            Ok(TraversOperation::StopPathRun)
        } else {
            Ok(TraversOperation::Continue)
        }
    };

    match (target_exist, target_is_symlink, target_is_directory, force) {
        (true, true, _, _) => {
            //A symbolic link already exist
            if valid_symlink {
                //ignore target exist if it's already the good symlink
                info!("Valid symlink {} already exist, nothing to do", target_path.display());
                Ok(TraversOperation::StopPathRun)
            } else {
                warn!("Path symlink {} already exist and will be override", target_path.display());
                operations.push_back(FSOperation::CreateSymlink {source: source_path.to_owned(), target: target_path.to_owned()});
                stop_if_directory()
            }
        }
        (true, false, false, true) => {
            // A real file already exist and force flag is set
            if backup {
                operations.push_back(FSOperation::Backup(target_path.to_owned()));
            } else {
                warn!("Path {} already exist and will be override !", target_path.display());
                operations.push_back(FSOperation::Delete(target_path.to_owned()));
            }
            operations.push_back(FSOperation::CreateSymlink {source: source_path.to_owned(), target: target_path.to_owned()});
            Ok(TraversOperation::Continue)
        },
        (true, false, false, false) => {
            // A real file already exist but force flag is not set => ERROR
            return Err(Error::new(ErrorKind::Other, format!("Path {} already exist. Set -f flag to force override", target_path.display())));
        },
        (true, false, true, _) => {
            //break for existing directory
            return Ok(TraversOperation::Continue);
        },
        (false, _, _, _) => {
            operations.push_back(FSOperation::CreateSymlink {source: source_path.to_owned(), target: target_path.to_owned()});
            stop_if_directory()
        }
    }

}

fn dryrun_interpreter(operations: &LinkedList<FSOperation>) -> io::Result<()> {
    for op in operations {
        match op {
            FSOperation::Backup(p) => info!("DRY-RUN : backup {}", p.display()),
            FSOperation::Delete(p) => {
                if p.is_dir() {
                    info!("DRY-RUN : delete directory recursively {}", p.display());
                } else {
                    info!("DRY-RUN : delete file {}", p.display());
                }
            }
            FSOperation::CreateSymlink{source, target} => info!("DRY-RUN : create symbolic link {} -> {}", source.display(), target.display())
        };
    };
    Ok(())
}

fn fs_effect_interpreter(operations: &LinkedList<FSOperation>) -> io::Result<()> {
    for op in operations {
        match op {
            FSOperation::Backup(p) => backup_path(p.as_path()),
            FSOperation::Delete(p) => delete_path(p.as_path()),
            FSOperation::CreateSymlink{source, target} => create_symlink(source.as_path(), target.as_path())
        };
    };
    Ok(())
}

fn create_symlink(source_path: &Path, target_path: &Path) -> io::Result<()> {
    if cfg!(target_family = "unix") {
        info!("create symbolic link {} -> {}", source_path.display(), target_path.display());
        symlink(source_path, target_path)
    } else {
        Err(Error::new(ErrorKind::Other, "OS not supported"))
    }
}

fn backup_path(target_path: &Path) -> io::Result<()> {
    let file_name = target_path.file_name()
        .and_then(|x: &OsStr| x.to_str())
        .expect("Unable to get filename");

    let parent_path = target_path.parent().expect("Unable to get parent directory");
    let backup_path = parent_path.join("backup-".to_owned()+file_name);

    info!("backup {} into {}", target_path.display(), backup_path.as_path().display());
    fs::rename(target_path, backup_path.as_path())
}

fn delete_path(path: &Path) -> io::Result<()> {
    if path.is_dir() {
        info!("delete directory recursively {}", path.display());
        fs::remove_dir_all(path)
    } else {
        info!("delete file {}", path.display());
        fs::remove_file(path)
    }
}

fn is_symlink(path: &Path) -> bool {
    match  path.symlink_metadata() {
        Ok(data) => data.file_type().is_symlink(),
        Err(_e) => false
    }
}

fn check_symlink(symlink_path: &Path, valid_dest: &Path) -> bool {
    match fs::read_link(symlink_path) {
        Ok(real) => valid_dest.eq(real.as_path()),
        Err(_e) => false
    }
}
