# rstow
Custom stow implementation in Rust

## Usage
```
rstow
Like stow but simpler and with more crabs

USAGE:
    rstow [FLAGS] [OPTIONS] --target <target>

FLAGS:
    -b, --backup       Create a backup of the file before override it with a symlink
    -d, --dryrun       Dry run rstow (this will do not affect files and logs what should be done)
    -f, --force        Force override files on target using a symlink
    -h, --help         Prints help information
    -u, --unstow       Un-stow a target path from source (will remove symlinks and rename re-use backup files if exist)
    -V, --version      Prints version information
    -v, --verbosity    Pass many times for more log output
                       
                       By default, it'll only report errors. Passing `-v` one time also prints warnings, `-vv` enables
                       info logging, `-vvv` debug, and `-vvvv` trace.

OPTIONS:
    -s, --source <source>    Source directory [default: ./]
    -t, --target <target>    Target directory
```
