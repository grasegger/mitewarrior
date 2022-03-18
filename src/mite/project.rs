use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Project {
    pub id: i64,
    pub name: String,
    pub customer_id: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectObject {
    pub project: Project,
}
