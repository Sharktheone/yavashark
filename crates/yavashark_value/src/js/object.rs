use std::collections::HashMap;
use super::Value;


#[derive(Debug, Clone, PartialEq)]
pub struct Object<F> {
    pub properties: HashMap<String, Value<F>>,
    pub call: Option<F>,
    pub construct: Option<F>,
}


impl <F> Object<F> {
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
