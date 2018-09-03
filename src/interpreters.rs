
use quicli::prelude::*;
use im::vector::*;

use std::io;
use std::path::{Path, PathBuf};
use std::collections::LinkedList;
use std::result::Result;

use fileutils::*;
use operations::FSOperation;
use errors::AppError;


pub(crate) fn dryrun_interpreter(operations: &Vector<Result<FSOperation, AppError>>) -> Result<(), AppError> {
    let mut has_error = false;
    for result in operations.iter() {
        match result {
            Ok(op) => {
                match op {
                    FSOperation::Nothing(p) => println!("DRY-RUN : nothing to do on {}", p.display()),
                    FSOperation::Backup(p) => println!("DRY-RUN : backup {}", p.display()),
                    FSOperation::Restore {backup, target} => println!("DRY-RUN : restore {} -> {}", backup.display(), target.display()),
                    FSOperation::Delete(p) => {
                        if p.is_dir() {
                            println!("DRY-RUN : delete directory recursively {}", p.display());
                        } else {
                            println!("DRY-RUN : delete file {}", p.display());
                        }
                    }
                    FSOperation::CreateSymlink{source, target} => println!("DRY-RUN : create symbolic link {} -> {}", source.display(), target.display()),
                };
            },
            Err(err) => {
                has_error = true;
                eprintln!("DRY-RUN : Error {}", err)
            }
        }
    };

    if has_error {
        error!("{}", AppError::ApplyError)
    }
    Ok(())
}

pub(crate) fn filesystem_interpreter(operations: &Vector<&FSOperation>) -> Result<(), AppError> {
    for op in operations.iter() {
        match op {
            FSOperation::Nothing(source) => {
                info!("Nothing to do on {}", source.display());
                Ok(())
            },
            FSOperation::Backup(p) => backup_path(p.as_path()),
            FSOperation::Delete(p) => delete_path(p.as_path()),
            FSOperation::Restore {backup, target} => restore_path(backup.as_path(), target.as_path()),
            FSOperation::CreateSymlink{source, target} => create_symlink(source.as_path(), target.as_path()),
        };
    };
    Ok(())
}