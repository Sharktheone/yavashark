use std::collections::HashMap;
use std::fmt::Debug;

use crate::Func;

use super::Value;

#[derive(Debug, PartialEq)]
pub struct Object<F: Func> {
    pub properties: HashMap<String, Value<F>>,
    pub call: Option<F>,
    pub construct: Option<F>,
}

impl<F: Func> Object<F> {
    #[allow(clippy::new_without_default)]
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

    pub fn get_property_mut(&mut self, name: &str) -> Option<&mut Value<F>> {
        self.properties.get_mut(name)
    }

    pub fn update_or_define_property(&mut self, name: String, value: Value<F>) {
        if let Some(prop) = self.properties.get_mut(&name) {
            *prop = value;
        } else {
            self.define_property(name, value);
        }
    }
}
