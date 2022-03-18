use serde::{Deserialize, Serialize};

use super::service::Service;

pub type Customer = Service;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CustomerObject {
    pub customer: Customer,
}
