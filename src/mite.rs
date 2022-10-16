use std::vec;

use serde::Serialize;
use ureq::Response;

use crate::model::project_selection::ProjectSelection;
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
    pub project_selection: Vec<ProjectSelection>,
}

#[derive(Serialize)]
pub struct TimeEntry {
    time_entry: Answer,
}

impl Mite {
    fn api_get(&self, url: &str) -> Result<Response, ureq::Error> {
        ureq::get(&format!("https://{}.mite.yo.lk{}", self.instance, url))
            .set("X-MiteApiKey", &self.api_key)
            .set(
                "User-Agent",
                concat!(
                    env!("CARGO_PKG_NAME"),
                    "/",
                    env!("CARGO_PKG_VERSION"),
                    " - ",
                    env!("CARGO_PKG_AUTHORS")
                ),
            )
            .call()
    }

    pub fn new(api_key: String, instance: String) -> Option<Self> {
        let mut mite = Self {
            api_key,
            instance,
            services: vec![],
            customers: vec![],
            projects: vec![],
            project_selection: vec![],
        };

        let answer = mite.api_get("/myself.json");
        match answer {
            Ok(answer) => {
                if answer.status() == 200 {
                    mite.load_services();
                    mite.load_projects();
                    mite.load_customers();
                    mite.create_project_selection();
                    Some(mite)
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    }

    pub fn create_time_entry(&self, time_entry: Answer) {
        _ = ureq::post(&format!(
            "https://{}.mite.yo.lk/time_entries.json",
            self.instance
        ))
        .set("X-MiteApiKey", &self.api_key)
        .set(
            "User-Agent",
            concat!(
                env!("CARGO_PKG_NAME"),
                "/",
                env!("CARGO_PKG_VERSION"),
                " - ",
                env!("CARGO_PKG_AUTHORS")
            ),
        )
        .send_json(TimeEntry { time_entry });
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

    fn create_project_selection(&mut self) {
        for project in &self.projects {
            let name = if let Some(customer_id) = project.project.customer_id {
                let mut out = format!("{}", project.project.name);

                for customer in &self.customers {
                    if customer.customer.id == customer_id {
                        out = format!("{} - {}", out, customer.customer.name)
                    }
                }
                out
            } else {
                format!("{}", project.project.name)
            };
            self.project_selection.push(ProjectSelection {
                customer_id: project.project.customer_id,
                id: project.project.id,
                name,
            })
        }
    }
}
