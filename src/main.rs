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

mod stow;
mod unstow;
mod interpreters;
mod fileutils;
mod operations;

use fileutils::*;
use operations::FSOperation;
use operations::TraversOperation;

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
                unstow::unstow_path(path.as_path(), target_file_path.as_path(), operations.borrow_mut())
            } else {
                //handle
                stow::stow_path(path.as_path(), target_file_path.as_path(), force, backup, operations.borrow_mut())
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
        interpreters::dryrun_interpreter(operations.borrow());
    } else {
        interpreters::filesystem_interpreter(operations.borrow());
    }
    Ok(())
}
