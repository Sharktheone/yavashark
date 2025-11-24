pub use super::object_impl::*;
use super::{Attributes, IntoValue, ObjectOrNull, PrimitiveValue, Value, Variable};
use crate::error::Error;
use crate::value::property_key::IntoPropertyKey;
use crate::{
    GCd, InternalPropertyKey, ObjectHandle, PreHashedPropertyKey, PropertyKey, Realm, Res, Symbol,
    ValueResult,
};
use indexmap::Equivalent;
use std::any::TypeId;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
#[cfg(feature = "dbg_object_gc")]
use std::sync::atomic::AtomicIsize;
use yavashark_garbage::{Collectable, Gc, GcRef, Weak};
use yavashark_string::{ToYSString, YSString};
use crate::conversion::FromValueOutput;

#[derive(Debug, PartialEq, Eq)]
pub enum DefinePropertyResult {
    Handled,
    ReadOnly,
    Setter(ObjectHandle, Value),
}

#[derive(Debug, PartialEq, Eq)]
pub enum Property {
    Value(Value, Attributes),
    Getter(ObjectHandle, Attributes),
}

impl Default for Property {
    fn default() -> Self {
        Self::Value(Value::Undefined, Attributes::default())
    }
}

impl From<ObjectProperty> for Property {
    fn from(prop: ObjectProperty) -> Self {
        if !prop.set.is_undefined() || !prop.get.is_undefined() {
            Self::Getter(
                prop.get.to_object().unwrap_or(crate::Object::null()),
                prop.attributes,
            )
        } else {
            Self::Value(prop.value, prop.attributes)
        }
    }
}

impl From<Variable> for Property {
    fn from(var: Variable) -> Self {
        Self::Value(var.value, var.properties)
    }
}

impl From<Value> for Property {
    fn from(value: Value) -> Self {
        Self::Value(value, Attributes::default())
    }
}

impl From<PropertyDescriptor> for Property {
    fn from(desc: PropertyDescriptor) -> Self {
        match desc {
            PropertyDescriptor::Data {
                value,
                writable,
                enumerable,
                configurable,
            } => Self::Value(
                value,
                Attributes::from_values(writable, enumerable, configurable),
            ),
            PropertyDescriptor::Accessor {
                get,
                set: _,
                enumerable,
                configurable,
            } => Self::Getter(
                get.unwrap_or(crate::Object::null()),
                Attributes::from_values(false, enumerable, configurable),
            ),
        }
    }
}

impl Property {
    pub const fn attributes(&self) -> Attributes {
        match self {
            Property::Value(_, attributes) => *attributes,
            Property::Getter(_, a) => *a,
        }
    }

    pub const fn is_data(&self) -> bool {
        matches!(self, Property::Value(_, _))
    }

    pub fn assert_value(self) -> Variable {
        match self {
            Property::Value(v, attributes) => Variable::with_attributes(v, attributes),
            Property::Getter(_, attributes) => {
                Variable::with_attributes(Value::Undefined, attributes)
            }
        }
    }

    pub fn into_value(self, realm: &mut Realm) -> Res<Value> {
        match self {
            Property::Value(v, _) => Ok(v),
            Property::Getter(g, _) => g.call(Vec::new(), Value::Undefined, realm),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropertyDescriptor {
    Data {
        value: Value,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    },
    Accessor {
        get: Option<ObjectHandle>,
        set: Option<ObjectHandle>,
        enumerable: bool,
        configurable: bool,
    },
}

impl PropertyDescriptor {
    pub fn into_value(self, realm: &mut Realm) -> Res<Value> {
        let obj = crate::Object::new(realm);

        match self {
            PropertyDescriptor::Data {
                value,
                writable,
                enumerable,
                configurable,
            } => {
                obj.define_property("value".into(), value, realm)?;
                obj.define_property("writable".into(), writable.into(), realm)?;
                obj.define_property("enumerable".into(), enumerable.into(), realm)?;
                obj.define_property("configurable".into(), configurable.into(), realm)?;
            }
            PropertyDescriptor::Accessor {
                get,
                set,
                enumerable,
                configurable,
            } => {
                if let Some(get) = get {
                    obj.define_property("get".into(), get.clone().into(), realm)?;
                } else {
                    obj.define_property("get".into(), Value::Undefined, realm)?;
                }

                if let Some(set) = set {
                    obj.define_property("set".into(), set.clone().into(), realm)?;
                } else {
                    obj.define_property("set".into(), Value::Undefined, realm)?;
                }

                obj.define_property("enumerable".into(), enumerable.into(), realm)?;
                obj.define_property("configurable".into(), configurable.into(), realm)?;
            }
        }

        Ok(obj.into())
    }
}

impl FromValueOutput for PropertyDescriptor {
    type Output = Self;

    fn from_value_out(value: crate::Value, realm: &mut Realm) -> Res<Self::Output> {
        let descriptor = value
            .as_object()?;


        let enumerable = descriptor
            .resolve_property("enumerable", realm)?
            .map(|v| v.is_truthy())
            .unwrap_or(false);
        let configurable = descriptor
            .resolve_property("configurable", realm)?
            .map(|v| v.is_truthy())
            .unwrap_or(false);

        let value = descriptor.resolve_property("value", realm)?;

        let writable = descriptor
            .resolve_property("writable", realm)?
            .map(|v| v.is_truthy());



        let get = descriptor.resolve_property("get", realm)?;
        if let Some(get) = &get {
            if !get.is_callable() && !get.is_undefined() {
                return Err(crate::Error::ty("Getter must be a function or undefined"));
            }
        }


        let set = descriptor.resolve_property("set", realm)?;
        if let Some(set) = &set {
            if !set.is_callable() && !set.is_undefined() {
                return Err(crate::Error::ty("Setter must be a function or undefined"));
            }
        }



        if get.is_some() || set.is_some() {
            if value.is_some() {
                return Err(crate::Error::ty(
                    "Property descriptor cannot be both a data and an accessor descriptor",
                ));
            }

            if writable.is_some() {
                return Err(crate::Error::ty(
                    "Property descriptor cannot be both a data and an accessor descriptor",
                ));
            }
        }

        if value.is_some() {
            Ok(Self::Data {
                value: value.unwrap_or(Value::Undefined),
                writable: writable.unwrap_or(false),
                enumerable,
                configurable,
            })
        } else {
            Ok(Self::Accessor {
                get: get.and_then(|v| v.to_object().ok()),
                set: set.and_then(|v| v.to_object().ok()),
                enumerable,
                configurable,
            })
        }
    }
}

impl From<Value> for PropertyDescriptor {
    fn from(value: Value) -> Self {
        PropertyDescriptor::Data {
            value,
            writable: true,
            enumerable: true,
            configurable: true,
        }
    }
}

impl From<Property> for PropertyDescriptor {
    fn from(value: Property) -> Self {
        match value {
            Property::Value(v, a) => PropertyDescriptor::Data {
                value: v,
                writable: a.is_writable(),
                enumerable: a.is_enumerable(),
                configurable: a.is_configurable(),
            },
            Property::Getter(g, props) => PropertyDescriptor::Accessor {
                get: Some(g),
                set: None,
                enumerable: props.is_enumerable(),
                configurable: props.is_configurable(),
            },
        }
    }
}

impl From<&ObjectProperty> for PropertyDescriptor {
    fn from(prop: &ObjectProperty) -> Self {
        if !prop.set.is_undefined() || !prop.get.is_undefined() {
            PropertyDescriptor::Accessor {
                get: if let Value::Object(obj) = &prop.get {
                    Some(obj.clone())
                } else {
                    None
                },
                set: if let Value::Object(obj) = &prop.set {
                    Some(obj.clone())
                } else {
                    None
                },
                enumerable: prop.attributes.is_enumerable(),
                configurable: prop.attributes.is_configurable(),
            }
        } else {
            PropertyDescriptor::Data {
                value: prop.value.clone(),
                writable: prop.attributes.is_writable(),
                enumerable: prop.attributes.is_enumerable(),
                configurable: prop.attributes.is_configurable(),
            }
        }
    }
}

impl From<Variable> for PropertyDescriptor {
    fn from(var: Variable) -> Self {
        PropertyDescriptor::Data {
            value: var.value,
            writable: var.properties.is_writable(),
            enumerable: var.properties.is_enumerable(),
            configurable: var.properties.is_configurable(),
        }
    }
}


#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DefinePropertyDescriptor {
    Data {
        value: Value,
        writable: Option<bool>,
        enumerable: Option<bool>,
        configurable: Option<bool>,
    },
    Accessor {
        get: Option<ObjectHandle>,
        set: Option<ObjectHandle>,
        enumerable: Option<bool>,
        configurable: Option<bool>,
    },
}


impl FromValueOutput for DefinePropertyDescriptor {
    type Output = Self;

    fn from_value_out(value: crate::Value, realm: &mut Realm) -> Res<Self::Output> {
        let descriptor = value
            .as_object()?;


        let enumerable = descriptor
            .resolve_property("enumerable", realm)?
            .map(|v| v.is_truthy());
        let configurable = descriptor
            .resolve_property("configurable", realm)?
            .map(|v| v.is_truthy());

        let value = descriptor.resolve_property("value", realm)?;

        let writable = descriptor
            .resolve_property("writable", realm)?
            .map(|v| v.is_truthy());



        let get = descriptor.resolve_property("get", realm)?;
        if let Some(get) = &get {
            if !get.is_callable() && !get.is_undefined() {
                return Err(crate::Error::ty("Getter must be a function or undefined"));
            }
        }


        let set = descriptor.resolve_property("set", realm)?;
        if let Some(set) = &set {
            if !set.is_callable() && !set.is_undefined() {
                return Err(crate::Error::ty("Setter must be a function or undefined"));
            }
        }



        if get.is_some() || set.is_some() {
            if value.is_some() {
                return Err(crate::Error::ty(
                    "Property descriptor cannot be both a data and an accessor descriptor",
                ));
            }

            if writable.is_some() {
                return Err(crate::Error::ty(
                    "Property descriptor cannot be both a data and an accessor descriptor",
                ));
            }
        }

        if value.is_some() {
            Ok(Self::Data {
                value: value.unwrap_or(Value::Undefined),
                writable,
                enumerable,
                configurable,
            })
        } else {
            Ok(Self::Accessor {
                get: get.and_then(|v| v.to_object().ok()),
                set: set.and_then(|v| v.to_object().ok()),
                enumerable,
                configurable,
            })
        }
    }
}

pub trait Obj: Debug + 'static {
    fn define_property(
        &self,
        name: InternalPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult>;
    fn define_property_attributes(
        &self,
        name: InternalPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult>;

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>>;
    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>>;

    fn define_getter(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res;

    fn define_getter_attributes(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        attributes: Attributes,
        realm: &mut Realm,
    ) -> Res {
        _ = attributes;
        self.define_getter(name, callback, realm)
    }
    fn define_setter(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res;

    fn define_setter_attributes(
        &self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        attributes: Attributes,
        realm: &mut Realm,
    ) -> Res {
        _ = attributes;
        self.define_setter(name, callback, realm)
    }

    fn delete_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>>;

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool>;

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool>;

    fn define_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.define_property(name.0, value, realm)
    }
    fn define_property_attributes_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.define_property_attributes(name.0, value, realm)
    }

    fn resolve_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.resolve_property(name.0, realm)
    }
    fn get_own_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.get_own_property(name.0, realm)
    }

    fn define_getter_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.define_getter(name.0, callback, realm)
    }
    fn define_setter_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.define_setter(name.0, callback, realm)
    }

    fn delete_property_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.delete_property(name.0, realm)
    }

    fn contains_own_key_pre_hash(
        &self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<bool> {
        self.contains_own_key(name.0, realm)
    }

    fn contains_key_pre_hash(&self, name: PreHashedPropertyKey, realm: &mut Realm) -> Res<bool> {
        self.contains_key(name.0, realm)
    }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>>;
    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>>;
    fn values(&self, realm: &mut Realm) -> Res<Vec<Value>>;

    fn enumerable_properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>>;
    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>>;
    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<Value>>;

    fn clear_properties(&self, realm: &mut Realm) -> Res;

    fn get_array_or_done(&self, idx: usize, realm: &mut Realm) -> Res<(bool, Option<Value>)>;
    fn call(&self, args: Vec<Value>, this: Value, realm: &mut Realm) -> Res<Value> {
        _ = args;
        _ = this;
        _ = realm;
        Err(Error::ty_error(format!("{} is not callable", self.name())))
    }
    fn is_callable(&self) -> bool {
        false
    }

    fn primitive(&self, realm: &mut Realm) -> Res<Option<PrimitiveValue>> {
        _ = realm;
        Ok(None)
    }

    fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull>;
    fn set_prototype(&self, prototype: ObjectOrNull, realm: &mut Realm) -> Res;

    fn construct(&self, args: Vec<Value>, realm: &mut Realm) -> Res<ObjectHandle> {
        _ = args;
        _ = realm;
        //TODO: i think this somehow needs to work differently
        Err(Error::ty_error(format!(
            "{} is not constructable",
            self.class_name()
        )))
    }
    fn is_constructable(&self) -> bool {
        false
    }

    fn get_property_descriptor(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<PropertyDescriptor>> {
        let Some(prop) = self.get_own_property(name, realm)? else {
            return Ok(None);
        };

        match prop {
            Property::Value(v, a) => Ok(Some(PropertyDescriptor::Data {
                value: v,
                writable: a.is_writable(),
                enumerable: a.is_enumerable(),
                configurable: a.is_configurable(),
            })),
            Property::Getter(g, props) => Ok(Some(PropertyDescriptor::Accessor {
                get: Some(g),
                set: None,
                enumerable: props.is_enumerable(),
                configurable: props.is_configurable(),
            })),
        }
    }

    fn name(&self) -> String {
        "Object".to_string()
    }

    fn class_name(&self) -> &'static str {
        std::any::type_name::<Self>()
    }

    fn object_type_id(&self) -> TypeId {
        TypeId::of::<Self>()
    }

    /// # Safety
    /// - Caller and implementer must ensure that the pointer is a valid pointer to the type which the type id represents
    /// - Caller and implementer must ensure that the pointer is valid for the same lifetime of self
    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        if ty == TypeId::of::<Self>() {
            Some(NonNull::from(self).cast())
        } else {
            None
        }
    }

    unsafe fn inner_downcast_fat_ptr(&self, ty: TypeId) -> Option<NonNull<[()]>> {
        _ = ty;
        None
    }

    fn is_extensible(&self) -> bool {
        true
    }

    fn prevent_extensions(&self) -> Res {
        Ok(())
    }

    fn is_frozen(&self) -> bool {
        false
    }

    fn freeze(&self) -> Res {
        Ok(())
    }

    fn is_sealed(&self) -> bool {
        false
    }

    fn seal(&self) -> Res {
        Ok(())
    }

    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>>;

    fn into_object(self) -> Object
    where
        Self: Sized,
    {
        Object::from_boxed(Box::new(self))
    }

    // fn as_any_object_ref(&self) -> &(dyn std::any::Any + '_) {
    //     &self
    // }
}

pub trait MutObj: Debug + 'static {
    fn define_property(
        &mut self,
        name: InternalPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult>;
    fn define_property_attributes(
        &mut self,
        name: InternalPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult>;

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>>;
    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>>;

    fn define_getter(
        &mut self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res;
    fn define_setter(
        &mut self,
        name: InternalPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res;

    fn delete_property(
        &mut self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>>;

    fn contains_own_key(&mut self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool>;

    fn contains_key(&mut self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool>;

    fn define_property_pre_hash(
        &mut self,
        name: PreHashedPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.define_property(name.0, value, realm)
    }
    fn define_property_attributes_pre_hash(
        &mut self,
        name: PreHashedPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.define_property_attributes(name.0, value, realm)
    }

    fn resolve_property_pre_hash(
        &mut self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.resolve_property(name.0, realm)
    }
    fn get_own_property_pre_hash(
        &mut self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.get_own_property(name.0, realm)
    }

    fn define_getter_pre_hash(
        &mut self,
        name: PreHashedPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.define_getter(name.0, callback, realm)
    }
    fn define_setter_pre_hash(
        &mut self,
        name: PreHashedPropertyKey,
        callback: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.define_setter(name.0, callback, realm)
    }

    fn delete_property_pre_hash(
        &mut self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.delete_property(name.0, realm)
    }

    fn contains_own_key_pre_hash(
        &mut self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<bool> {
        self.contains_own_key(name.0, realm)
    }

    fn contains_key_pre_hash(
        &mut self,
        name: PreHashedPropertyKey,
        realm: &mut Realm,
    ) -> Res<bool> {
        self.contains_key(name.0, realm)
    }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>>;
    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>>;
    fn values(&self, realm: &mut Realm) -> Res<Vec<Value>>;

    fn enumerable_properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>>;
    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>>;
    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<Value>>;

    fn clear_properties(&mut self, realm: &mut Realm) -> Res;

    fn get_array_or_done(&mut self, idx: usize, realm: &mut Realm) -> Res<(bool, Option<Value>)>;
    fn call(&mut self, args: Vec<Value>, this: Value, realm: &mut Realm) -> Res<Value> {
        _ = args;
        _ = this;
        _ = realm;

        Err(Error::ty_error(format!(
            "{} is not callable",
            self.class_name()
        )))
    }
    fn is_callable(&self) -> bool {
        false
    }

    fn primitive(&mut self, realm: &mut Realm) -> Res<Option<PrimitiveValue>> {
        _ = realm;
        Ok(None)
    }

    fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull>;
    fn set_prototype(&mut self, prototype: ObjectOrNull, realm: &mut Realm) -> Res;

    fn construct(&mut self, args: Vec<Value>, realm: &mut Realm) -> Res<ObjectHandle> {
        _ = args;
        _ = realm;
        //TODO: i think this somehow needs to work differently
        Err(Error::ty_error(format!(
            "{} is not constructable",
            self.class_name()
        )))
    }
    fn is_constructable(&self) -> bool {
        false
    }

    fn get_property_descriptor(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<PropertyDescriptor>> {
        let Some(prop) = self.get_own_property(name, realm)? else {
            return Ok(None);
        };

        match prop {
            Property::Value(v, a) => Ok(Some(PropertyDescriptor::Data {
                value: v,
                writable: a.is_writable(),
                enumerable: a.is_enumerable(),
                configurable: a.is_configurable(),
            })),
            Property::Getter(g, props) => Ok(Some(PropertyDescriptor::Accessor {
                get: Some(g),
                set: None,
                enumerable: props.is_enumerable(),
                configurable: props.is_configurable(),
            })),
        }
    }

    fn class_name(&mut self) -> &'static str {
        std::any::type_name::<Self>()
    }

    /// # Safety
    /// - Caller and implementer must ensure that the pointer is a valid pointer to the type which the type id represents
    /// - Caller and implementer must ensure that the pointer is valid for the same lifetime of self
    unsafe fn inner_downcast(&mut self, ty: TypeId) -> Option<NonNull<()>> {
        if ty == TypeId::of::<Self>() {
            Some(NonNull::from(self).cast())
        } else {
            None
        }
    }

    unsafe fn inner_downcast_fat_ptr(&mut self, ty: TypeId) -> Option<NonNull<[()]>> {
        _ = ty;
        None
    }

    fn is_extensible(&self) -> bool {
        true
    }

    fn prevent_extensions(&mut self) -> Res {
        Ok(())
    }

    fn is_frozen(&self) -> bool {
        false
    }

    fn freeze(&mut self) -> Res {
        Ok(())
    }

    fn is_sealed(&self) -> bool {
        false
    }

    fn seal(&mut self) -> Res {
        Ok(())
    }

    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>>;
}
#[cfg(feature = "dbg_object_gc")]
pub struct ObjectCount(AtomicIsize);

#[cfg(feature = "dbg_object_gc")]
impl ObjectCount {
    const fn new() -> Self {
        Self(AtomicIsize::new(0))
    }

    fn increment(&self) {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
    }

    fn decrement(&self) {
        self.0.fetch_sub(1, std::sync::atomic::Ordering::SeqCst);
    }

    pub fn get(&self) -> isize {
        self.0.load(std::sync::atomic::Ordering::SeqCst)
    }
}

#[cfg(feature = "dbg_object_gc")]
pub static OBJECT_COUNT: ObjectCount = ObjectCount::new();
#[cfg(feature = "dbg_object_gc")]
pub static OBJECT_ALLOC: ObjectCount = ObjectCount::new();

#[repr(transparent)]
pub struct BoxedObj(Box<dyn Obj>);

impl Deref for BoxedObj {
    type Target = dyn Obj;

    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl DerefMut for BoxedObj {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

unsafe impl Collectable for BoxedObj {
    fn get_refs(&self) -> Vec<GcRef<Self>> {
        self.gc_refs()
    }

    #[cfg(any(feature = "obj_dbg", feature = "obj_trace"))]
    fn trace_name(&self) -> &'static str {
        self.0.class_name()
    }
}

impl BoxedObj {
    fn new(obj: Box<dyn Obj>) -> Self {
        #[cfg(feature = "dbg_object_gc")]
        {
            OBJECT_COUNT.increment();
            OBJECT_ALLOC.increment();
        }
        Self(obj)
    }

    #[allow(clippy::needless_lifetimes)]
    pub fn downcast<'a, T: 'static>(&'a self) -> Option<&'a T> {
        // Safety:
        // - we only interpret the returned pointer as T
        // - we only say the reference is valid for 'a this being the lifetime of self
        unsafe {
            let ptr = self.deref().inner_downcast(TypeId::of::<T>())?.cast();

            Some(ptr.as_ref())
        }
    }
}

#[derive(Clone)]
pub struct Object(Gc<BoxedObj>);

impl Deref for Object {
    type Target = Gc<BoxedObj>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Display for Object {
    /// This function shouldn't be used in production code, only for debugging
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[object {}]", self.name())
    }
}

impl ToYSString for Object {
    fn to_ys_string(&self) -> YSString {
        format!("[object {}]", self.name()).into()
    }
}

#[cfg(feature = "dbg_object_gc")]
impl Drop for BoxedObj {
    fn drop(&mut self) {
        OBJECT_COUNT.decrement();
    }
}

impl Debug for Object {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", *self)
    }
}

impl Hash for Object {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

impl Eq for Object {}

impl PartialEq for Object {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Object {
    pub fn resolve_property(
        &self,
        name: impl IntoPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Value>> {
        let Some(p) = self
            .0
            .resolve_property(name.into_internal_property_key(realm)?, realm)?
        else {
            return Ok(None);
        };

        match p {
            Property::Value(v, _) => Ok(Some(v)),
            Property::Getter(g, _) => g.call(Vec::new(), self.clone().into(), realm).map(Some),
        }
    }

    pub fn call_method(
        &self,
        name: impl IntoPropertyKey,
        realm: &mut Realm,
        args: Vec<Value>,
    ) -> ValueResult {
        let name = name.into_internal_property_key(realm)?;

        let method = self.resolve_property(name.clone(), realm)?.ok_or_else(|| {
            Error::reference_error(format!("Cannot call {} on {}", name, self.class_name()))
        })?;

        method.call(realm, args, self.clone().into())
    }
    pub fn resolve_property_no_get_set(
        &self,
        name: impl IntoPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        let name = name.into_internal_property_key(realm)?;

        self.0.resolve_property(name, realm)
    }

    pub fn get_own_property_no_get_set(
        &self,
        name: impl IntoPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        let name = name.into_internal_property_key(realm)?;
        self.0.get_own_property(name.clone(), realm)
    }

    pub fn get_own_property(&self, name: impl IntoPropertyKey, realm: &mut Realm) -> Res<Value> {
        let name = name.into_internal_property_key(realm)?;
        let property =
            self.0
                .get_own_property(name.clone(), realm)?
                .ok_or(Error::reference_error(format!(
                    "{name} does not exist on object"
                )))?;

        match property {
            Property::Value(v, _) => Ok(v),
            Property::Getter(g, _) => g.call(Vec::new(), self.clone().into(), realm),
        }
    }

    pub fn get_property_opt(
        &self,
        name: impl IntoPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Value>> {
        let name = name.into_internal_property_key(realm)?;
        let Some(prop) = self.0.resolve_property(name, realm)? else {
            return Ok(None);
        };

        match prop {
            Property::Value(v, _) => Ok(Some(v)),
            Property::Getter(g, _) => Ok(Some(g.call(Vec::new(), self.clone().into(), realm)?)),
        }
    }



    pub fn define_property_descriptor(&self, name: InternalPropertyKey, desc: PropertyDescriptor, realm: &mut Realm) -> Res<DefinePropertyResult> {
        match desc {
            PropertyDescriptor::Data { value, writable, enumerable, configurable } => {
                let attributes = Attributes::from_values(writable, enumerable, configurable);
                self.define_property_attributes(name, Variable::with_attributes(value, attributes), realm)
            },
            PropertyDescriptor::Accessor { get, set, enumerable, configurable } => {
                if let Some(getter) = get {
                    self.define_getter_attributes(name.clone(), getter, Attributes::from_values(false, enumerable, configurable), realm)?;
                }
                if let Some(setter) = set {
                    self.define_setter_attributes(name, setter, Attributes::from_values(false, enumerable, configurable), realm)?;
                }
                Ok(DefinePropertyResult::Handled)
            },
        }
    }


    pub fn define_descriptor(&self, name: InternalPropertyKey, desc: DefinePropertyDescriptor, realm: &mut Realm) -> Res<DefinePropertyResult> {
        match desc {
            DefinePropertyDescriptor::Data { value, writable, enumerable, configurable } => {

                let (writable, enumerable, configurable) = if writable.is_none() || enumerable.is_none() || configurable.is_none() {
                    let current = self.get_own_property_no_get_set(name.clone(), realm)?;
                    if let Some(Property::Value(_, attrs)) = current {
                        (
                            writable.unwrap_or(attrs.is_writable()),
                            enumerable.unwrap_or(attrs.is_enumerable()),
                            configurable.unwrap_or(attrs.is_configurable()),
                        )
                    } else {
                        (
                            writable.unwrap_or(false),
                            enumerable.unwrap_or(false),
                            configurable.unwrap_or(false),
                        )
                    }
                } else {
                    (writable.unwrap_or(false), enumerable.unwrap_or(false), configurable.unwrap_or(false))
                };


                let attributes = Attributes::from_values(writable, enumerable, configurable);
                self.define_property_attributes(name, Variable::with_attributes(value, attributes), realm)
            },
            DefinePropertyDescriptor::Accessor { get, set, enumerable, configurable } => {
                let (enumerable, configurable) = if enumerable.is_none() || configurable.is_none() {
                    let current = self.get_own_property_no_get_set(name.clone(), realm)?;
                    if let Some(Property::Getter(_, attrs)) = current {
                        (
                            enumerable.unwrap_or(attrs.is_enumerable()),
                            configurable.unwrap_or(attrs.is_configurable()),
                        )
                    } else {
                        (
                            enumerable.unwrap_or(false),
                            configurable.unwrap_or(false),
                        )
                    }
                } else {
                    (enumerable.unwrap_or(false), configurable.unwrap_or(false))
                };


                if let Some(getter) = get {
                    self.define_getter_attributes(name.clone(), getter, Attributes::from_values(false, enumerable, configurable), realm)?;
                }
                if let Some(setter) = set {
                    self.define_setter_attributes(name, setter, Attributes::from_values(false, enumerable, configurable), realm)?;
                }
                Ok(DefinePropertyResult::Handled)
            },
        }
    }

    #[must_use]
    pub fn name(&self) -> String {
        self.0.name()
    }

    #[must_use]
    pub fn id(&self) -> usize {
        self.0.ptr_id()
    }

    pub fn downcast<T: 'static>(&self) -> Option<GCd<T>> {
        self.get_owning().maybe_map(BoxedObj::downcast::<T>).ok()
    }

    pub fn set(
        &self,
        name: impl IntoPropertyKey,
        value: impl Into<Variable>,
        realm: &mut Realm,
    ) -> ValueResult {
        let name = name.into_internal_property_key(realm)?;
        let value = value.into();

        self.0
            .define_property_attributes(name, value, realm)
            .map(|_| Value::Undefined)
    }

    pub fn get(&self, name: impl IntoPropertyKey, realm: &mut Realm) -> ValueResult {
        let name = name.into_internal_property_key(realm)?;

        self.0
            .resolve_property(name, realm)?
            .map_or(Ok(Value::Undefined), |x| match x {
                Property::Value(v, _) => Ok(v),
                Property::Getter(g, _) => g.call(Vec::new(), self.clone().into(), realm),
            })
    }

    pub fn get_opt(&self, name: impl IntoPropertyKey, realm: &mut Realm) -> Res<Option<Value>> {
        let name = name.into_internal_property_key(realm)?;

        self.0
            .resolve_property(name, realm)?
            .map_or(Ok(None), |x| match x {
                Property::Value(v, _) => Ok(Some(v)),
                Property::Getter(g, _) => g.call(Vec::new(), self.clone().into(), realm).map(Some),
            })
    }

    pub fn to_primitive(&self, hint: Hint, realm: &mut Realm) -> ValueResult {
        if let Some(prim) = self.primitive(realm)? {
            return Ok(prim.into());
        }

        let to_prim = self.resolve_property(Symbol::TO_PRIMITIVE, realm)?;

        match to_prim {
            Some(Value::Object(to_prim)) => {
                return to_prim
                    .call(vec![hint.into_value()], self.clone().into(), realm)?
                    .assert_no_object();
            }
            Some(Value::Undefined | Value::Null) | None => {}
            Some(to_prim) => {
                return Err(Error::ty_error(format!(
                    "Symbol.toPrimitive must be a function, got {}",
                    to_prim.type_of()
                )))
            }
        }

        if hint == Hint::String {
            let to_string = self.resolve_property("toString", realm)?;

            if let Some(Value::Object(to_string)) = to_string {
                if to_string.is_callable() {
                    return to_string
                        .call(Vec::new(), self.clone().into(), realm)?
                        .assert_no_object();
                }
            }

            let to_value = self.resolve_property("valueOf", realm)?;

            if let Some(Value::Object(to_value)) = to_value {
                if to_value.is_callable() {
                    return to_value
                        .call(Vec::new(), self.clone().into(), realm)?
                        .assert_no_object();
                }
            }
        }

        let to_value = self.resolve_property("valueOf", realm)?;

        if let Some(Value::Object(to_value)) = to_value {
            if to_value.is_callable() {
                let val = to_value.call(Vec::new(), self.clone().into(), realm)?;

                if !val.is_object() {
                    return Ok(val);
                }
            }
        }

        let to_string = self.resolve_property("toString", realm)?;

        if let Some(Value::Object(to_string)) = to_string {
            if to_string.is_callable() {
                return to_string
                    .call(Vec::new(), self.clone().into(), realm)?
                    .assert_no_object();
            }
        }
        Err(Error::ty("Cannot convert object to primitive"))
    }

    pub fn enum_properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>> {
        self.0.enumerable_properties(realm)
    }

    #[must_use]
    pub fn downgrade(&self) -> WeakObject {
        WeakObject::new(self)
    }

    #[must_use]
    pub fn gc_ref(&self) -> Option<GcRef<BoxedObj>> {
        Some(self.get_ref())
    }
}

impl From<Box<dyn Obj>> for Object {
    fn from(obj: Box<dyn Obj>) -> Self {
        Self(Gc::new(BoxedObj::new(obj)))
    }
}

impl From<Gc<BoxedObj>> for Object {
    fn from(obj: Gc<BoxedObj>) -> Self {
        Self(obj)
    }
}

impl Object {
    #[must_use]
    pub fn from_boxed(obj: Box<dyn Obj>) -> Self {
        Self(Gc::new(BoxedObj::new(obj)))
    }

    pub fn new<O: Obj + 'static>(obj: O) -> Self {
        Self(Gc::new(BoxedObj::new(Box::new(obj))))
    }

    pub fn to_string(&self, realm: &mut Realm) -> Res<YSString> {
        let Some(to_string) = self.get_opt("toString", realm)? else {
            if let Some(to_string_tag) = self.get_opt(Symbol::TO_STRING_TAG, realm)? {
                return Ok(format!("[object {}]", to_string_tag.to_string(realm)?).into());
            }

            return Ok(format!("[object {}]", self.name()).into());
        };

        to_string
            .call(realm, vec![], self.clone().into())?
            .to_string(realm)
    }
}

#[derive(Clone)]
pub struct WeakObject(Weak<BoxedObj>);

impl WeakObject {
    #[must_use]
    pub fn new(obj: &Object) -> Self {
        Self(Gc::downgrade(&obj.0))
    }

    pub fn upgrade(&self) -> Option<Object> {
        self.0.upgrade().map(Object::from)
    }
}

impl Debug for WeakObject {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self.upgrade() {
            Some(obj) => write!(f, "WeakObject({obj:?})"),
            None => write!(f, "WeakObject(<dead>)"),
        }
    }
}

impl PartialEq for WeakObject {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl Equivalent<Object> for WeakObject {
    fn equivalent(&self, key: &Object) -> bool {
        self.0 == key.0
    }
}

impl Equivalent<WeakObject> for Object {
    fn equivalent(&self, key: &WeakObject) -> bool {
        self.0 == key.0
    }
}

impl Hash for WeakObject {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.as_ptr().hash(state);
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Hint {
    Number,
    String,
    None,
}

impl IntoValue for Hint {
    fn into_value(self) -> Value {
        match self {
            Self::Number => "number".into(),
            Self::String => "string".into(),
            Self::None => Value::Undefined,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct ObjectProperty {
    pub value: Value,
    pub attributes: Attributes,
    pub get: Value,
    pub set: Value,
}

impl ObjectProperty {
    #[must_use]
    pub const fn new(value: Value) -> Self {
        Self {
            value,
            attributes: Attributes::new(),
            get: Value::Undefined,
            set: Value::Undefined,
        }
    }

    #[must_use]
    pub const fn getter(value: Value) -> Self {
        Self {
            value: Value::Undefined,
            attributes: Attributes::config(),
            get: value,
            set: Value::Undefined,
        }
    }

    #[must_use]
    pub const fn setter(value: Value) -> Self {
        Self {
            value: Value::Undefined,
            attributes: Attributes::config(),
            get: Value::Undefined,
            set: value,
        }
    }

    pub fn get(self, this: Value, realm: &mut Realm) -> ValueResult {
        if self.get.is_nullish() {
            Ok(self.value)
        } else {
            self.get.call(realm, vec![], this)
        }
    }

    pub fn resolve(&self, this: Value, realm: &mut Realm) -> ValueResult {
        if self.get.is_nullish() {
            Ok(self.value.copy())
        } else {
            self.get.call(realm, vec![], this)
        }
    }

    #[must_use]
    pub fn copy(&self) -> Self {
        Self {
            value: self.value.copy(),
            attributes: self.attributes,
            get: self.get.copy(),
            set: self.set.copy(),
        }
    }

    pub fn descriptor(self, obj: &Object, realm: &mut Realm) -> Res {
        if !self.set.is_undefined() || !self.get.is_undefined() {
            obj.set("get", self.get, realm)?;
            obj.set("set", self.set, realm)?;
        } else {
            obj.set("value", self.value, realm)?;
        }

        obj.set("writable", self.attributes.is_writable(), realm)?;
        obj.set("enumerable", self.attributes.is_enumerable(), realm)?;
        obj.set("configurable", self.attributes.is_configurable(), realm)?;

        Ok(())
    }

    pub fn property(&self) -> Property {
        if !self.set.is_undefined() || !self.get.is_undefined() {
            let Ok(obj) = self.get.clone().to_object() else {
                return Property::Value(Value::Undefined, Attributes::config());
            };

            Property::Getter(obj, self.attributes)
        } else {
            Property::Value(self.value.clone(), self.attributes)
        }
    }
}

impl From<Variable> for ObjectProperty {
    fn from(v: Variable) -> Self {
        Self {
            value: v.value,
            attributes: v.properties,
            get: Value::Undefined,
            set: Value::Undefined,
        }
    }
}

// impl<C: Ctx> From<Value> for ObjectProperty {
//     fn from(v: Value) -> Self {
//         Self {
//             value: v,
//             attributes: Attributes::new(),
//             get: Value::Undefined,
//             set: Value::Undefined,
//         }
//     }
// }

impl<V: Into<Value>> From<V> for ObjectProperty {
    fn from(v: V) -> Self {
        Self {
            value: v.into(),
            attributes: Attributes::new(),
            get: Value::Undefined,
            set: Value::Undefined,
        }
    }
}
