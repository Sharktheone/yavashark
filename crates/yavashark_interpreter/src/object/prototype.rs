mod common;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use crate::{NativeFunction, Value};
use common::*;
use yavashark_value::Obj;
use crate::context::Context;


#[derive(Debug, PartialEq, Eq)]
pub struct Prototype {
    properties: HashMap<Value, Value>,
    parent: Option<Rc<RefCell<Prototype>>>,

    //common properties
    defined_getter: Value,
    defined_setter: Value,
    lookup_getter: Value,
    lookup_setter: Value,
    constructor: Value,
    has_own_property: Value,
    is_prototype_of: Value,
    property_is_enumerable: Value,
    to_locale_string: Value,
    to_string: Value,
    value_of: Value,
}


impl Default for Prototype {
    fn default() -> Self {
        Self::new()
    }
}

impl Prototype {
    pub fn new() -> Self {
        Self {
            properties: HashMap::new(),
            parent: None,
            defined_getter: NativeFunction::new("__define_getter__", define_getter).into(),
            defined_setter: NativeFunction::new("__define_setter__", define_setter).into(),
            lookup_getter: NativeFunction::new("__lookup_getter__", lookup_getter).into(),
            lookup_setter: NativeFunction::new("__lookup_setter__", lookup_setter).into(),
            constructor: NativeFunction::new("Object", object_constructor).into(),
            has_own_property: NativeFunction::new("hasOwnProperty", has_own_property).into(),
            is_prototype_of: NativeFunction::new("isPrototypeOf", is_prototype_of).into(),
            property_is_enumerable: NativeFunction::new("propertyIsEnumerable", &property_is_enumerable).into(),
            to_locale_string: NativeFunction::new("toLocaleString", to_locale_string).into(),
            to_string: NativeFunction::new("toString", to_string).into(),
            value_of: NativeFunction::new("valueOf", value_of).into(),
        }
    }
}


impl Obj<Context> for Prototype {
    fn define_property(&mut self, name: Value, value: Value) {
        if let Value::String(name) = &name {
            match name.as_str() {
                "__define_getter__" => {
                    self.defined_getter = value;
                    return;
                },
                "__define_setter__" => {
                    self.defined_setter = value;
                    return;
                },
                
                "__lookup_getter__" => {
                    self.lookup_getter = value;
                    return;
                },
                
                "__lookup_setter__" => {
                    self.lookup_setter = value;
                    return;
                },
                
                "constructor" => {
                    self.constructor = value;
                    return;
                },
                
                "hasOwnProperty" => {
                    self.has_own_property = value;
                    return;
                },
                
                "isPrototypeOf" => {
                    self.is_prototype_of = value;
                    return;
                },
                
                "propertyIsEnumerable" => {
                    self.property_is_enumerable = value;
                    return;
                },
                
                "toLocaleString" => {
                    self.to_locale_string = value;
                    return;
                },
                
                "toString" => {
                    self.to_string = value;
                    return;
                },
                
                "valueOf" => {
                    self.value_of = value;
                    return;
                },
                
                _ => {}
            }
        }
        
        self.properties.insert(name, value);
    }
    
    
    fn resolve_property(&self, name: &Value) -> Option<Value> {
        self.properties.get(&name).map(|v| v.copy())
    }

    fn get_property(&self, name: &Value) -> Option<&Value> {
        if let Value::String(name) = name {
            match name.as_str() {
                "__define_getter__" => return Some(&self.defined_getter),
                "__define_setter__" => return Some(&self.defined_setter),
                "__lookup_getter__" => return Some(&self.lookup_getter),
                "__lookup_setter__" => return Some(&self.lookup_setter),
                "constructor" => return Some(&self.constructor),
                "hasOwnProperty" => return Some(&self.has_own_property),
                "isPrototypeOf" => return Some(&self.is_prototype_of),
                "propertyIsEnumerable" => return Some(&self.property_is_enumerable),
                "toLocaleString" => return Some(&self.to_locale_string),
                "toString" => return Some(&self.to_string),
                "valueOf" => return Some(&self.value_of),
                _ => {}
            }
        }
        
        self.properties.get(name)
    }

    fn get_property_mut(&mut self, name: &Value) -> Option<&mut Value> {
        if let Value::String(name) = name {
            match name.as_str() {
                "__define_getter__" => return Some(&mut self.defined_getter),
                "__define_setter__" => return Some(&mut self.defined_setter),
                "__lookup_getter__" => return Some(&mut self.lookup_getter),
                "__lookup_setter__" => return Some(&mut self.lookup_setter),
                "constructor" => return Some(&mut self.constructor),
                "hasOwnProperty" => return Some(&mut self.has_own_property),
                "isPrototypeOf" => return Some(&mut self.is_prototype_of),
                "propertyIsEnumerable" => return Some(&mut self.property_is_enumerable),
                "toLocaleString" => return Some(&mut self.to_locale_string),
                "toString" => return Some(&mut self.to_string),
                "valueOf" => return Some(&mut self.value_of),
                _ => {}
            }
        }
        
        self.properties.get_mut(name)
    }

    fn contains_key(&self, name: &Value) -> bool {
        if let Value::String(name) = name {
            match name.as_str() {
                "__define_getter__" => return true,
                "__define_setter__" => return true,
                "__lookup_getter__" => return true,
                "__lookup_setter__" => return true,
                "constructor" => return true,
                "hasOwnProperty" => return true,
                "isPrototypeOf" => return true,
                "propertyIsEnumerable" => return true,
                "toLocaleString" => return true,
                "toString" => return true,
                "valueOf" => return true,
                _ => {}
            }
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