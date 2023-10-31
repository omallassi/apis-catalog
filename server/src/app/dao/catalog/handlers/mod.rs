use std::fmt::Debug;

pub mod implem;

pub trait SpecHandler: Send + Sync + Debug + Clone{
    fn get_version(&self) -> String;

    fn get_title(&self) -> String;

    fn get_description(&self) -> String;

    fn get_paths_len(&self) -> usize;

    fn get_paths(&self) -> Vec<Path>;
}

#[derive(Debug, Clone)]
pub enum SpecType {
    OpenApi, 
    AsyncApi, 
}

#[derive(Debug, Clone)]
pub struct Path {
    pub path: String, 
    pub methods: Vec<Method>
}

#[derive(Debug, Clone)]
pub struct Method {
    pub method: String, 
    pub description: String, 
    pub summary: String
}