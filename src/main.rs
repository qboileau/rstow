#![allow(unused)]

#[macro_use] extern crate quicli;
use quicli::prelude::*;

use std::io;
use std::io::{Error, ErrorKind};
use std::fs::{self};
use std::ffi::OsStr;
use std::path::Path;
use std::os::unix::fs::symlink;

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

enum RunOperation {
    Continue,
    StopPathRun
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
    for src_dir_entry in source_paths {
        let path = src_dir_entry.unwrap().path();

        let target_file_path = target.join(path.as_path().file_name().expect("Unable to get path filename"));

        //handle
        let result = handler2(path.as_path(), target_file_path.as_path(), dryrun, force, backup);

        match result {
            Ok(RunOperation::StopPathRun) => (),
            Ok(RunOperation::Continue) =>
                if path.as_path().is_dir() {
                    visit_sync(path.as_path(), target_file_path.as_path(), dryrun, force, backup);
                },
            Err(e) => error!("{}", e)
        }
    }
    Ok(())
}

fn handler2(source_path: &Path, target_path: &Path, dryrun: bool, force: bool, backup: bool) -> io::Result<RunOperation> {
    let is_directory = source_path.is_dir();
    let target_exist = target_path.exists();
    let target_is_symlink = is_symlink(target_path);
    let valid_symlink = check_symlink(target_path, source_path);

    match (target_exist, target_is_symlink, force) {
        (true, true, _) => {
            if valid_symlink {
                () //ignore target exist if it's already the good symlink
            } else {
                warn!("Path symlink {} already exist and will be override", target_path.display());
            }
        }
        (true, false, true) => {
            if backup {
                backup_path(target_path, dryrun);
            } else {
                warn!("Path {} already exist and will be override !", target_path.display());
                delete_path(target_path, dryrun);
            }
        },
        (true, false, false) => {
            return Err(Error::new(ErrorKind::Other, format!("Path {} already exist. Set -f flag to force override", target_path.display())));
        },
        (false, _, _) => ()
    }

    create_symlink(source_path, target_path, dryrun);
    if is_directory {
        Ok(RunOperation::StopPathRun)
    } else {
        Ok(RunOperation::Continue)
    }
}


fn handler(source_path: &Path, target_path: &Path, dryrun: bool, force: bool, backup: bool) -> io::Result<RunOperation> {
    if source_path.is_dir() {
        directory_handler(source_path, target_path, dryrun, force, backup)
    } else {
        file_handler(source_path, target_path, dryrun, force, backup)
    }
}

fn directory_handler(source_path: &Path, target_path: &Path, dryrun: bool, force: bool, backup: bool) -> io::Result<RunOperation> {
    let path_is_symlink = is_symlink(target_path);
    if target_path.exists() {
        if path_is_symlink {
            warn!("target directory {} already exist as simlink", target_path.display());
            //TODO check if symlink is valid or override it if force flag is true (ignore backup flag)
            let valid_symlink = check_symlink(target_path, source_path);
            if valid_symlink {
                info!("valid symlink destination for {}", target_path.display());
            } else {
                warn!("must override symlink {}", target_path.display());
            }
            Ok(RunOperation::StopPathRun)
        } else {
            warn!("target directory {} already exist", target_path.display());
            if force {
                if backup {
                    backup_path(target_path, dryrun);
                }
                create_symlink(source_path, target_path, dryrun);
                Ok(RunOperation::StopPathRun)
            } else {
                Ok(RunOperation::Continue)
            }
        }
    } else {
        create_symlink(source_path, target_path, dryrun);
        Ok(RunOperation::StopPathRun)
    }
}

fn file_handler(source_path: &Path, target_path: &Path, dryrun: bool, force: bool, backup: bool) -> io::Result<RunOperation> {
    let target_exist = target_path.exists();
    let target_is_symlink = is_symlink(target_path);

    if target_exist {
        if target_is_symlink {
            warn!("File {} already exist and is a symlink", target_path.display());
            let valid_symlink = check_symlink(target_path, source_path);
            if valid_symlink {
                info!("valid symlink destination for {}", target_path.display());
            } else {
                warn!("must override symlink {}", target_path.display());
            }
            //TODO check if symlink is valid or override it if force flag is true (ignore backup flag)
        } else {
            warn!("File {} already exist and is not a symlink", target_path.display());
            if force {
                if backup {
                    backup_path(target_path, dryrun);
                }
                create_symlink(source_path, target_path, dryrun);
            }
        }
    } else {
        create_symlink(source_path, target_path, dryrun);
    }
    Ok(RunOperation::Continue)
}

fn create_symlink(source_path: &Path, target_path: &Path, dryrun: bool) -> io::Result<()> {
    if dryrun {
        info!("DRY-RUN : create symbolic link {} -> {}", source_path.display(), target_path.display());
        Ok(())
    } else {
        if cfg!(target_family = "unix") {
            info!("create symbolic link {} -> {}", source_path.display(), target_path.display());
            symlink(source_path, target_path)
        } else {
            Err(Error::new(ErrorKind::Other, "OS not supported"))
        }
    }
}

fn backup_path(target_path: &Path, dryrun: bool) -> io::Result<()> {
    let file_name = target_path.file_name()
        .and_then(|x: &OsStr| x.to_str())
        .expect("Unable to get filename");

    let parent_path = target_path.parent().expect("Unable to get parent directory");
    let backup_path = parent_path.join("backup-".to_owned()+file_name);

    if dryrun {
        info!("DRY-RUN : backup {} into {}", target_path.display(), backup_path.as_path().display());
        Ok(())
    } else {
        info!("backup {} into {}", target_path.display(), backup_path.as_path().display());
        fs::rename(target_path, backup_path.as_path())
    }
}

fn delete_path(path: &Path, dryrun: bool) -> io::Result<()> {
    if dryrun {
        if path.is_dir() {
            info!("DRY-RUN : delete directory recursively {}", path.display());
        } else {
            info!("DRY-RUN : delete file {}", path.display());
        }
        Ok(())
    } else {
        if path.is_dir() {
            info!("delete directory recursively {}", path.display());
            fs::remove_dir_all(path)
        } else {
            info!("delete file {}", path.display());
            fs::remove_file(path)
        }
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
