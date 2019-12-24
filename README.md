[![Build Status](https://travis-ci.org/qboileau/rstow.svg?branch=master)](https://travis-ci.org/qboileau/rstow)

# rstow
Custom stow implementation in Rust

## Build / install
Need cargo and rust installed

#### Manually
`cargo build --release` binary will be located in `./target/release/rstow`

#### Arch User Repository
On Arch Linux :
```bash
cd ./pkg

#build
makepkg

#install
makepkg --install
``` 

#### Curl
`curl -sLO https://github.com/qboileau/rstow/releases/download/1.0/rstow && chmod +x ./rstow && ./rstow -h`

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

### Example
Stow from `./dotfiles/home` folder to actual user home folder
```sh
rstow --force --backup --source ./dotfiles/home --target $HOME -vv
```
