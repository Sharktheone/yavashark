use std::collections::HashMap;
use std::fmt::Debug;

use yavashark_value::Obj;
use yavashark_value::Object as ObjWrapper;

use crate::context::Context;
use crate::Value;

#[derive(Debug)]
pub struct Object {
    properties: HashMap<Value, Value>,
}

impl Object {
    pub fn new() -> ObjWrapper<Context> {
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

    fn get_property(&self, name: &Value) -> Option<&Value> {
        self.properties.get(name)
    }

    fn get_property_mut(&mut self, name: &Value) -> Option<&mut Value> {
        todo!()
    }

    fn update_or_define_property(&mut self, name: Value, value: Value) {
        todo!()
    }

    fn contains_key(&self, name: &Value) -> bool {
        todo!()
    }

    fn name(&self) -> &str {
        todo!()
    }

    fn to_string(&self) -> String {
        todo!()
    }
}