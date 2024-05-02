mod common;

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use lazy_static::lazy_static;
use crate::{NativeFunction, Value};
use common::*;


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