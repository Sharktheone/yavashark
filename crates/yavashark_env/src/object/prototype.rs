use std::any::Any;

use common::{
    define_getter, define_setter, has_own_property, is_prototype_of, lookup_getter, lookup_setter,
    object_constructor, property_is_enumerable, to_locale_string, to_string, value_of,
};
use yavashark_value::Obj;

use crate::object::Object;
use crate::realm::Realm;
use crate::{Error, NativeFunction, ObjectProperty, Res, Value, Variable};

mod common;

pub trait Proto: Obj<Realm> {
    fn as_any(&mut self) -> &mut dyn Any;
}

#[derive(Debug, PartialEq, Eq)]
pub struct Prototype {
    object: Object,

    //common properties
    defined_getter: ObjectProperty,
    defined_setter: ObjectProperty,
    lookup_getter: ObjectProperty,
    lookup_setter: ObjectProperty,
    constructor: ObjectProperty,
    has_own_property: ObjectProperty,
    has_own_property_descriptor: ObjectProperty,
    is_prototype_of: ObjectProperty,
    property_is_enumerable: ObjectProperty,
    to_locale_string: ObjectProperty,
    to_string: ObjectProperty,
    value_of: ObjectProperty,
}

impl Default for Prototype {
    fn default() -> Self {
        Self::new()
    }
}

impl Prototype {
    #[must_use]
    pub fn new() -> Self {
        Self {
            object: Object::raw_with_proto(Value::Undefined),
            defined_getter: Value::Undefined.into(),
            defined_setter: Value::Undefined.into(),
            lookup_getter: Value::Undefined.into(),
            lookup_setter: Value::Undefined.into(),
            constructor: Value::Undefined.into(),
            has_own_property: Value::Undefined.into(),
            has_own_property_descriptor: Value::Undefined.into(),
            is_prototype_of: Value::Undefined.into(),
            property_is_enumerable: Value::Undefined.into(),
            to_locale_string: Value::Undefined.into(),
            to_string: Value::Undefined.into(),
            value_of: Value::Undefined.into(),
        }
    }

    pub(crate) fn initialize(&mut self, func: Value, this: Value) -> Res {
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

        self.constructor
            .value
            .define_property("prototype".into(), this)?;

        self.has_own_property =
            NativeFunction::with_proto("hasOwnProperty", has_own_property, func.copy()).into();
        self.has_own_property_descriptor =
            NativeFunction::with_proto("hasOwnPropertyDescriptor", has_own_property, func.copy()).into();
        self.is_prototype_of =
            NativeFunction::with_proto("isPrototypeOf", is_prototype_of, func.copy()).into();
        self.property_is_enumerable =
            NativeFunction::with_proto("propertyIsEnumerable", property_is_enumerable, func.copy())
                .into();
        self.to_locale_string =
            NativeFunction::with_proto("toLocaleString", to_locale_string, func.copy()).into();
        self.to_string = NativeFunction::with_proto("toString", to_string, func.copy()).into();
        self.value_of = NativeFunction::with_proto("valueOf", value_of, func).into();

        Ok(())
    }

    const DIRECT_PROPERTIES: &'static [&'static str] = &[
        "__define_getter__",
        "__define_setter__",
        "__lookup_getter__",
        "__lookup_setter__",
        "constructor",
        "hasOwnProperty",
        "isPrototypeOf",
        "propertyIsEnumerable",
        "toLocaleString",
        "toString",
        "valueOf",
    ];
}

impl Obj<Realm> for Prototype {
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
                
                "hasOwnPropertyDescriptor" => {
                    self.has_own_property_descriptor = value.into();
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

        self.object.define_property(name, value);
    }

    fn define_variable(&mut self, name: Value, value: Variable) {
        self.object.define_variable(name, value);
    }

    fn resolve_property(&self, name: &Value) -> Option<ObjectProperty> {
        if let Value::String(name) = name {
            match name.as_str() {
                "__define_getter__" => return Some(self.defined_getter.copy()),
                "__define_setter__" => return Some(self.defined_setter.copy()),
                "__lookup_getter__" => return Some(self.lookup_getter.copy()),
                "__lookup_setter__" => return Some(self.lookup_setter.copy()),
                "constructor" => return Some(self.constructor.copy()),
                "hasOwnProperty" => return Some(self.has_own_property.copy()),
                "hasOwnPropertyDescriptor" => return Some(self.has_own_property_descriptor.copy()),
                "isPrototypeOf" => return Some(self.is_prototype_of.copy()),
                "propertyIsEnumerable" => return Some(self.property_is_enumerable.copy()),
                "toLocaleString" => return Some(self.to_locale_string.copy()),
                "toString" => return Some(self.to_string.copy()),
                "valueOf" => return Some(self.value_of.copy()),
                _ => {}
            }
        }
        self.object.resolve_property(name)
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
                "hasOwnPropertyDescriptor" => return Some(&self.has_own_property_descriptor.value),
                "isPrototypeOf" => return Some(&self.is_prototype_of.value),
                "propertyIsEnumerable" => return Some(&self.property_is_enumerable.value),
                "toLocaleString" => return Some(&self.to_locale_string.value),
                "toString" => return Some(&self.to_string.value),
                "valueOf" => return Some(&self.value_of.value),
                _ => {}
            }
        }

        self.object.get_property(name)
    }

    fn define_getter(&mut self, name: Value, value: Value) -> Res {
        self.object.define_getter(name, value)
    }

    fn define_setter(&mut self, name: Value, value: Value) -> Res {
        self.object.define_setter(name, value)
    }

    fn get_getter(&self, name: &Value) -> Option<Value> {
        self.object.get_getter(name)
    }

    fn get_setter(&self, name: &Value) -> Option<Value> {
        self.object.get_setter(name)
    }

    fn delete_property(&mut self, name: &Value) -> Option<Value> {
        if let Value::String(name) = name {
            if Self::DIRECT_PROPERTIES.contains(&name.as_str()) {
                return None;
            }
        }
        self.object.delete_property(name)
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
                | "hasOwnPropertyDescriptor"
                | "isPrototypeOf"
                | "propertyIsEnumerable"
                | "toLocaleString"
                | "toString"
                | "valueOf" => return true,
                _ => {}
            }
        }

        self.object.contains_key(name)
    }

    fn name(&self) -> String {
        "Object".to_string()
    }

    fn to_string(&self, _realm: &mut Realm) -> Result<String, Error> {
        Ok("[object Object]".to_string())
    }

    fn to_string_internal(&self) -> String {
        "[object Prototype]".to_string()
    }

    fn properties(&self) -> Vec<(Value, Value)> {
        let mut props = self.object.properties();
        props.push((
            Value::String("__define_getter__".to_string()),
            self.defined_getter.value.copy(),
        ));
        props.push((
            Value::String("__define_setter__".to_string()),
            self.defined_setter.value.copy(),
        ));
        props.push((
            Value::String("__lookup_getter__".to_string()),
            self.lookup_getter.value.copy(),
        ));
        props.push((
            Value::String("__lookup_setter__".to_string()),
            self.lookup_setter.value.copy(),
        ));
        props.push((
            Value::String("constructor".to_string()),
            self.constructor.value.copy(),
        ));
        props.push((
            Value::String("hasOwnProperty".to_string()),
            self.has_own_property.value.copy(),
        ));
        props.push((
            Value::String("hasOwnPropertyDescriptor".to_string()),
            self.has_own_property_descriptor.value.copy(),
        ));
        props.push((
            Value::String("isPrototypeOf".to_string()),
            self.is_prototype_of.value.copy(),
        ));
        props.push((
            Value::String("propertyIsEnumerable".to_string()),
            self.property_is_enumerable.value.copy(),
        ));
        props.push((
            Value::String("toLocaleString".to_string()),
            self.to_locale_string.value.copy(),
        ));
        props.push((
            Value::String("toString".to_string()),
            self.to_string.value.copy(),
        ));
        props.push((
            Value::String("valueOf".to_string()),
            self.value_of.value.copy(),
        ));
        props
    }

    fn keys(&self) -> Vec<Value> {
        let mut keys = self.object.keys();
        keys.push(Value::String("__define_getter__".to_string()));
        keys.push(Value::String("__define_setter__".to_string()));
        keys.push(Value::String("__lookup_getter__".to_string()));
        keys.push(Value::String("__lookup_setter__".to_string()));
        keys.push(Value::String("constructor".to_string()));
        keys.push(Value::String("hasOwnProperty".to_string()));
        keys.push(Value::String("hasOwnPropertyDescriptor".to_string()));
        keys.push(Value::String("isPrototypeOf".to_string()));
        keys.push(Value::String("propertyIsEnumerable".to_string()));
        keys.push(Value::String("toLocaleString".to_string()));
        keys.push(Value::String("toString".to_string()));
        keys.push(Value::String("valueOf".to_string()));
        keys
    }

    fn values(&self) -> Vec<Value> {
        let mut values = self.object.values();
        values.push(self.defined_getter.value.copy());
        values.push(self.defined_setter.value.copy());
        values.push(self.lookup_getter.value.copy());
        values.push(self.lookup_setter.value.copy());
        values.push(self.constructor.value.copy());
        values.push(self.has_own_property.value.copy());
        values.push(self.has_own_property_descriptor.value.copy());
        values.push(self.is_prototype_of.value.copy());
        values.push(self.property_is_enumerable.value.copy());
        values.push(self.to_locale_string.value.copy());
        values.push(self.to_string.value.copy());
        values.push(self.value_of.value.copy());
        values
    }

    fn get_array_or_done(&self, index: usize) -> (bool, Option<Value>) {
        self.object.get_array_or_done(index)
    }

    fn clear_values(&mut self) {
        self.object.clear_values();
    }

    fn prototype(&self) -> ObjectProperty {
        Value::Undefined.into()
    }

    fn constructor(&self) -> ObjectProperty {
        self.constructor.clone()
    }
}

impl Proto for Prototype {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
