use std::{
    collections::HashMap,
    io::{self, Stdin},
};

use regex::Regex;
use serde::Serialize;

use crate::tw_entry::TWEntry;

#[derive(Debug, Serialize, Clone)]
pub struct TimewInput {
    pub settings: HashMap<String, String>,
    pub data: Vec<TWEntry>,
}

impl TimewInput {
    pub(crate) fn read_from_stdin() -> Self {
        let stdin = io::stdin();
        let settings = Self::read_settings(&stdin);

        let mut json_string = String::new();

        loop {
            if stdin.read_line(&mut json_string).unwrap() == 0 {
                break;
            }
        }

        TimewInput {
            settings,
            data: serde_yaml::from_str(&json_string).unwrap(),
        }
    }

    fn read_settings(stdin: &Stdin) -> HashMap<String, String> {
        let re = Regex::new(r"^(?m) *$").unwrap();
        let mut settings = HashMap::new();

        let mut line = String::new();

        stdin.read_line(&mut line).unwrap();

        while !re.is_match(&line) {
            let mut setting = line.split(':');
            let key = setting.nth(0).unwrap();
            let value = setting.nth(0).unwrap_or("").trim();
            settings.insert(key.to_string(), value.to_string());
            line = String::new();
            stdin.read_line(&mut line).unwrap();
        }

        return settings;
    }
}
