use std::any::Any;
use std::collections::HashMap;
use std::fmt::Debug;

pub use prototype::*;
use yavashark_value::Obj;

use crate::context::Context;
use crate::Value;

mod prototype;

#[derive(Debug)]
pub struct Object {
    pub properties: HashMap<Value, Value>,
    pub array: Vec<(usize, Value)>,
    pub prototype: Value,
}

impl Object {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(context: &mut Context) -> crate::ObjectHandle {
        let prototype = context.obj_prototype.clone().into();

        let this: Box<dyn Obj<Context>> = Box::new(Self {
            properties: HashMap::new(),
            prototype,
            array: Vec::new(),
        });

        this.into()
    }

    pub fn with_proto(proto: Value) -> crate::ObjectHandle {
        let this: Box<dyn Obj<Context>> = Box::new(Self {
            properties: HashMap::new(),
            prototype: proto,
            array: Vec::new(),
        });

        this.into()
    }


    pub fn raw(context: &mut Context) -> Self {
        let prototype = context.obj_prototype.clone().into();

        Self {
            properties: HashMap::new(),
            prototype,
            array: Vec::new(),
        }
    }


    pub fn raw_with_proto(proto: Value) -> Self {
        Self {
            properties: HashMap::new(),
            prototype: proto,
            array: Vec::new(),
        }
    }


    pub fn array_position(&self, index: usize) -> (usize, bool) {
        if self.array.is_empty() {
            return (0, false);
        }

        if self.array.len() > 100 {
            return self.array.binary_search_by(|(i, _)| i.cmp(&index)).map(|i| (i, true)).unwrap_or_else(|i| (i, false));
        }

        for (i, (j, _)) in self.array.iter().enumerate() {
            if *j == index {
                return (i, true);
            }

            if *j > index {
                return (i, false);
            }
        }

        (self.array.len(), false)
    }


    pub fn insert_array(&mut self, index: usize, value: Value) {
        let (i, found) = self.array_position(index);

        if found {
            if let Some(v) = self.array.get_mut(i) {
                v.1 = value;
                return;
            };
        }

        self.array.insert(i, (index, value));
    }

    pub fn resolve_array(&self, index: usize) -> Option<Value> {
        let (i, found) = self.array_position(index);

        if found {
            return self.array.get(i).map(|v| v.1.copy());
        }

        None
    }

    pub fn get_array(&self, index: usize) -> Option<&Value> {
        let (i, found) = self.array_position(index);

        if found {
            return self.array.get(i).map(|v| &v.1);
        }

        None
    }

    pub fn get_array_mut(&mut self, index: usize) -> Option<&mut Value> {
        let (i, found) = self.array_position(index);

        if found {
            return self.array.get_mut(i).map(|v| &mut v.1);
        }

        None
    }

    pub fn contains_array_key(&self, index: usize) -> bool {
        let (_, found) = self.array_position(index);

        found
    }
}

impl Obj<Context> for Object {
    fn define_property(&mut self, name: Value, value: Value) {
        if let Value::Number(n) = &name {
            self.insert_array(*n as usize, value);
            return;
        }


        self.properties.insert(name, value);
    }

    fn resolve_property(&self, name: &Value) -> Option<Value> {
        if name == &Value::String("__proto__".to_string()) {
            return Some(self.prototype.copy());
        }

        if let Value::Number(n) = name {
            return self.resolve_array(*n as usize);
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
        
        if let Value::Number(n) = name {
            return self.get_array(*n as usize);
        }

        self.properties.get(name)
    }

    fn get_property_mut(&mut self, name: &Value) -> Option<&mut Value> {
        if name == &Value::String("__proto__".to_string()) {
            return Some(&mut self.prototype);
        }


        if let Value::Number(n) = name {
            return self.get_array_mut(*n as usize);
        }

        self.properties.get_mut(name)
    }

    fn contains_key(&self, name: &Value) -> bool {
        if name == &Value::String("__proto__".to_string()) {
            return true;
        }


        if let Value::Number(n) = name {
            return self.contains_array_key(*n as usize);
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
