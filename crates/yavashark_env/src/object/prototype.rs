use crate::value::{BoxedObj, DefinePropertyResult, MutObj, Obj, ObjectOrNull, Property};
use common::{
    define_getter, define_setter, has_own_property, is_prototype_of, lookup_getter, lookup_setter,
    property_is_enumerable, to_locale_string, to_string, value_of,
};
use std::any::Any;
use std::cell::RefCell;
use yavashark_garbage::GcRef;

use crate::object::constructor::ObjectConstructor;
use crate::object::prototype::common::get_own_property_descriptor;
use crate::realm::Realm;
use crate::{InternalPropertyKey, MutObject, NativeFunction, ObjectHandle, ObjectProperty, PropertyKey, Res, Value, Variable};
use crate::realm::resolve::ResolveModuleResult;

pub mod common;

pub trait Proto: Obj {
    fn as_any(&mut self) -> &mut dyn Any;
}

#[derive(Debug, PartialEq, Eq)]
struct MutPrototype {
    object: MutObject,

    //common properties
    defined_getter: Variable,
    defined_setter: Variable,
    lookup_getter: Variable,
    lookup_setter: Variable,
    constructor: Variable,
    has_own_property: Variable,
    get_own_property_descriptor: Variable,
    is_prototype_of: Variable,
    property_is_enumerable: Variable,
    to_locale_string: Variable,
    to_string: Variable,
    value_of: Variable,
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

    pub(crate) fn initialize(&self, func: ObjectHandle, this: ObjectHandle, realm: &mut Realm) -> Res {
        let obj_constructor = ObjectConstructor::new(this.clone(), func.clone(), realm)?;

        let mut this_borrow = self.inner.try_borrow_mut()?;

        this_borrow.defined_getter = Variable::write_config(
            NativeFunction::with_proto("__defineGetter__", define_getter, func.clone(), realm).into(),
        )
        .into();
        this_borrow.defined_setter = Variable::write_config(
            NativeFunction::with_proto("__defineSetter__", define_setter, func.clone(), realm).into(),
        )
        .into();
        this_borrow.lookup_getter = Variable::write_config(
            NativeFunction::with_proto("__lookupGetter__", lookup_getter, func.clone(), realm).into(),
        )
        .into();
        this_borrow.lookup_setter = Variable::write_config(
            NativeFunction::with_proto("__lookupSetter__", lookup_setter, func.clone(), realm).into(),
        )
        .into();
        this_borrow.constructor = obj_constructor.into();

        this_borrow
            .constructor
            .value
            .as_object()?
            .define_property_attributes("prototype".into(), Variable::new_read_only(this.into()), realm)?;

        this_borrow.has_own_property =
            NativeFunction::with_proto("hasOwnProperty", has_own_property, func.clone(), realm).into();
        this_borrow.get_own_property_descriptor = NativeFunction::with_proto(
            "getOwnPropertyDescriptor",
            get_own_property_descriptor,
            func.clone(),
            realm,
        )
        .into();
        this_borrow.is_prototype_of =
            NativeFunction::with_proto("isPrototypeOf", is_prototype_of, func.clone(), realm).into();
        this_borrow.property_is_enumerable = NativeFunction::with_proto(
            "propertyIsEnumerable",
            property_is_enumerable,
            func.clone(),
            realm,
        )
        .into();
        this_borrow.to_locale_string =
            NativeFunction::with_proto("toLocaleString", to_locale_string, func.clone(), realm).into();
        this_borrow.to_string =
            NativeFunction::with_proto("toString", to_string, func.clone(), realm).into();
        this_borrow.value_of = NativeFunction::with_proto("valueOf", value_of, func, realm).into();

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
    fn define_property(&self, name: InternalPropertyKey, value: Value, realm: &mut Realm) -> Res<DefinePropertyResult> {
        let mut this = self.inner.try_borrow_mut()?;

        if let InternalPropertyKey::String(name) = &name {
            match name.as_str() {
                "__defineGetter__" => {
                    this.defined_getter = value.into();
                    return Ok(DefinePropertyResult::Handled);
                }
                "__defineSetter__" => {
                    this.defined_setter = value.into();
                    return Ok(DefinePropertyResult::Handled);
                }

                "__lookupGetter__" => {
                    this.lookup_getter = value.into();
                    return Ok(DefinePropertyResult::Handled);
                }

                "__lookupSetter__" => {
                    this.lookup_setter = value.into();
                    return Ok(DefinePropertyResult::Handled);
                }

                "constructor" => {
                    this.constructor = value.into();
                    return Ok(DefinePropertyResult::Handled);
                }

                "hasOwnProperty" => {
                    this.has_own_property = value.into();
                    return Ok(DefinePropertyResult::Handled);
                }

                "getOwnPropertyDescriptor" => {
                    this.get_own_property_descriptor = value.into();
                    return Ok(DefinePropertyResult::Handled);
                }

                "isPrototypeOf" => {
                    this.is_prototype_of = value.into();
                    return Ok(DefinePropertyResult::Handled);
                }

                "propertyIsEnumerable" => {
                    this.property_is_enumerable = value.into();
                    return Ok(DefinePropertyResult::Handled);
                }

                "toLocaleString" => {
                    this.to_locale_string = value.into();
                    return Ok(DefinePropertyResult::Handled);
                }

                "toString" => {
                    this.to_string = value.into();
                    return Ok(DefinePropertyResult::Handled);
                }

                "valueOf" => {
                    this.value_of = value.into();
                    return Ok(DefinePropertyResult::Handled);
                }

                _ => {}
            }
        }

        this.object.define_property(name, value, realm)
    }

    fn define_property_attributes(&self, name: InternalPropertyKey, value: Variable, realm: &mut Realm) -> Res<DefinePropertyResult> {
        let mut this = self.inner.try_borrow_mut()?;

        this.object.define_property_attributes(name, value, realm)
    }

    fn resolve_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        let this = self.inner.try_borrow()?;

        if let InternalPropertyKey::String(ref name) = name {
            match name.as_str() {
                "__defineGetter__" => return Ok(Some(this.defined_getter.value.clone().into())),
                "__defineSetter__" => return Ok(Some(this.defined_setter.value.clone().into())),
                "__lookupGetter__" => return Ok(Some(this.lookup_getter.value.clone().into())),
                "__lookupSetter__" => return Ok(Some(this.lookup_setter.value.clone().into())),
                "constructor" => return Ok(Some(this.constructor.value.clone().into())),
                "hasOwnProperty" => return Ok(Some(this.has_own_property.value.clone().into())),
                "getOwnPropertyDescriptor" => {
                    return Ok(Some(this.get_own_property_descriptor.value.clone().into()))
                }
                "isPrototypeOf" => return Ok(Some(this.is_prototype_of.value.clone().into())),
                "propertyIsEnumerable" => return Ok(Some(this.property_is_enumerable.value.clone().into())),
                "toLocaleString" => return Ok(Some(this.to_locale_string.value.clone().into())),
                "toString" => return Ok(Some(this.to_string.value.clone().into())),
                "valueOf" => return Ok(Some(this.value_of.value.clone().into())),
                _ => {}
            }
        }
        this.object.resolve_property(name, realm)
    }

    fn get_own_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        let this = self.inner.try_borrow()?;

        if let InternalPropertyKey::String(ref name) = name {
            match name.as_str() {
                "__defineGetter__" => return Ok(Some(this.defined_getter.value.clone().into())),
                "__defineSetter__" => return Ok(Some(this.defined_setter.value.clone().into())),
                "__lookupGetter__" => return Ok(Some(this.lookup_getter.value.clone().into())),
                "__lookupSetter__" => return Ok(Some(this.lookup_setter.value.clone().into())),
                "constructor" => return Ok(Some(this.constructor.value.clone().into())),
                "hasOwnProperty" => return Ok(Some(this.has_own_property.value.clone().into())),
                "getOwnPropertyDescriptor" => {
                    return Ok(Some(this.get_own_property_descriptor.value.clone().into()))
                }
                "isPrototypeOf" => return Ok(Some(this.is_prototype_of.value.clone().into())),
                "propertyIsEnumerable" => return Ok(Some(this.property_is_enumerable.value.clone().into())),
                "toLocaleString" => return Ok(Some(this.to_locale_string.value.clone().into())),
                "toString" => return Ok(Some(this.to_string.value.clone().into())),
                "valueOf" => return Ok(Some(this.value_of.value.clone().into())),
                _ => {}
            }
        }

        this.object.get_own_property(name, realm)
    }

    fn define_getter(&self, name: InternalPropertyKey, value: ObjectHandle, realm: &mut Realm) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        this.object.define_getter(name, value, realm)
    }

    fn define_setter(&self, name: InternalPropertyKey, value: ObjectHandle, realm: &mut Realm) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        this.object.define_setter(name, value, realm)
    }

    fn delete_property(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        if let InternalPropertyKey::String(ref name) = name {
            match name.as_str() {
                "__defineGetter__" => {
                    let mut this = self.inner.try_borrow_mut()?;
                    this.defined_getter = Value::Undefined.into();
                    return Ok(Some(Value::Undefined.into()));
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

        this.object.delete_property(name, realm)
    }

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        if let InternalPropertyKey::String(ref name) = name {
            if Self::DIRECT_PROPERTIES.contains(&name.as_str()) {
                return Ok(true);
            }
        }

        let mut this = self.inner.try_borrow_mut()?;

        this.object.contains_key(name, realm)
    }

    fn name(&self) -> String {
        "Object".to_string()
    }

    // fn to_string(&self, _realm: &mut Realm) -> Res<YSString, Error> {
    //     Ok("[object Object]".into())
    // }
    //
    // fn to_string_internal(&self) -> Res<YSString> {
    //     Ok("[object Prototype]".into())
    // }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>> {
        let this = self.inner.try_borrow()?;

        let mut props = this.object.properties(realm)?;
        props.push((
            PropertyKey::String("__defineGetter__".into()),
            this.defined_getter.value.copy(),
        ));
        props.push((
            PropertyKey::String("__defineSetter__".into()),
            this.defined_setter.value.copy(),
        ));
        props.push((
            PropertyKey::String("__lookupGetter__".into()),
            this.lookup_getter.value.copy(),
        ));
        props.push((
            PropertyKey::String("__lookupSetter__".into()),
            this.lookup_setter.value.copy(),
        ));
        props.push((PropertyKey::String("constructor".into()), this.constructor.value.copy()));
        props.push((
            PropertyKey::String("hasOwnProperty".into()),
            this.has_own_property.value.copy(),
        ));
        props.push((
            PropertyKey::String("getOwnPropertyDescriptor".into()),
            this.get_own_property_descriptor.value.copy(),
        ));
        props.push((
            PropertyKey::String("isPrototypeOf".into()),
            this.is_prototype_of.value.copy(),
        ));
        props.push((
            PropertyKey::String("propertyIsEnumerable".into()),
            this.property_is_enumerable.value.copy(),
        ));
        props.push((
            PropertyKey::String("toLocaleString".into()),
            this.to_locale_string.value.copy(),
        ));
        props.push((PropertyKey::String("toString".into()), this.to_string.value.copy()));
        props.push((PropertyKey::String("valueOf".into()), this.value_of.value.copy()));

        Ok(props)
    }

    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        let this = self.inner.try_borrow()?;

        let mut keys = this.object.keys(realm)?;

        for key in Self::DIRECT_PROPERTIES {
            keys.push(PropertyKey::String((*key).into()));
        }

        Ok(keys)
    }

    fn values(&self, realm: &mut Realm) -> Res<Vec<Value>> {
        let this = self.inner.try_borrow()?;

        let mut values = this.object.values(realm)?;

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

    fn get_array_or_done(&self, index: usize, realm: &mut Realm) -> Res<(bool, Option<Value>)> {
        let mut this = self.inner.try_borrow_mut()?;

        this.object.get_array_or_done(index, realm)
    }

    fn clear_properties(&self, realm: &mut Realm) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        this.object.clear_properties(realm)
    }

    fn prototype(&self, _: &mut Realm) -> Res<ObjectOrNull> {
        Ok(ObjectOrNull::Null)
    }

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {

        if let InternalPropertyKey::String(ref name) = name {
            if Self::DIRECT_PROPERTIES.contains(&name.as_str()) {
                return Ok(true);
            }
        }

        let mut this = self.inner.try_borrow_mut()?;

        this.object.contains_own_key(name, realm)
    }

    fn enumerable_properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
        let this = self.inner.try_borrow()?;

        let props = this.object.enumerable_properties(realm)?;

        Ok(props)
    }

    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        let this = self.inner.try_borrow()?;

        let mut keys = this.object.enumerable_keys(realm)?;

        Ok(keys)
    }

    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
        let this = self.inner.try_borrow()?;

        let values = this.object.enumerable_values(realm)?;

        Ok(values)
    }

    fn set_prototype(&self, prototype: ObjectOrNull, realm: &mut Realm) -> Res {
        let mut this = self.inner.try_borrow_mut()?;

        this.object.set_prototype(prototype, realm)
    }

    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        let this = self.inner.borrow();

        let mut refs = this.object.gc_refs();

        if let Some(obj) = this.defined_getter.value.gc_ref() {
            refs.push(obj);
        }

        if let Some(obj) = this.defined_setter.value.gc_ref() {
            refs.push(obj);
        }

        if let Some(obj) = this.lookup_getter.value.gc_ref() {
            refs.push(obj);
        }

        if let Some(obj) = this.lookup_setter.value.gc_ref() {
            refs.push(obj);
        }

        if let Some(obj) = this.constructor.value.gc_ref() {
            refs.push(obj);
        }

        if let Some(obj) = this.has_own_property.value.gc_ref() {
            refs.push(obj);
        }

        if let Some(obj) = this.get_own_property_descriptor.value.gc_ref() {
            refs.push(obj);
        }

        if let Some(obj) = this.is_prototype_of.value.gc_ref() {
            refs.push(obj);
        }

        if let Some(obj) = this.property_is_enumerable.value.gc_ref() {
            refs.push(obj);
        }

        if let Some(obj) = this.to_locale_string.value.gc_ref() {
            refs.push(obj);
        }

        if let Some(obj) = this.to_string.value.gc_ref() {
            refs.push(obj);
        }

        if let Some(obj) = this.value_of.value.gc_ref() {
            refs.push(obj);
        }

        refs
    }

    // fn constructor(&self) -> Res<ObjectProperty> {
    //     let this = self.inner.try_borrow()?;
    //
    //     Ok(this.constructor.clone())
    // }
}

impl Proto for Prototype {
    fn as_any(&mut self) -> &mut dyn Any {
        self
    }
}
