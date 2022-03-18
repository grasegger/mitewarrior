use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::tw_entry::TWEntry;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Task {
    pub uuid: String,
    pub description: String,
    #[serde(default = "default_project")]
    pub project: String,
    pub tags: Option<Vec<String>>,
    pub trellourl: Option<String>,
    pub githuburl: Option<String>,
}

fn default_project() -> String {
    "---".to_string()
}

impl Task {
    pub fn from_twentry(entry: &TWEntry) -> Self {
        let command = Command::new("task")
            .arg(format!("{}", entry.tags.first().unwrap()))
            .arg("export")
            .output()
            .expect("Unable to export from taskwarrior.")
            .stdout;

        let json = String::from_utf8_lossy(&command);

        let input: Vec<Self> = serde_json::from_str(&json).unwrap();
        input.first().unwrap().clone()
    }
}
