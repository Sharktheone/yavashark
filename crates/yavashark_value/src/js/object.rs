use super::Value;
use std::collections::HashMap;
use std::fmt::{Debug, Display};

#[derive(Debug, PartialEq)]
pub struct Object<F: Debug> {
    pub properties: HashMap<String, Value<F>>,
    pub call: Option<F>,
    pub construct: Option<F>,
}

impl<F: Debug> Object<F> {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            call: None,
            construct: None,
        }
    }

    pub fn define_property(&mut self, name: String, value: Value<F>) {
        self.properties.insert(name, value);
    }

    pub fn get_property(&self, name: &str) -> Option<&Value<F>> {
        self.properties.get(name)
    }
}

impl Default for Object<()> {
    fn default() -> Self {
        Self::new()
    }
}
