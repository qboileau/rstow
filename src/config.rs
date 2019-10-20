
use quicli::prelude::*;

use serde_derive::Deserialize;
use toml;
use std::io;
use std::fs;

use std::path::Path;
use toml::value::*;
use std::error::Error;

#[derive(Deserialize)]
pub(crate) struct RstowConfig {
    symlink_current_dir: bool,
    ignore_file: Array,
}

const RSTOW_FILE_NAME: &str = ".rstow";

pub(crate) fn read_config_file(directory: &Path) -> Option<RstowConfig> {
    let config_file = directory.join(RSTOW_FILE_NAME);

    let content = fs::read_to_string(config_file).unwrap_or("".to_string());
    match toml::from_str(content.as_str()) {
        Ok(conf) => Some(conf),
        Err(error) => {
            debug!("Unable to read rstow configuration file in {} : {}", directory.display(), error.description());
            None
        }
    }
}


#[cfg(test)]
mod test_config {
    use super::*;
    use test_utils::*;
    use std::path::PathBuf;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn test_config_file() {
        with_test_directories(&"test_config_file",|source: &PathBuf, target: &PathBuf| {
            let mut config_file = File::create(source.as_path().join(RSTOW_FILE_NAME)).unwrap();
            let content = r#"
    symlink_current_dir = true
    ignore_file = [ "secret-file.txt" ]
            "#;

            config_file.write_all(content.as_bytes()).unwrap();

            let config_opt: Option<RstowConfig> = read_config_file(source.as_path());
            assert!(config_opt.is_some());
            let config = config_opt.unwrap();
            assert_eq!(config.symlink_current_dir, true);
            let mut ignores = config.ignore_file.into_iter();
            assert_eq!(ignores.next().unwrap().as_str(), Some("secret-file.txt"));
        });
    }

    #[test]
    fn test_no_config_file() {
        with_test_directories(&"test_no_config_file",|source: &PathBuf, target: &PathBuf| {
            let config_opt: Option<RstowConfig> = read_config_file(source.as_path());
            assert!(config_opt.is_none());
        });
    }
}
