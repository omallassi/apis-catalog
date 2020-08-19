extern crate serde_json;
extern crate uuid;
use schema::domain;
use serde_json;
use uuid::Uuid;

#[derive(Queryable)]
pub struct DomainItem {
    pub name: std::string::String,
    pub id: uuid::Uuid,
    pub description: String,
}
