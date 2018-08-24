#![allow(unused)]

#[macro_use] extern crate quicli;
#[macro_use] extern crate im;
extern crate failure;
#[macro_use] extern crate failure_derive;

use quicli::prelude::*;
use im::vector::*;
use failure::Error;
use failure::ResultExt;

use std::result::Result;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::collections::LinkedList;
use std::borrow::BorrowMut;
use std::borrow::Borrow;

mod stow;
mod unstow;
mod interpreters;
mod fileutils;
mod operations;
mod errors;

#[cfg(test)]
mod test_utils;

use fileutils::*;
use operations::*;
use errors::*;

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
   program(&args);
});

/// Program execution follow 3 steps :
/// 1- Extract and verify inputs (provided `source` and `target` became absolute paths)
/// 2- Traverse recursively (if directory) `source` and build paths relative to `target` to find what operation
/// should be done to stow or unstow `source` to `target`
/// 3- Apply operations using an interpreter (`dryrun` or `filesystem` depending of `dryrun` flag state)
fn program(args: &Cli) {
    let dryrun = &args.dryrun;
    let force = &args.force;
    let backup = &args.backup;
    let unstow = &args.unstow;

    let source = fs::canonicalize(&args.source).expect("Unresolved absolute source path");
    let target = fs::canonicalize(&args.target).expect("Unresolved absolute target path");

    info!("Stow from Source {:?} to target {:?}", source.display(), target.display());

    let mut operations: Vector<Result<FSOperation, AppError>> = Vector::new();
    traverse_fs(source.as_path(), target.as_path(), *force, *backup, *unstow, &mut operations).expect("An error occurred when traversing directories");
    apply(operations.borrow(), *dryrun).unwrap_or_else(|e| {
        error!("{}", e);
    })
}


fn traverse_fs<'a, 'b, 'c>(source: &'a Path, target: &'b Path, force: bool, backup: bool, unstow: bool, operations: &'c mut Vector<Result<FSOperation, AppError>>) -> Result<(), AppError> {

    if source.is_dir() {
        let source_paths = fs::read_dir(source).context("test")?;
        for src_dir_entry in source_paths {
            let path = src_dir_entry.unwrap().path();
            let target_file_path = target.join(path.as_path().file_name().expect("Unable to get path filename"));

            let travers_result = visit_node(path.as_path(), target_file_path.as_path(), force, backup, unstow, operations.borrow_mut());
            match travers_result {
                Ok(TraversOperation::StopPathRun) => (),
                Ok(TraversOperation::Continue) => {
                    if path.as_path().is_dir() {
                        traverse_fs(path.as_path(), target_file_path.as_path(), force, backup, unstow, operations).expect("trololo");
                    }
                },
                Err(e) => error!("{}", e),
            }
        }
    } else {
        visit_node(source, target, force, backup, unstow, operations.borrow_mut());
    }
    Ok(())
}

fn visit_node<'a, 'b, 'c>(source: &'a Path, target: &'b Path, force: bool, backup: bool, unstow: bool, operations: &'c mut Vector<Result<FSOperation, AppError>>) -> Result<TraversOperation, AppError> {

    let mut node_operations: Vector<FSOperation> = Vector::new();
    let travers_result = {
        if unstow {
            unstow::unstow_path(source, target, node_operations.borrow_mut())
        } else {
            stow::stow_path(source, target, force, backup, node_operations.borrow_mut())
        }
    };

    match travers_result {
        Ok(travers_op) => {
            for op in node_operations {
                operations.push_back(Ok(op));
            }
            Ok(travers_op)
        },
        Err(e) => {
            operations.push_back(Err(e));
            Ok(TraversOperation::Continue)
        },
    }
}

fn apply<'a>(operations: &'a Vector<Result<FSOperation, AppError>>, dryrun: bool) -> Result<(), AppError> {
    if dryrun {
        interpreters::dryrun_interpreter(operations.borrow())
    } else {

        let mut operations_valid: Vector<&FSOperation> = Vector::new();
        let mut operations_error: Vector<&AppError> = Vector::new();

        for res_op in operations.iter() {
            match res_op {
                Ok(op) => operations_valid.push_back(op),
                Err(err) => operations_error.push_back(err)
            }
        };

        if !operations_error.is_empty() {
            for err in operations_error.iter() {
                error!("{}", err)
            }
            Err(AppError::ApplyError)
        } else {
            interpreters::filesystem_interpreter(operations_valid.borrow())
        }
    }
}