use crate::value::{MutObj, Obj, ObjectOrNull};
use common::{
    define_getter, define_setter, has_own_property, is_prototype_of, lookup_getter, lookup_setter,
    property_is_enumerable, to_locale_string, to_string, value_of,
};
use std::any::Any;
use std::cell::RefCell;
use yavashark_string::YSString;

use crate::object::constructor::ObjectConstructor;
use crate::object::prototype::common::get_own_property_descriptor;
use crate::realm::Realm;
use crate::{Error, MutObject, NativeFunction, ObjectHandle, ObjectProperty, Res, Value, Variable};

pub mod common;

pub trait Proto: Obj {
    fn as_any(&mut self) -> &mut dyn Any;
}

#[derive(Debug, PartialEq, Eq)]
struct MutPrototype {
    object: MutObject,

    //common properties
    defined_getter: ObjectProperty,
    defined_setter: ObjectProperty,
    lookup_getter: ObjectProperty,
    lookup_setter: ObjectProperty,
    constructor: ObjectProperty,
    has_own_property: ObjectProperty,
    get_own_property_descriptor: ObjectProperty,
    is_prototype_of: ObjectProperty,
    property_is_enumerable: ObjectProperty,
    to_locale_string: ObjectProperty,
    to_string: ObjectProperty,
    value_of: ObjectProperty,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Prototype {
    inner: RefCell<MutPrototype>,
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
            inner: RefCell::new(MutPrototype {
                object: MutObject::with_proto(ObjectOrNull::Null),
                defined_getter: Value::Undefined.into(),
                defined_setter: Value::Undefined.into(),
                lookup_getter: Value::Undefined.into(),
                lookup_setter: Value::Undefined.into(),
                constructor: Value::Undefined.into(),
                has_own_property: Value::Undefined.into(),
                get_own_property_descriptor: Value::Undefined.into(),
                is_prototype_of: Value::Undefined.into(),
                property_is_enumerable: Value::Undefined.into(),
                to_locale_string: Value::Undefined.into(),
                to_string: Value::Undefined.into(),
                value_of: Value::Undefined.into(),
            }),
        }
    }

    pub(crate) fn initialize(&self, func: ObjectHandle, this: ObjectHandle) -> Res {
        let obj_constructor = ObjectConstructor::new(this.clone(), func.clone())?;

        let mut this_borrow = self.inner.try_borrow_mut()?;

        this_borrow.defined_getter = Variable::write_config(
            NativeFunction::with_proto("__defineGetter__", define_getter, func.clone()).into(),
        )
        .into();
        this_borrow.defined_setter = Variable::write_config(
            NativeFunction::with_proto("__defineSetter__", define_setter, func.clone()).into(),
        )
        .into();
        this_borrow.lookup_getter = Variable::write_config(
            NativeFunction::with_proto("__lookupGetter__", lookup_getter, func.clone()).into(),
        )
        .into();
        this_borrow.lookup_setter = Variable::write_config(
            NativeFunction::with_proto("__lookupSetter__", lookup_setter, func.clone()).into(),
        )
        .into();
        this_borrow.constructor = obj_constructor.into();

        this_borrow
            .constructor
            .value
            .as_object()?
            .define_variable("prototype".into(), Variable::new_read_only(this.into()))?;

        this_borrow.has_own_property =
            NativeFunction::with_proto("hasOwnProperty", has_own_property, func.clone()).into();
        this_borrow.get_own_property_descriptor = NativeFunction::with_proto(
            "getOwnPropertyDescriptor",
            get_own_property_descriptor,
            func.clone(),
        )
        .into();
        this_borrow.is_prototype_of =
            NativeFunction::with_proto("isPrototypeOf", is_prototype_of, func.clone()).into();
        this_borrow.property_is_enumerable = NativeFunction::with_proto(
            "propertyIsEnumerable",
            property_is_enumerable,
            func.clone(),
        )
        .into();
        this_borrow.to_locale_string =
            NativeFunction::with_proto("toLocaleString", to_locale_string, func.clone()).into();
        this_borrow.to_string =
            NativeFunction::with_proto("toString", to_string, func.clone()).into();
        this_borrow.value_of = NativeFunction::with_proto("valueOf", value_of, func).into();

        Ok(())
    }

    const DIRECT_PROPERTIES: &'static [&'static str] = &[
        "__defineGetter__",
        "__defineSetter__",
        "__lookupGetter__",
        "__lookupSetter__",
        "constructor",
        "hasOwnProperty",
        "isPrototypeOf",
        "propertyIsEnumerable",
        "toLocaleString",
        "toString",
        "valueOf",
    ];
}

impl Obj for Prototype {
    fn define_property(&self, name: Value, value: Value) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        if let Value::String(name) = &name {
            match name.as_str() {
                "__defineGetter__" => {
                    this.defined_getter = value.into();
                    return Ok(());
                }
                "__defineSetter__" => {
                    this.defined_setter = value.into();
                    return Ok(());
                }

                "__lookupGetter__" => {
                    this.lookup_getter = value.into();
                    return Ok(());
                }

                "__lookupSetter__" => {
                    this.lookup_setter = value.into();
                    return Ok(());
                }

                "constructor" => {
                    this.constructor = value.into();
                    return Ok(());
                }

                "hasOwnProperty" => {
                    this.has_own_property = value.into();
                    return Ok(());
                }

                "getOwnPropertyDescriptor" => {
                    this.get_own_property_descriptor = value.into();
                    return Ok(());
                }

                "isPrototypeOf" => {
                    this.is_prototype_of = value.into();
                    return Ok(());
                }

                "propertyIsEnumerable" => {
                    this.property_is_enumerable = value.into();
                    return Ok(());
                }

                "toLocaleString" => {
                    this.to_locale_string = value.into();
                    return Ok(());
                }

                "toString" => {
                    this.to_string = value.into();
                    return Ok(());
                }

                "valueOf" => {
                    this.value_of = value.into();
                    return Ok(());
                }

                _ => {}
            }
        }

        this.object.define_property(name, value)
    }

    fn define_variable(&self, name: Value, value: Variable) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        this.object.define_variable(name, value)
    }

    fn resolve_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        let this = self.inner.try_borrow()?;

        if let Value::String(name) = name {
            match name.as_str() {
                "__defineGetter__" => return Ok(Some(this.defined_getter.copy())),
                "__defineSetter__" => return Ok(Some(this.defined_setter.copy())),
                "__lookupGetter__" => return Ok(Some(this.lookup_getter.copy())),
                "__lookupSetter__" => return Ok(Some(this.lookup_setter.copy())),
                "constructor" => return Ok(Some(this.constructor.copy())),
                "hasOwnProperty" => return Ok(Some(this.has_own_property.copy())),
                "getOwnPropertyDescriptor" => {
                    return Ok(Some(this.get_own_property_descriptor.copy()))
                }
                "isPrototypeOf" => return Ok(Some(this.is_prototype_of.copy())),
                "propertyIsEnumerable" => return Ok(Some(this.property_is_enumerable.copy())),
                "toLocaleString" => return Ok(Some(this.to_locale_string.copy())),
                "toString" => return Ok(Some(this.to_string.copy())),
                "valueOf" => return Ok(Some(this.value_of.copy())),
                _ => {}
            }
        }
        this.object.resolve_property(name)
    }

    fn get_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        let this = self.inner.try_borrow()?;

        if let Value::String(name) = name {
            match name.as_str() {
                "__defineGetter__" => return Ok(Some(this.defined_getter.copy())),
                "__defineSetter__" => return Ok(Some(this.defined_setter.copy())),
                "__lookupGetter__" => return Ok(Some(this.lookup_getter.copy())),
                "__lookupSetter__" => return Ok(Some(this.lookup_setter.copy())),
                "constructor" => return Ok(Some(this.constructor.copy())),
                "hasOwnProperty" => return Ok(Some(this.has_own_property.copy())),
                "getOwnPropertyDescriptor" => {
                    return Ok(Some(this.get_own_property_descriptor.copy()))
                }
                "isPrototypeOf" => return Ok(Some(this.is_prototype_of.copy())),
                "propertyIsEnumerable" => return Ok(Some(this.property_is_enumerable.copy())),
                "toLocaleString" => return Ok(Some(this.to_locale_string.copy())),
                "toString" => return Ok(Some(this.to_string.copy())),
                "valueOf" => return Ok(Some(this.value_of.copy())),
                _ => {}
            }
        }

        this.object.get_property(name).map(|v| v.map(|v| v.copy()))
    }

    fn define_getter(&self, name: Value, value: Value) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        this.object.define_getter(name, value)
    }

    fn define_setter(&self, name: Value, value: Value) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        this.object.define_setter(name, value)
    }

    fn delete_property(&self, name: &Value) -> Res<Option<Value>> {
        if let Value::String(name) = name {
            match name.as_str() {
                "__defineGetter__" => {
                    let mut this = self.inner.try_borrow_mut()?;
                    this.defined_getter = Value::Undefined.into();
                    return Ok(Some(Value::Undefined));
                }
                "__defineSetter__" => {
                    let mut this = self.inner.try_borrow_mut()?;
                    this.defined_setter = Value::Undefined.into();
                    return Ok(None);
                }
                "__lookupGetter__" => {
                    let mut this = self.inner.try_borrow_mut()?;
                    this.lookup_getter = Value::Undefined.into();
                    return Ok(None);
                }
                "__lookupSetter__" => {
                    let mut this = self.inner.try_borrow_mut()?;
                    this.lookup_setter = Value::Undefined.into();
                    return Ok(None);
                }
                "constructor" => {
                    let mut this = self.inner.try_borrow_mut()?;
                    this.constructor = Value::Undefined.into();
                    return Ok(None);
                }
                "hasOwnProperty" => {
                    let mut this = self.inner.try_borrow_mut()?;
                    this.has_own_property = Value::Undefined.into();
                    return Ok(None);
                }
                "getOwnPropertyDescriptor" => {
                    let mut this = self.inner.try_borrow_mut()?;
                    this.get_own_property_descriptor = Value::Undefined.into();
                    return Ok(None);
                }
                "isPrototypeOf" => {
                    let mut this = self.inner.try_borrow_mut()?;
                    this.is_prototype_of = Value::Undefined.into();
                    return Ok(None);
                }
                "propertyIsEnumerable" => {
                    let mut this = self.inner.try_borrow_mut()?;
                    this.property_is_enumerable = Value::Undefined.into();
                    return Ok(None);
                }
                "toLocaleString" => {
                    let mut this = self.inner.try_borrow_mut()?;
                    this.to_locale_string = Value::Undefined.into();
                    return Ok(None);
                }
                "toString" => {
                    let mut this = self.inner.try_borrow_mut()?;
                    this.to_string = Value::Undefined.into();
                    return Ok(None);
                }
                "valueOf" => {
                    let mut this = self.inner.try_borrow_mut()?;
                    this.value_of = Value::Undefined.into();
                    return Ok(None);
                }

                _ => {}
            }
        }

        let mut this = self.inner.try_borrow_mut()?;

        this.object.delete_property(name)
    }

    fn contains_key(&self, name: &Value) -> Res<bool> {
        if let Value::String(name) = name {
            if Self::DIRECT_PROPERTIES.contains(&name.as_str()) {
                return Ok(true);
            }
        }

        let this = self.inner.try_borrow()?;

        this.object.contains_key(name)
    }

    fn name(&self) -> String {
        "Object".to_string()
    }

    fn to_string(&self, _realm: &mut Realm) -> Res<YSString, Error> {
        Ok("[object Object]".into())
    }

    fn to_string_internal(&self) -> Res<YSString> {
        Ok("[object Prototype]".into())
    }

    fn properties(&self) -> Res<Vec<(Value, Value)>> {
        let this = self.inner.try_borrow()?;

        let mut props = this.object.properties()?;
        props.push((
            Value::string("__defineGetter__"),
            this.defined_getter.value.copy(),
        ));
        props.push((
            Value::string("__defineSetter__"),
            this.defined_setter.value.copy(),
        ));
        props.push((
            Value::string("__lookupGetter__"),
            this.lookup_getter.value.copy(),
        ));
        props.push((
            Value::string("__lookupSetter__"),
            this.lookup_setter.value.copy(),
        ));
        props.push((Value::string("constructor"), this.constructor.value.copy()));
        props.push((
            Value::string("hasOwnProperty"),
            this.has_own_property.value.copy(),
        ));
        props.push((
            Value::string("getOwnPropertyDescriptor"),
            this.get_own_property_descriptor.value.copy(),
        ));
        props.push((
            Value::string("isPrototypeOf"),
            this.is_prototype_of.value.copy(),
        ));
        props.push((
            Value::string("propertyIsEnumerable"),
            this.property_is_enumerable.value.copy(),
        ));
        props.push((
            Value::string("toLocaleString"),
            this.to_locale_string.value.copy(),
        ));
        props.push((Value::string("toString"), this.to_string.value.copy()));
        props.push((Value::string("valueOf"), this.value_of.value.copy()));

        Ok(props)
    }

    fn keys(&self) -> Res<Vec<Value>> {
        let this = self.inner.try_borrow()?;

        let mut keys = this.object.keys()?;

        for key in Self::DIRECT_PROPERTIES {
            keys.push(Value::string(key));
        }

        Ok(keys)
    }

    fn values(&self) -> Res<Vec<Value>> {
        let this = self.inner.try_borrow()?;

        let mut values = this.object.values()?;

        values.push(this.defined_getter.value.copy());
        values.push(this.defined_setter.value.copy());
        values.push(this.lookup_getter.value.copy());
        values.push(this.lookup_setter.value.copy());
        values.push(this.constructor.value.copy());
        values.push(this.has_own_property.value.copy());
        values.push(this.get_own_property_descriptor.value.copy());
        values.push(this.is_prototype_of.value.copy());
        values.push(this.property_is_enumerable.value.copy());
        values.push(this.to_locale_string.value.copy());
        values.push(this.to_string.value.copy());
        values.push(this.value_of.value.copy());

        Ok(values)
    }

    fn get_array_or_done(&self, index: usize) -> Res<(bool, Option<Value>)> {
        let this = self.inner.try_borrow()?;

        this.object.get_array_or_done(index)
    }

    fn clear_values(&self) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        this.object.clear_values()
    }

    fn prototype(&self) -> Res<ObjectProperty> {
        Ok(Value::Null.into())
    }

    fn constructor(&self) -> Res<ObjectProperty> {
        let this = self.inner.try_borrow()?;

        Ok(this.constructor.clone())
    }
}

impl Proto for Prototype {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
