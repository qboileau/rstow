#![allow(unused)]

#[macro_use]
extern crate quicli;
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

mod fileutils;

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
    /// Dry run rstow (this will do not affect files and logs what should be done)
    #[structopt(long = "dryrun", short = "d")]
    dryrun: bool,
    /// Un-stow a target path from source (will remove symlinks and rename re-use backup files if exist)
    #[structopt(long = "unstow", short = "u")]
    unstow: bool,
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
    Restore { backup: PathBuf, target: PathBuf },
    CreateSymlink { source: PathBuf, target: PathBuf },
    Delete(PathBuf)
}

main!(|args: Cli, log_level: verbosity| {
    let dryrun = &args.dryrun;
    let force = &args.force;
    let backup = &args.backup;
    let unstow = &args.unstow;

    let source = fs::canonicalize(&args.source).expect("Unresolved absolute source path");
    let target = fs::canonicalize(&args.target).expect("Unresolved absolute target path");

    info!("Stow from Source {:?} to target {:?}", source.display(), target.display());
    visit_sync(source.as_path(), target.as_path(), *dryrun, *force, *backup, *unstow).expect("An error occurred when visiting directories") ;

});

fn visit_sync(source: &Path, target: &Path, dryrun: bool, force: bool, backup: bool, unstow: bool)-> io::Result<()> {
    let source_paths = fs::read_dir(source).unwrap();

    let mut operations: LinkedList<FSOperation> = LinkedList::new();
    for src_dir_entry in source_paths {
        let path = src_dir_entry.unwrap().path();

        let target_file_path = target.join(path.as_path().file_name().expect("Unable to get path filename"));

        let result = {
            if unstow {
                unstow_path(path.as_path(), target_file_path.as_path(), operations.borrow_mut())
            } else {
                //handle
                stow_path(path.as_path(), target_file_path.as_path(), force, backup, operations.borrow_mut())
            }
        };

        match result {
            Ok(TraversOperation::StopPathRun) => (),
            Ok(TraversOperation::Continue) =>
                if path.as_path().is_dir() {
                    visit_sync(path.as_path(), target_file_path.as_path(), dryrun, force, backup, unstow);
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
    let target_is_symlink = fileutils::is_symlink(target_path);
    let is_valid_symlink = fileutils::check_symlink(target_path, source_path);

    let stop_if_directory = || -> io::Result<TraversOperation> {
        if target_is_directory {
            Ok(TraversOperation::StopPathRun)
        } else {
            Ok(TraversOperation::Continue)
        }
    };

    let symlink_operation = FSOperation::CreateSymlink { source: source_path.to_owned(), target: target_path.to_owned() };

    match (target_exist, target_is_symlink, target_is_directory, force) {
        (true, true, _, _) => {
            //A symbolic link already exist
            if is_valid_symlink {
                //ignore target exist if it's already the good symlink
                info!("Valid symlink {} already exist, nothing to do", target_path.display());
                Ok(TraversOperation::StopPathRun)
            } else {
                warn!("Path symlink {} already exist and will be override", target_path.display());
                operations.push_back(symlink_operation);
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
            operations.push_back(symlink_operation);
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
            operations.push_back(symlink_operation);
            stop_if_directory()
        }
    }
}

fn unstow_path<'a>(source_path: &'a Path, target_path: &'a Path, operations: &'a mut LinkedList<FSOperation>) -> io::Result<TraversOperation> {
    let target_is_directory = source_path.is_dir();
    let target_exist = target_path.exists();
    let target_is_symlink = fileutils::is_symlink(target_path);
    let is_valid_symlink = fileutils::check_symlink(target_path, source_path);
    let backup_path = fileutils::build_backup_path(target_path)?;
    let backup_exist = backup_path.exists();

    if !target_exist || !target_is_symlink || !is_valid_symlink {
        // do nothing if target doesn't exist, is a real file or is not a symlink valid
        return Ok(TraversOperation::Continue);
    }

    //remove symlink
    operations.push_back(FSOperation::Delete(target_path.to_owned()));

    //restore backup if exist
    if backup_exist {
        operations.push_back(FSOperation::Restore { backup: backup_path.to_owned(), target: target_path.to_owned()});
    }
    Ok(TraversOperation::Continue)
}

fn dryrun_interpreter(operations: &LinkedList<FSOperation>) -> io::Result<()> {
    for op in operations {
        match op {
            FSOperation::Backup(p) => info!("DRY-RUN : backup {}", p.display()),
            FSOperation::Restore {backup, target} => info!("DRY-RUN : restore {} -> {}", backup.display(), target.display()),
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
            FSOperation::Backup(p) => fileutils::backup_path(p.as_path()),
            FSOperation::Delete(p) => fileutils::delete_path(p.as_path()),
            FSOperation::Restore {backup, target} => fileutils::restore_path(backup.as_path(), target.as_path()),
            FSOperation::CreateSymlink{source, target} => fileutils::create_symlink(source.as_path(), target.as_path())
        };
    };
    Ok(())
}
