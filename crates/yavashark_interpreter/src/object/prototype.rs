use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use common::*;
use yavashark_value::Obj;

use crate::context::Context;
use crate::{NativeFunction, Value};

mod common;

pub trait Proto: Obj<Context> {
    fn as_any(&mut self) -> &mut dyn Any;
}

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
            defined_getter: Value::Undefined,
            defined_setter: Value::Undefined,
            lookup_getter: Value::Undefined,
            lookup_setter: Value::Undefined,
            constructor: Value::Undefined,
            has_own_property: Value::Undefined,
            is_prototype_of: Value::Undefined,
            property_is_enumerable: Value::Undefined,
            to_locale_string: Value::Undefined,
            to_string: Value::Undefined,
            value_of: Value::Undefined,
        }
    }

    pub(crate) fn initialize(&mut self, func: Value) {
        self.defined_getter =
            NativeFunction::with_proto("__define_getter__", define_getter, func.copy()).into();
        self.defined_setter =
            NativeFunction::with_proto("__define_setter__", define_setter, func.copy()).into();
        self.lookup_getter =
            NativeFunction::with_proto("__lookup_getter__", lookup_getter, func.copy()).into();
        self.lookup_setter =
            NativeFunction::with_proto("__lookup_setter__", lookup_setter, func.copy()).into();
        self.constructor =
            NativeFunction::with_proto("Object", object_constructor, func.copy()).into();
        self.has_own_property =
            NativeFunction::with_proto("hasOwnProperty", has_own_property, func.copy()).into();
        self.is_prototype_of =
            NativeFunction::with_proto("isPrototypeOf", is_prototype_of, func.copy()).into();
        self.property_is_enumerable = NativeFunction::with_proto(
            "propertyIsEnumerable",
            property_is_enumerable,
            func.copy(),
        )
        .into();
        self.to_locale_string =
            NativeFunction::with_proto("toLocaleString", to_locale_string, func.copy()).into();
        self.to_string = NativeFunction::with_proto("toString", to_string, func.copy()).into();
        self.value_of = NativeFunction::with_proto("valueOf", value_of, func).into();
    }
}

impl Obj<Context> for Prototype {
    fn define_property(&mut self, name: Value, value: Value) {
        if let Value::String(name) = &name {
            match name.as_str() {
                "__define_getter__" => {
                    self.defined_getter = value;
                    return;
                }
                "__define_setter__" => {
                    self.defined_setter = value;
                    return;
                }

                "__lookup_getter__" => {
                    self.lookup_getter = value;
                    return;
                }

                "__lookup_setter__" => {
                    self.lookup_setter = value;
                    return;
                }

                "constructor" => {
                    self.constructor = value;
                    return;
                }

                "hasOwnProperty" => {
                    self.has_own_property = value;
                    return;
                }

                "isPrototypeOf" => {
                    self.is_prototype_of = value;
                    return;
                }

                "propertyIsEnumerable" => {
                    self.property_is_enumerable = value;
                    return;
                }

                "toLocaleString" => {
                    self.to_locale_string = value;
                    return;
                }

                "toString" => {
                    self.to_string = value;
                    return;
                }

                "valueOf" => {
                    self.value_of = value;
                    return;
                }

                _ => {}
            }
        }

        self.properties.insert(name, value);
    }

    fn resolve_property(&self, name: &Value) -> Option<Value> {
        self.properties.get(name).map(|v| v.copy())
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
    
    fn properties(&self) -> Vec<(Value, Value)> {
        self.properties.iter().map(|(k, v)| (k.copy(), v.copy())).collect()
    }
    
    fn keys(&self) -> Vec<Value> {
        self.properties.keys().map(|v| v.copy()).collect()
    }
    
    fn values(&self) -> Vec<Value> {
        self.properties.values().map(|v| v.copy()).collect()
    }
}

impl Proto for Prototype {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
