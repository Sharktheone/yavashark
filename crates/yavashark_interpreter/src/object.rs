mod prototype;

pub use prototype::*;
use std::any::Any;
use std::cell::{Ref, RefCell};
use std::collections::HashMap;
use std::fmt::Debug;
use std::rc::Rc;

use yavashark_value::Obj;

use crate::context::Context;
use crate::Value;

#[derive(Debug)]
pub struct Object {
    pub properties: HashMap<Value, Value>,
    pub prototype: Value,
}

impl Object {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(context: &mut Context) -> crate::ObjectHandle {
        let prototype = context.obj_prototype.clone().into();

        let this: Box<dyn Obj<Context>> = Box::new(Self {
            properties: HashMap::new(),
            prototype,
        });

        this.into()
    }

    pub fn with_proto(proto: Value) -> crate::ObjectHandle {
        let this: Box<dyn Obj<Context>> = Box::new(Self {
            properties: HashMap::new(),
            prototype: proto,
        });

        this.into()
    }
    
    
    pub fn raw(context: &mut Context) -> Self {
        let prototype = context.obj_prototype.clone().into();
        
        Self {
            properties: HashMap::new(),
            prototype,
        }
    }
    
    
    pub fn raw_with_proto(proto: Value) -> Self {
        Self {
            properties: HashMap::new(),
            prototype: proto,
        }
    }
}

impl Obj<Context> for Object {
    fn define_property(&mut self, name: Value, value: Value) {
        self.properties.insert(name, value);
    }

    fn resolve_property(&self, name: &Value) -> Option<Value> {
        if name == &Value::String("__proto__".to_string()) {
            return Some(self.prototype.copy());
        }
        self.properties
            .get(name)
            .map(|v| v.copy())
            .or_else(|| match &self.prototype {
                Value::Object(o) => o.get_property(name).ok(),
                Value::Function(f) => f.get_property(name).ok(),
                _ => None,
            })
    }

    fn get_property(&self, name: &Value) -> Option<&Value> {
        if name == &Value::String("__proto__".to_string()) {
            return Some(&self.prototype);
        }
        self.properties.get(name)
    }

    fn get_property_mut(&mut self, name: &Value) -> Option<&mut Value> {
        if name == &Value::String("__proto__".to_string()) {
            return Some(&mut self.prototype);
        }
        self.properties.get_mut(name)
    }

    fn contains_key(&self, name: &Value) -> bool {
        if name == &Value::String("__proto__".to_string()) {
            return true;
        }
        self.properties.contains_key(name)
    }

    fn name(&self) -> String {
        "Object".to_string()
    }

    fn to_string(&self) -> String {
        "[object Object]".to_string()
    }
}