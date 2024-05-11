use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use common::*;
use yavashark_value::Obj;

use crate::context::Context;
use crate::{NativeFunction, Value, Variable};

mod common;

pub trait Proto: Obj<Context> {
    fn as_any(&mut self) -> &mut dyn Any;
}

#[derive(Debug, PartialEq, Eq)]
pub struct Prototype {
    properties: HashMap<Value, Variable>,
    parent: Option<Rc<RefCell<Prototype>>>,

    //common properties
    defined_getter: Variable,
    defined_setter: Variable,
    lookup_getter: Variable,
    lookup_setter: Variable,
    constructor: Variable,
    has_own_property: Variable,
    is_prototype_of: Variable,
    property_is_enumerable: Variable,
    to_locale_string: Variable,
    to_string: Variable,
    value_of: Variable,
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
            defined_getter: Value::Undefined.into(),
            defined_setter: Value::Undefined.into(),
            lookup_getter: Value::Undefined.into(),
            lookup_setter: Value::Undefined.into(),
            constructor: Value::Undefined.into(),
            has_own_property: Value::Undefined.into(),
            is_prototype_of: Value::Undefined.into(),
            property_is_enumerable: Value::Undefined.into(),
            to_locale_string: Value::Undefined.into(),
            to_string: Value::Undefined.into(),
            value_of: Value::Undefined.into(),
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
        self.property_is_enumerable =
            NativeFunction::with_proto("propertyIsEnumerable", property_is_enumerable, func.copy())
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
                    self.defined_getter = value.into();
                    return;
                }
                "__define_setter__" => {
                    self.defined_setter = value.into();
                    return;
                }

                "__lookup_getter__" => {
                    self.lookup_getter = value.into();
                    return;
                }

                "__lookup_setter__" => {
                    self.lookup_setter = value.into();
                    return;
                }

                "constructor" => {
                    self.constructor = value.into();
                    return;
                }

                "hasOwnProperty" => {
                    self.has_own_property = value.into();
                    return;
                }

                "isPrototypeOf" => {
                    self.is_prototype_of = value.into();
                    return;
                }

                "propertyIsEnumerable" => {
                    self.property_is_enumerable = value.into();
                    return;
                }

                "toLocaleString" => {
                    self.to_locale_string = value.into();
                    return;
                }

                "toString" => {
                    self.to_string = value.into();
                    return;
                }

                "valueOf" => {
                    self.value_of = value.into();
                    return;
                }

                _ => {}
            }
        }

        self.properties.insert(name, value.into());
    }

    fn define_variable(&mut self, name: Value, value: Variable) {
        todo!()
    }

    fn resolve_property(&self, name: &Value) -> Option<Value> {
        self.properties.get(name).map(yavashark_value::variable::Variable::copy)
    }

    fn get_property(&self, name: &Value) -> Option<&Value> {
        if let Value::String(name) = name {
            match name.as_str() {
                "__define_getter__" => return Some(&self.defined_getter.value),
                "__define_setter__" => return Some(&self.defined_setter.value),
                "__lookup_getter__" => return Some(&self.lookup_getter.value),
                "__lookup_setter__" => return Some(&self.lookup_setter.value),
                "constructor" => return Some(&self.constructor.value),
                "hasOwnProperty" => return Some(&self.has_own_property.value),
                "isPrototypeOf" => return Some(&self.is_prototype_of.value),
                "propertyIsEnumerable" => return Some(&self.property_is_enumerable.value),
                "toLocaleString" => return Some(&self.to_locale_string.value),
                "toString" => return Some(&self.to_string.value),
                "valueOf" => return Some(&self.value_of.value),
                _ => {}
            }
        }

        Some(&self.properties.get(name)?.value)
    }

    fn get_property_mut(&mut self, name: &Value) -> Option<&mut Value> {
        if let Value::String(name) = name {
            match name.as_str() {
                "__define_getter__" => return Some(&mut self.defined_getter.value),
                "__define_setter__" => return Some(&mut self.defined_setter.value),
                "__lookup_getter__" => return Some(&mut self.lookup_getter.value),
                "__lookup_setter__" => return Some(&mut self.lookup_setter.value),
                "constructor" => return Some(&mut self.constructor.value),
                "hasOwnProperty" => return Some(&mut self.has_own_property.value),
                "isPrototypeOf" => return Some(&mut self.is_prototype_of.value),
                "propertyIsEnumerable" => return Some(&mut self.property_is_enumerable.value),
                "toLocaleString" => return Some(&mut self.to_locale_string.value),
                "toString" => return Some(&mut self.to_string.value),
                "valueOf" => return Some(&mut self.value_of.value),
                _ => {}
            }
        }

        Some(&mut self.properties.get_mut(name)?.value) //TODO: Check if &mut is allowed (is_writable)
    }

    fn contains_key(&self, name: &Value) -> bool {
        if let Value::String(name) = name {
            match name.as_str() {
                "__define_getter__"
                | "__define_setter__"
                | "__lookup_getter__"
                | "__lookup_setter__"
                | "constructor"
                | "hasOwnProperty"
                | "isPrototypeOf"
                | "propertyIsEnumerable"
                | "toLocaleString"
                | "toString"
                | "valueOf" => return true,
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
        self.properties
            .iter()
            .map(|(k, v)| (k.copy(), v.copy()))
            .collect()
    }

    fn keys(&self) -> Vec<Value> {
        self.properties.keys().map(yavashark_value::Value::copy).collect()
    }

    fn values(&self) -> Vec<Value> {
        self.properties.values().map(yavashark_value::variable::Variable::copy).collect()
    }
}

impl Proto for Prototype {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
