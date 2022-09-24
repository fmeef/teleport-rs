use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use crate::schema::Spec;
use anyhow::Result;

mod methods;
#[allow(dead_code)]
pub(crate) mod naming;
pub(crate) mod schema;
mod types;
pub(crate) mod util;

/// Type for mapping type names for multitype enums
pub(crate) type MultiTypes = Arc<RwLock<HashMap<String, String>>>;

/// Generator for both types and methods
pub struct Generate {
    spec: Arc<Spec>,
    multitypes: Arc<RwLock<HashMap<String, String>>>,
}

pub(crate) static MULTITYPE_ENUM_PREFIX: &str = "E";
pub(crate) static ARRAY_OF: &str = "Array of ";
pub(crate) static INPUT_FILE: &str = "InputFile";
pub(crate) static UPDATE: &str = "Update";

impl Generate {
    pub fn new<T: AsRef<str>>(json: T) -> Result<Generate> {
        Ok(Generate {
            spec: Arc::new(serde_json::from_str(json.as_ref())?),
            multitypes: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub fn generate_types(&self) -> Result<String> {
        let generate_types =
            types::GenerateTypes::new(Arc::clone(&self.spec), Arc::clone(&self.multitypes));
        generate_types.generate_types()
    }

    pub fn generate_methods(&self) -> Result<String> {
        let generate_methods = Arc::new(methods::GenerateMethods::new(
            Arc::clone(&self.spec),
            Arc::clone(&self.multitypes),
        ));
        generate_methods.generate_methods()
    }
}
