use serde::{Deserialize, Serialize};
use std::{fs, path::PathBuf, process::Command};
use xdg::BaseDirectories;

#[derive(Serialize, Deserialize, Debug)]
pub struct Config {
    pub mite_api_key_command: String,
    pub mite_api_instance: String,
}

impl Config {
    // todo read from lazy static
    pub fn get_api_key(&self) -> String {
        let mut splitted_command = self.mite_api_key_command.split(" ");
        let command = splitted_command.next().unwrap();
        let args: Vec<&str> = splitted_command.map(|x| x).collect();
        let output = Command::new(command)
            .args(args)
            .output()
            .expect("failed to execute process");
        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        result
    }

    fn get_config_file_location() -> Option<PathBuf> {
        let xdg_dirs = BaseDirectories::with_prefix(env!("CARGO_PKG_NAME")).unwrap();
        xdg_dirs.find_config_file("config.yaml")
    }

    pub fn load() -> Self {
        match Self::get_config_file_location() {
            Some(location) => serde_yaml::from_str(
                &fs::read_to_string(location).expect("Unable to open config file."),
            )
            .expect("Unable to parse config file as yaml."),
            None => serde_yaml::from_str(include_str!("../config.yaml")).unwrap(),
        }
    }
}
