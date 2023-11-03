use std::fmt::{Debug};

use strum_macros::{Display, EnumString};

pub mod implem;

pub trait SpecHandler: Sync + Send + SpecHandlerClone + Debug {
    fn get_version(&self) -> String;

    fn get_title(&self) -> String;

    fn get_description(&self) -> String;

    fn get_paths_len(&self) -> usize;

    fn get_paths(&self) -> Vec<Path>;

    fn get_audience(&self) -> String;

    fn get_api_id(&self) -> String;

    fn get_layer(&self) -> String;

    fn get_systems(&self) -> Vec<String>;

    fn get_domain(&self) -> String;
}

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
#[derive(EnumString, Display)]
pub enum SpecType {
    #[strum(serialize = "OpenAPI.v3")]
    OpenAPIv3, 
    #[strum(serialize = "AsyncAPI.v1")]
    AsyncAPIv1, 
    #[strum(serialize = "AsyncAPI.v2")]
    AsyncAPIv2,
    #[strum(serialize = "Proto3")]
    Proto3, 
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

// SpecItem struct will link to a handler of Box<dyn SpecHandler> *and* must be Clong
// *but* Trait SpecHandler cannot have Clone (because of Object Safety).
// Having this Trait kinda help. not fully understood tbh and deeply inspired by 
// https://stackoverflow.com/questions/30353462/how-to-clone-a-struct-storing-a-boxed-trait-object
pub trait SpecHandlerClone {
    fn clone_box(&self) -> Box<dyn SpecHandler>;
}

impl<T> SpecHandlerClone for T where T: 'static + SpecHandler + Clone, 
{
    fn clone_box(&self) -> Box<dyn SpecHandler> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn SpecHandler> {
    fn clone(&self) -> Box<dyn SpecHandler> {
        self.clone_box()
    }
}