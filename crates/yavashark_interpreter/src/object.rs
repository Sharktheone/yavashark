mod prototype;
pub use prototype::*;
use std::collections::HashMap;
use std::fmt::Debug;

use yavashark_value::Obj;

use crate::context::Context;
use crate::Value;

#[derive(Debug)]
pub struct Object {
    properties: HashMap<Value, Value>,
}

impl Object {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> crate::Object {
        let this: Box<dyn Obj<Context>> = Box::new(Self {
            properties: HashMap::new(),
        });

        this.into()
    }
}

impl Obj<Context> for Object {
    fn define_property(&mut self, name: Value, value: Value) {
        self.properties.insert(name, value);
    }
    
    fn resolve_property(&self, name: &Value) -> Option<Value> {
        self.properties.get(&name).map(|v| v.copy())
    }

    fn get_property(&self, name: &Value) -> Option<&Value> {
        self.properties.get(name)
    }

    fn get_property_mut(&mut self, name: &Value) -> Option<&mut Value> {
        self.properties.get_mut(name)
    }


    fn contains_key(&self, name: &Value) -> bool {
        self.properties.contains_key(name)
    }

    fn name(&self) -> String {
        "Object".to_string()
    }

    fn to_string(&self) -> String {
        "[object Object]".to_string()
    }
}