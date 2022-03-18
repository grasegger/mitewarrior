use std::vec;

use serde::Serialize;
use ureq::Response;

use crate::ui::Answer;

use self::customer::CustomerObject;
use self::project::ProjectObject;
use self::service::ServiceObject;

pub mod customer;
pub mod project;
pub mod service;

#[derive(Debug, Clone)]
pub struct Mite {
    api_key: String,
    instance: String,
    pub services: Vec<ServiceObject>,
    pub customers: Vec<CustomerObject>,
    pub projects: Vec<ProjectObject>,
}

#[derive(Serialize)]
pub struct TimeEntry {
    time_entry: Answer
}

impl Mite {
    fn api_get(&self, url: &str) -> Result<Response, ureq::Error> {
        ureq::get(&format!("https://{}.mite.yo.lk{}", self.instance, url))
            .set("X-MiteApiKey", &self.api_key)
            .set("User-Agent", concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION") , " - ", env!("CARGO_PKG_AUTHORS")))
            .call()
    }

    pub fn new(api_key: String, instance: String) -> Option<Self> {
        let mut mite = Self {
            api_key,
            instance,
            services: vec![],
            customers: vec![],
            projects: vec![],
        };

        let answer = mite.api_get("/myself.json");
        match answer {
            Ok(answer) => {
                if answer.status() == 200 {
                    mite.load_services();
                    mite.load_projects();
                    mite.load_customers();
                    Some(mite)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    pub fn create_time_entry(&self, time_entry: Answer) {
        _ = ureq::post(&format!("https://{}.mite.yo.lk/time_entries.json", self.instance))
        .set("X-MiteApiKey", &self.api_key)
        .set("User-Agent", concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION") , " - ", env!("CARGO_PKG_AUTHORS")))
        .send_json( TimeEntry {
            time_entry
        });
    }

    fn load_services(&mut self) {
        let answer = self
            .api_get("/services.json")
            .unwrap()
            .into_string()
            .unwrap();
        self.services = serde_yaml::from_str(&answer).unwrap();
    }

    fn load_customers(&mut self) {
        let answer = self
            .api_get("/customers.json")
            .unwrap()
            .into_string()
            .unwrap();
        self.customers = serde_yaml::from_str(&answer).unwrap();
    }

    fn load_projects(&mut self) {
        let answer = self
            .api_get("/projects.json")
            .unwrap()
            .into_string()
            .unwrap();
        self.projects = serde_yaml::from_str(&answer).unwrap();
    }

    pub fn find_client_name(&self, id: i64) -> String {
        let filtered: Vec<CustomerObject> = self
            .customers
            .clone()
            .into_iter()
            .filter(|x| x.customer.id == id)
            .collect();
        match filtered.first() {
            Some(data) => data.customer.name.clone(),
            None => "Client name not found".to_string(),
        }
    }

    pub(crate) fn find_project_name(&self, id: i64) -> String {
        let filtered: Vec<ProjectObject> = self
            .projects
            .clone()
            .into_iter()
            .filter(|x| x.project.id == id)
            .collect();
        match filtered.first() {
            Some(data) => data.project.name.clone(),
            None => "Client name not found".to_string(),
        }
    }

    pub fn find_service_name(&self, id: i64) -> String {
        let filtered: Vec<ServiceObject> = self
            .services
            .clone()
            .into_iter()
            .filter(|x| x.service.id == id)
            .collect();
        match filtered.first() {
            Some(data) => data.service.name.clone(),
            None => "Service name not found".to_string(),
        }
    }
}
