use std::collections::HashMap;
use yavashark_value::Value;

pub struct Scope<'ctx> {
    parent: Option<&'ctx mut Scope<'ctx>>,
    variables: HashMap<String, Value>,
}



impl<'a> Scope<'a> {
    pub fn new() -> Self {
        Self {
            parent: None,
            variables: HashMap::new(),
        }
    }
    
    pub fn with_parent(parent: &'a mut Scope<'a>) -> Self {
        Self {
            parent: Some(parent),
            variables: HashMap::new(),
        }
    }
}