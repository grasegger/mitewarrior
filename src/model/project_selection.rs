#[derive(Debug, Clone)]
pub struct ProjectSelection {
    pub name: String,
    pub id: i64,
    pub customer_id: Option<i64>,
}
