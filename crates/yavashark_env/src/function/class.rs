use crate::realm::Realm;
use crate::value::{BoxedObj, ConstructorFn, DefinePropertyResult, Obj, Property, Variable};
use crate::{
    Error, InternalPropertyKey, Object, ObjectHandle, ObjectOrNull, PropertyKey,
    Res, Value, ValueResult,
};
use rustc_hash::FxHashMap;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ops::Deref;
use std::ptr::NonNull;
use yavashark_garbage::GcRef;
use yavashark_macro::properties;
use yavashark_string::YSString;

#[derive(Clone, Debug)]
pub enum PrivateMember {
    Field(Value),
    Method(Value),
    Accessor {
        get: Option<Value>,
        set: Option<Value>,
    },
}

impl PrivateMember {
    fn as_value(&self) -> Value {
        match self {
            Self::Field(v) | Self::Method(v) => v.clone(),
            Self::Accessor { get, .. } => get.clone().unwrap_or(Value::Undefined),
        }
    }
}

// #[object(function, constructor, direct(prototype))]
#[derive(Debug)]
pub struct Class {
    pub inner: ObjectHandle,
    pub sup: Option<ObjectHandle>,

    pub private_props: RefCell<FxHashMap<String, PrivateMember>>,
    pub name: RefCell<String>,
    // #[gc(untyped)]
    pub prototype: RefCell<ObjectHandle>,
    // #[gc(untyped)]
    pub constructor: Option<Box<dyn ConstructorFn>>,
}

impl Obj for Class {
    fn define_property(
        &self,
        name: InternalPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        if matches!(&name, InternalPropertyKey::String(s) if s.as_str() == "prototype") {
            *self.prototype.borrow_mut() = value.to_object()?;
            Ok(DefinePropertyResult::Handled)
        } else {
            self.inner.define_property(name, value, realm)
        }
    }

    fn define_property_attributes(
        &self,
        name: InternalPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        if matches!(&name, InternalPropertyKey::String(s) if s.as_str() == "prototype") {
            *self.prototype.borrow_mut() = value.value.to_object()?;

            Ok(DefinePropertyResult::Handled)
        } else {
            self.inner.define_property_attributes(name, value, realm)
        }
    }

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if matches!(name, InternalPropertyKey::String(ref s) if s.as_str() == "prototype") {
            let val: Value = self.prototype.borrow().clone().into();
            Ok(Some(val.into()))
        } else {
            self.inner.deref().resolve_property(name, realm)
        }
    }

    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if matches!(name, InternalPropertyKey::String(ref s) if s.as_str() == "prototype") {
            let val: Value = self.prototype.borrow().clone().into();
            Ok(Some(val.into()))
        } else {
            self.inner.deref().get_own_property(name, realm)
        }
    }

    fn define_getter(
        &self,
        name: InternalPropertyKey,
        value: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        if matches!(&name, InternalPropertyKey::String(s) if s.as_str() == "prototype") {
            return Err(Error::new("Cannot set prototype property"));
        }

        self.inner.define_getter(name, value, realm)
    }

    fn define_setter(
        &self,
        name: InternalPropertyKey,
        value: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        if matches!(&name, InternalPropertyKey::String(ref s) if s.as_str() == "prototype") {
            return Err(Error::new("Cannot set prototype property"));
        }

        self.inner.define_setter(name, value, realm)
    }

    fn delete_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if matches!(name, InternalPropertyKey::String(ref s) if s.as_str() == "prototype") {
            return Ok(None);
        }

        self.inner.delete_property(name, realm)
    }

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        if matches!(name, InternalPropertyKey::String(ref s) if s.as_str() == "prototype") {
            Ok(true)
        } else {
            self.inner.contains_own_key(name, realm)
        }
    }

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        if matches!(name, InternalPropertyKey::String(ref s) if s.as_str() == "prototype") {
            Ok(true)
        } else {
            self.inner.contains_key(name, realm)
        }
    }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>> {
        let mut props = self.inner.properties(realm)?;

        props.push((
            PropertyKey::String("prototype".into()),
            self.prototype.borrow().clone().into(),
        ));

        for (key, value) in &*self.private_props.try_borrow()? {
            props.push((PropertyKey::String(key.clone().into()), value.as_value()));
            //TODO: is this correct?
        }

        Ok(props)
    }

    // fn to_string(&self, realm: &mut Realm) -> Res<YSString> {
    //     self.inner.to_string(realm)
    // }
    //
    // fn to_string_internal(&self) -> Res<YSString> {
    //     self.inner.to_string_internal()
    // }

    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        let mut keys = self.inner.keys(realm)?;

        keys.push(PropertyKey::String("prototype".into()));

        for key in self.private_props.try_borrow()?.keys() {
            keys.push(PropertyKey::String(key.clone().into())); //TODO: is this correct?
        }

        Ok(keys)
    }

    fn values(&self, realm: &mut Realm) -> Res<Vec<Value>> {
        let mut values = self.inner.values(realm)?;

        values.push(self.prototype.borrow().clone().into());

        for value in self.private_props.try_borrow()?.values() {
            values.push(value.as_value()); //TODO: is this correct?
        }

        Ok(values)
    }

    fn enumerable_properties(
        &self,
        realm: &mut Realm,
    ) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
        let mut props = self.inner.enumerable_properties(realm)?;

        props.push((
            PropertyKey::String("prototype".into()),
            self.prototype.borrow().clone().into(),
        ));

        for (key, value) in &*self.private_props.try_borrow()? {
            props.push((PropertyKey::String(key.clone().into()), value.as_value()));
            //TODO: is this correct?
        }

        Ok(props)
    }

    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        let mut keys = self.inner.enumerable_keys(realm)?;

        keys.push(PropertyKey::String("prototype".into()));

        for key in self.private_props.try_borrow()?.keys() {
            keys.push(PropertyKey::String(key.clone().into())); //TODO: is this correct?
        }

        Ok(keys)
    }

    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
        let mut values = self.inner.enumerable_values(realm)?;

        values.push(self.prototype.borrow().clone().into());

        for value in self.private_props.try_borrow()?.values() {
            values.push(value.as_value()); //TODO: is this correct?
        }

        Ok(values)
    }

    fn clear_properties(&self, realm: &mut Realm) -> Res {
        self.inner.clear_properties(realm)
    }

    fn get_array_or_done(&self, index: usize, realm: &mut Realm) -> Res<(bool, Option<Value>)> {
        self.inner.get_array_or_done(index, realm)
    }

    fn call(&self, _args: Vec<Value>, _this: Value, _realm: &mut Realm) -> ValueResult {
        Err(Error::new(
            "Class constructor cannot be invoked without 'new'",
        ))
    }

    fn is_callable(&self) -> bool {
        true
    }

    fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull> {
        self.inner.prototype(realm)
    }

    fn set_prototype(&self, proto: ObjectOrNull, realm: &mut Realm) -> Res {
        self.inner.set_prototype(proto, realm)
    }

    fn construct(&self, args: Vec<Value>, realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(if let Some(constructor) = &self.constructor {
            let this = ClassInstance::new_with_proto(
                self.prototype.try_borrow()?.clone(),
                self.name.borrow().clone(),
            )
            .into_value();

            constructor.construct(args, this.copy(), realm)?;

            this.to_object()?
        } else if let Some(sup) = &self.sup {
            let c = sup.construct(args, realm)?;

            c.set_prototype(self.prototype.try_borrow()?.clone().into(), realm)?;

            ClassInstance {
                inner: RefCell::new(c),
                private_props: RefCell::new(HashMap::new()),
                name: self.name.borrow().clone(),
            }
            .into_object()
        } else {
            ClassInstance::new_with_proto(
                self.prototype.try_borrow()?.clone(),
                self.name.borrow().clone(),
            )
            .into_object()
        })
    }

    fn is_constructable(&self) -> bool {
        true
    }

    fn name(&self) -> String {
        self.name.borrow().clone()
    }

    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        if ty == TypeId::of::<Self>() {
            Some(NonNull::from(self).cast())
        } else {
            self.inner.inner_downcast(ty)
        }
    }

    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        Vec::new() //TODO
    }
}

impl Class {
    pub fn new(realm: &mut Realm, name: String) -> Res<Self> {
        Self::new_with_proto(realm.intrinsics.func.clone().into(), name, realm)
    }

    pub fn new_with_proto(proto: ObjectHandle, name: String, realm: &mut Realm) -> Res<Self> {
        let inner = Object::with_proto(proto);

        inner.define_property_attributes(
            "name".into(),
            Variable::write_config(name.clone().into()),
            realm,
        )?;

        Ok(Self {
            inner,
            sup: None,
            private_props: RefCell::new(FxHashMap::default()),
            constructor: None,
            name: RefCell::new(name),
            prototype: RefCell::new(Object::null()),
        })
    }

    pub fn with_super(sup: ObjectHandle, name: String, realm: &mut Realm) -> Res<Self> {
        let inner = Object::with_proto(sup.clone());

        inner.define_property_attributes(
            "name".into(),
            Variable::write_config(name.clone().into()),
            realm,
        )?;

        Ok(Self {
            inner,
            sup: Some(sup),
            private_props: RefCell::new(FxHashMap::default()),
            constructor: None,
            name: RefCell::new(name),
            prototype: RefCell::new(Object::null()),
        })
    }

    fn insert_private_member(&self, key: String, member: PrivateMember) {
        self.private_props.borrow_mut().insert(key, member);
    }

    pub fn define_private_field(&self, key: String, value: Value) {
        self.insert_private_member(key, PrivateMember::Field(value));
    }

    pub fn define_private_method(&self, key: String, value: Value) {
        self.insert_private_member(key, PrivateMember::Method(value));
    }

    pub fn define_private_getter(&self, key: String, value: Value) -> Res {
        let mut props = self.private_props.borrow_mut();

        match props.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(PrivateMember::Accessor {
                    get: Some(value),
                    set: None,
                });

                Ok(())
            }
            Entry::Occupied(mut entry) => match entry.get_mut() {
                PrivateMember::Accessor { get, .. } => {
                    *get = Some(value);
                    Ok(())
                }
                _ => Err(Error::ty("Cannot redeclare private field as accessor")),
            },
        }
    }

    pub fn define_private_setter(&self, key: String, value: Value) -> Res {
        let mut props = self.private_props.borrow_mut();

        match props.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(PrivateMember::Accessor {
                    get: None,
                    set: Some(value),
                });

                Ok(())
            }
            Entry::Occupied(mut entry) => match entry.get_mut() {
                PrivateMember::Accessor { set, .. } => {
                    *set = Some(value);
                    Ok(())
                }
                _ => Err(Error::ty("Cannot redeclare private field as accessor")),
            },
        }
    }

    pub fn update_private_field(&self, key: &str, value: Value) {
        self.insert_private_member(key.to_string(), PrivateMember::Field(value));
    }

    #[must_use]
    pub fn get_private_prop(&self, key: &str) -> Option<PrivateMember> {
        self.private_props.borrow().get(key).cloned()
    }

    pub fn set_proto(&mut self, proto: ObjectHandle) -> Res<(), Error> {
        *self.prototype.try_borrow_mut()? = proto;

        Ok(())
    }

    pub fn set_constructor(&mut self, constructor: impl ConstructorFn + 'static) {
        self.constructor = Some(Box::new(constructor));
    }

    pub fn update_name(&self, n: &str, realm: &mut Realm) -> Res {
        let mut name = self.name.try_borrow_mut()?;

        if name.is_empty() {
            *name = n.to_owned();

            if let Some(obj) = self.inner.downcast::<Object>() {
                obj.inner_mut()?
                    .force_update_property_cb("name".into(), |v| {
                        if let Some(v) = v {
                            if !v.value.is_string() {
                                return None;
                            }
                        }

                        Some(YSString::from_ref(n).into())
                    })?;
            } else {
                let name_prop = self.inner.get_property_opt("name", realm)?;

                if name_prop.is_none_or(|p| p.is_string()) {
                    self.inner.define_property(
                        "name".into(),
                        YSString::from_ref(n).into(),
                        realm,
                    )?;
                }
            }
        }

        Ok(())
    }
}

#[properties]
impl Class {
    #[constructor(raw)]
    pub fn construct(args: Vec<Value>, realm: &mut Realm) -> Res<ObjectHandle> {
        let this = Self::new(realm, "Class".to_string())?.into_value();

        if let Value::Object(o) = this.copy() {
            let constructor = o.get("constructor", realm)?;

            constructor.call(realm, args, this)?.to_object()
        } else {
            Err(Error::ty("Class constructor called with invalid receiver"))
        }
    }
}

// #[object(name)]
#[derive(Debug)]
pub struct ClassInstance {
    pub inner: RefCell<ObjectHandle>,
    // #[mutable]
    pub(crate) private_props: RefCell<HashMap<String, PrivateMember>>,
    name: String,
}

impl Obj for ClassInstance {
    fn define_property(
        &self,
        name: InternalPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.inner.try_borrow()?.define_property(name, value, realm)
    }

    fn define_property_attributes(
        &self,
        name: InternalPropertyKey,
        value: Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        self.inner
            .try_borrow()?
            .define_property_attributes(name, value, realm)
    }

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.inner
            .try_borrow()?
            .resolve_property_no_get_set(name, realm)
    }

    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        Ok(self
            .inner
            .try_borrow()?
            .get_property_opt(name, realm)?
            .map(Into::into))
    }

    fn define_getter(
        &self,
        name: InternalPropertyKey,
        value: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.inner.try_borrow()?.define_getter(name, value, realm)
    }

    fn define_setter(
        &self,
        name: InternalPropertyKey,
        value: ObjectHandle,
        realm: &mut Realm,
    ) -> Res {
        self.inner.try_borrow()?.define_setter(name, value, realm)
    }

    fn delete_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        self.inner.try_borrow()?.delete_property(name, realm)
    }

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        self.inner.try_borrow()?.contains_own_key(name, realm)
    }

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        self.inner.try_borrow()?.contains_key(name, realm)
    }

    // fn to_string(&self, realm: &mut Realm) -> Res<YSString> {
    //     Obj::to_string(&****self.inner.try_borrow()?, realm)
    // }
    //
    // fn to_string_internal(&self) -> Res<YSString> {
    //     self.inner.try_borrow()?.to_string_internal()
    // }
    //
    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Value)>> {
        let mut props = self.inner.try_borrow()?.properties(realm)?;

        for (key, value) in &*self.private_props.try_borrow()? {
            props.push((PropertyKey::String(key.clone().into()), value.as_value()));
        }

        Ok(props)
    }

    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        let mut keys = self.inner.try_borrow()?.keys(realm)?;

        for key in self.private_props.try_borrow()?.keys() {
            keys.push(PropertyKey::String(key.clone().into()));
        }

        Ok(keys)
    }

    fn values(&self, realm: &mut Realm) -> Res<Vec<Value>> {
        let mut values = self.inner.try_borrow()?.values(realm)?;

        for value in self.private_props.try_borrow()?.values() {
            values.push(value.as_value());
        }

        Ok(values)
    }

    fn enumerable_properties(
        &self,
        realm: &mut Realm,
    ) -> Res<Vec<(PropertyKey, crate::value::Value)>> {
        self.inner.try_borrow()?.enumerable_properties(realm)
    }

    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        self.inner.try_borrow()?.enumerable_keys(realm)
    }

    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<crate::value::Value>> {
        self.inner.try_borrow()?.enumerable_values(realm)
    }

    fn clear_properties(&self, realm: &mut Realm) -> Res {
        self.inner.try_borrow()?.clear_properties(realm)
    }

    fn get_array_or_done(&self, index: usize, realm: &mut Realm) -> Res<(bool, Option<Value>)> {
        self.inner.try_borrow()?.get_array_or_done(index, realm)
    }

    fn prototype(&self, realm: &mut Realm) -> Res<ObjectOrNull> {
        self.inner.try_borrow()?.prototype(realm)
    }

    fn set_prototype(&self, proto: ObjectOrNull, realm: &mut Realm) -> Res {
        self.inner.try_borrow()?.set_prototype(proto, realm)
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        if ty == TypeId::of::<Self>() {
            Some(NonNull::from(self).cast())
        } else {
            self.inner.borrow().inner_downcast(ty)
        }
    }

    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        Vec::new() //TODO
    }
}

impl ClassInstance {
    #[must_use]
    pub fn new(realm: &Realm, name: String) -> Self {
        Self {
            inner: RefCell::new(Object::new(realm)),
            private_props: RefCell::new(HashMap::new()),
            name,
        }
    }

    #[must_use]
    pub fn new_with_proto(proto: ObjectHandle, name: String) -> Self {
        Self {
            inner: RefCell::new(Object::with_proto(proto)),
            private_props: RefCell::new(HashMap::new()),
            name,
        }
    }

    fn insert_private_member(&self, key: String, member: PrivateMember) {
        self.private_props.borrow_mut().insert(key, member);
    }

    pub fn define_private_field(&self, key: String, value: Value) {
        self.insert_private_member(key, PrivateMember::Field(value));
    }

    pub fn define_private_method(&self, key: String, value: Value) {
        self.insert_private_member(key, PrivateMember::Method(value));
    }

    pub fn define_private_getter(&self, key: String, value: Value) -> Res {
        let mut props = self.private_props.try_borrow_mut()?;

        match props.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(PrivateMember::Accessor {
                    get: Some(value),
                    set: None,
                });

                Ok(())
            }
            Entry::Occupied(mut entry) => match entry.get_mut() {
                PrivateMember::Accessor { get, .. } => {
                    *get = Some(value);
                    Ok(())
                }
                _ => Err(Error::ty("Cannot redeclare private field as accessor")),
            },
        }
    }

    pub fn define_private_setter(&self, key: String, value: Value) -> Res {
        let mut props = self.private_props.try_borrow_mut()?;

        match props.entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(PrivateMember::Accessor {
                    get: None,
                    set: Some(value),
                });

                Ok(())
            }
            Entry::Occupied(mut entry) => match entry.get_mut() {
                PrivateMember::Accessor { set, .. } => {
                    *set = Some(value);
                    Ok(())
                }
                _ => Err(Error::ty("Cannot redeclare private field as accessor")),
            },
        }
    }

    pub fn update_private_field(&self, key: &str, value: Value) {
        self.insert_private_member(key.to_string(), PrivateMember::Field(value));
    }

    pub fn get_private_prop(&self, key: &str, realm: &mut Realm) -> Res<Option<PrivateMember>> {
        let private_props = self.private_props.try_borrow()?;

        let mut prop = private_props.get(key).cloned();

        if prop.is_none() {
            let proto = self.inner.try_borrow()?.prototype(realm)?;

            let ObjectOrNull::Object(proto) = proto else {
                return Ok(prop);
            };

            if let Some(class) = proto.downcast::<Self>() {
                prop = class.get_private_prop(key, realm)?;
            }
        }

        Ok(prop)
    }
}
