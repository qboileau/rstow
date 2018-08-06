#[macro_use] extern crate quicli;
use quicli::prelude::*;

use std::io;
use std::fs::{self};
use std::path::Path;
//use std::os::unix::fs::symlink as unixsymlink;


// Add cool slogan for your app here, e.g.:
/// Get first n lines of a file
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
    let target = Path::new(&args.target);

    info!("Stow from Source {:?} to target {:?}", source.display(), target.display());
    visit_sync(source.as_path(), target, *dryrun, *force, *backup).expect("An error occurred when visiting directories") ;

});

fn visit_sync(source: &Path, target: &Path, dryrun: bool, force: bool, backup: bool)-> io::Result<()> {
    let source_paths = fs::read_dir(source).unwrap();
    for src_dir_entry in source_paths {
        let path = src_dir_entry.unwrap().path();

        let target_file_path = target.join(path.as_path().file_name().expect("Unable to get path filename"));

        //handle
        let result = handler(path.as_path(), target_file_path.as_path(), dryrun, force, backup);

        match result {
            Ok(RunOperation::StopPathRun) => (),
            Ok(RunOperation::Continue) =>
                if path.as_path().is_dir() {
                    visit_sync(path.as_path(), target_file_path.as_path(), dryrun, force, backup);
                },
            Err(_e) => ()
        }


    }
    Ok(())
}

fn is_symlink(path: &Path) -> bool {
    match  path.symlink_metadata() {
        Ok(data) =>  data.file_type().is_symlink(),
        Err(_e) => false
    }
}

fn handler(source_path: &Path, target_path: &Path, dryrun: bool, force: bool, backup: bool) -> io::Result<RunOperation> {
    if source_path.is_dir() {
        return directory_handler(source_path, target_path, force, backup);
    } else {
        return file_handler(source_path, target_path, dryrun, force, backup);
    }
}

fn directory_handler(source_file: &Path, target_path: &Path, force: bool, backup: bool) -> io::Result<RunOperation> {
    let path_is_simlink = is_symlink(target_path);
    if !target_path.exists() {
        info!("create symbolic link directory from {} -> {}", source_file.display(), target_path.display());
        Ok(RunOperation::StopPathRun)
    } else {
        info!("target directory {} already exist", target_path.display());
        Ok(RunOperation::Continue)
    }
}

fn file_handler(source_file: &Path, target_path: &Path, dryrun: bool, force: bool, backup: bool) -> io::Result<RunOperation> {
    let target_exist = target_path.exists();
    let target_is_simlink = is_symlink(target_path);

    if target_exist {
        if target_is_simlink {
            warn!("File {} already exist and is a symlink", target_path.display());
        } else {
            warn!("File {} already exist and is not a symlink", target_path.display());
        }
    } else {
        info!("create symbolic link {} -> {}", source_file.display(), target_path.display());
    }
    Ok(RunOperation::Continue)
}