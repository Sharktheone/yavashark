use crate::realm::Realm;
use crate::{Error, Object, ObjectHandle, ObjectProperty, Res, Value, ValueResult};
use rustc_hash::FxHashMap;
use std::any::TypeId;
use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::ptr::NonNull;
use yavashark_macro::properties;
use yavashark_string::YSString;
use yavashark_value::{ConstructorFn, Obj, Variable};

#[derive(Clone, Debug)]
pub enum PrivateMember {
    Field(Value),
    Method(Value),
    Accessor { get: Option<Value>, set: Option<Value> },
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
    pub prototype: RefCell<ObjectProperty>,
    // #[gc(untyped)]
    pub constructor: Option<Box<dyn ConstructorFn<Realm>>>,
}

impl Obj<Realm> for Class {
    fn define_property(&self, name: Value, value: Value) -> Res {
        if matches!(&name, Value::String(s) if s.as_str() == "prototype") {
            self.prototype.borrow_mut().value = value;
            Ok(())
        } else {
            self.inner.define_property(name, value)
        }
    }

    fn define_variable(&self, name: Value, value: Variable<Realm>) -> Res {
        if matches!(&name, Value::String(s) if s.as_str() == "prototype") {
            *self.prototype.borrow_mut() = value.into();

            Ok(())
        } else {
            self.inner.define_variable(name, value)
        }
    }

    fn resolve_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        if matches!(name, Value::String(s) if s.as_str() == "prototype") {
            Ok(Some(self.prototype.borrow().clone()))
        } else {
            self.inner.resolve_property_no_get_set(name)
        }
    }

    fn get_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        if matches!(name, Value::String(s) if s.as_str() == "prototype") {
            Ok(Some(self.prototype.borrow().clone()))
        } else {
            self.inner.get_property_opt(name)
        }
    }

    fn define_getter(&self, name: Value, value: Value) -> Res {
        if matches!(&name, Value::String(s) if s.as_str() == "prototype") {
            self.prototype.borrow_mut().get = value;
            Ok(())
        } else {
            self.inner.define_getter(name, value)
        }
    }

    fn define_setter(&self, name: Value, value: Value) -> Res {
        if matches!(&name, Value::String(s) if s.as_str() == "prototype") {
            self.prototype.borrow_mut().set = value;
            Ok(())
        } else {
            self.inner.define_setter(name, value)
        }
    }

    fn delete_property(&self, name: &Value) -> Res<Option<Value>> {
        if matches!(name, Value::String(s) if s.as_str() == "prototype") {
            Ok(None)
        } else {
            self.inner.delete_property(name)
        }
    }

    fn contains_key(&self, name: &Value) -> Res<bool> {
        if matches!(name, Value::String(s) if s.as_str() == "prototype") {
            Ok(true)
        } else {
            self.inner.contains_key(name)
        }
    }

    fn has_key(&self, name: &Value) -> Res<bool> {
        if matches!(name, Value::String(s) if s.as_str() == "prototype") {
            Ok(true)
        } else {
            self.inner.has_key(name)
        }
    }

    fn name(&self) -> String {
        self.name.borrow().clone()
    }

    fn to_string(&self, realm: &mut Realm) -> Res<YSString> {
        self.inner.to_string(realm)
    }

    fn to_string_internal(&self) -> Res<YSString> {
        self.inner.to_string_internal()
    }

    fn properties(&self) -> Res<Vec<(Value, Value)>> {
        let mut props = self.inner.properties()?;

        props.push((
            Value::String("prototype".into()),
            self.prototype.borrow().value.clone(),
        ));

        for (key, value) in &*self.private_props.try_borrow()? {
            props.push((Value::String(key.clone().into()), value.as_value()));
        }

        Ok(props)
    }

    fn keys(&self) -> Res<Vec<Value>> {
        let mut keys = self.inner.keys()?;

        keys.push(Value::String("prototype".into()));

        for key in self.private_props.try_borrow()?.keys() {
            keys.push(Value::String(key.clone().into()));
        }

        Ok(keys)
    }

    fn values(&self) -> Res<Vec<Value>> {
        let mut values = self.inner.values()?;

        values.push(self.prototype.borrow().value.clone());

        for value in self.private_props.try_borrow()?.values() {
            values.push(value.as_value());
        }

        Ok(values)
    }

    fn get_array_or_done(&self, index: usize) -> Res<(bool, Option<Value>)> {
        self.inner.get_array_or_done(index)
    }

    fn clear_values(&self) -> Res {
        self.inner.clear_values()
    }

    fn call(&self, _realm: &mut Realm, _args: Vec<Value>, _this: Value) -> ValueResult {
        Err(Error::new(
            "Class constructor cannot be invoked without 'new'",
        ))
    }

    fn is_function(&self) -> bool {
        true
    }

    fn prototype(&self) -> Res<ObjectProperty> {
        self.inner.prototype()
    }

    fn set_prototype(&self, proto: ObjectProperty) -> Res {
        self.inner.set_prototype(proto)
    }

    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        Ok(if let Some(constructor) = &self.constructor {
            let this = ClassInstance::new_with_proto(
                self.prototype.try_borrow()?.value.clone(),
                self.name.borrow().clone(),
            )
            .into_value();

            constructor.construct(args, this.copy(), realm)?;

            this
        } else if let Some(sup) = &self.sup {
            let c = sup.construct(realm, args)?.to_object()?;

            c.set_prototype(self.prototype.try_borrow()?.clone())?;

            ClassInstance {
                inner: RefCell::new(c),
                private_props: RefCell::new(HashMap::new()),
                name: self.name.borrow().clone(),
            }
            .into_value()
        } else {
            ClassInstance::new_with_proto(
                self.prototype.try_borrow()?.value.clone(),
                self.name.borrow().clone(),
            )
            .into_value()
        })
    }

    fn is_constructor(&self) -> bool {
        true
    }

    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        if ty == TypeId::of::<Self>() {
            Some(NonNull::from(self).cast())
        } else {
            self.inner.inner_downcast(ty)
        }
    }
}

impl Class {
    pub fn new(realm: &Realm, name: String) -> Res<Self> {
        Self::new_with_proto(realm.intrinsics.func.clone().into(), name)
    }

    pub fn new_with_proto(proto: Value, name: String) -> Res<Self> {
        let inner = Object::with_proto(proto);

        inner.define_variable("name".into(), Variable::write_config(name.clone().into()))?;

        Ok(Self {
            inner,
            sup: None,
            private_props: RefCell::new(FxHashMap::default()),
            constructor: None,
            name: RefCell::new(name),
            prototype: RefCell::new(ObjectProperty::new(Value::Undefined)),
        })
    }

    pub fn with_super(sup: ObjectHandle, name: String) -> Res<Self> {
        let inner = Object::with_proto(sup.clone().into());

        inner.define_variable("name".into(), Variable::write_config(name.clone().into()))?;

        Ok(Self {
            inner,
            sup: Some(sup),
            private_props: RefCell::new(FxHashMap::default()),
            constructor: None,
            name: RefCell::new(name),
            prototype: RefCell::new(ObjectProperty::new(Value::Undefined)),
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

    pub fn set_proto(&mut self, proto: ObjectProperty) -> Res<(), Error> {
        *self.prototype.try_borrow_mut()? = proto;

        Ok(())
    }

    pub fn set_constructor(&mut self, constructor: impl ConstructorFn<Realm> + 'static) {
        self.constructor = Some(Box::new(constructor));
    }

    pub fn update_name(&self, n: &str) -> Res {
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
                let name_prop = self.inner.get_property_opt(&"name".into())?;

                if name_prop.is_none_or(|p| p.value.is_string()) {
                    self.inner
                        .define_property("name".into(), YSString::from_ref(n).into())?;
                }
            }
        }

        Ok(())
    }
}

#[properties]
impl Class {
    #[constructor(raw)]
    pub fn construct(args: Vec<Value>, realm: &mut Realm) -> ValueResult {
        let this = Self::new(realm, "Class".to_string())?.into_value();

        if let Value::Object(o) = this.copy() {
            let deez = o.guard();
            let constructor = deez.constructor()?;
            drop(deez);
            let constructor = constructor.resolve(Value::Object(o), realm)?;

            constructor.call(realm, args, this)
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

impl Obj<Realm> for ClassInstance {
    fn define_property(&self, name: Value, value: Value) -> Res {
        self.inner.try_borrow()?.define_property(name, value)
    }

    fn define_variable(&self, name: Value, value: Variable<Realm>) -> Res {
        self.inner.try_borrow()?.define_variable(name, value)
    }

    fn resolve_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        self.inner.try_borrow()?.resolve_property_no_get_set(name)
    }

    fn get_property(&self, name: &Value) -> Res<Option<ObjectProperty>> {
        self.inner.try_borrow()?.get_property_opt(name)
    }

    fn define_getter(&self, name: Value, value: Value) -> Res {
        self.inner.try_borrow()?.define_getter(name, value)
    }

    fn define_setter(&self, name: Value, value: Value) -> Res {
        self.inner.try_borrow()?.define_setter(name, value)
    }

    fn delete_property(&self, name: &Value) -> Res<Option<Value>> {
        self.inner.try_borrow()?.delete_property(name)
    }

    fn contains_key(&self, name: &Value) -> Res<bool> {
        self.inner.try_borrow()?.contains_key(name)
    }

    fn has_key(&self, name: &Value) -> Res<bool> {
        self.inner.try_borrow()?.has_key(name)
    }

    fn name(&self) -> String {
        self.name.clone()
    }

    fn to_string(&self, realm: &mut Realm) -> Res<YSString> {
        Obj::to_string(&****self.inner.try_borrow()?, realm)
    }

    fn to_string_internal(&self) -> Res<YSString> {
        self.inner.try_borrow()?.to_string_internal()
    }

    fn properties(&self) -> Res<Vec<(Value, Value)>> {
        let mut props = self.inner.try_borrow()?.properties()?;

        for (key, value) in &*self.private_props.try_borrow()? {
            props.push((Value::String(key.clone().into()), value.as_value()));
        }

        Ok(props)
    }

    fn keys(&self) -> Res<Vec<Value>> {
        let mut keys = self.inner.try_borrow()?.keys()?;

        for key in self.private_props.try_borrow()?.keys() {
            keys.push(Value::String(key.clone().into()));
        }

        Ok(keys)
    }

    fn values(&self) -> Res<Vec<Value>> {
        let mut values = self.inner.try_borrow()?.values()?;

        for value in self.private_props.try_borrow()?.values() {
            values.push(value.as_value());
        }

        Ok(values)
    }

    fn get_array_or_done(&self, index: usize) -> Res<(bool, Option<Value>)> {
        self.inner.try_borrow()?.get_array_or_done(index)
    }

    fn clear_values(&self) -> Res {
        self.inner.try_borrow()?.clear_values()
    }

    fn prototype(&self) -> Res<ObjectProperty> {
        self.inner.try_borrow()?.prototype()
    }

    fn set_prototype(&self, proto: ObjectProperty) -> Res {
        self.inner.try_borrow()?.set_prototype(proto)
    }

    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        if ty == TypeId::of::<Self>() {
            Some(NonNull::from(self).cast())
        } else {
            self.inner.borrow().inner_downcast(ty)
        }
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
    pub fn new_with_proto(proto: Value, name: String) -> Self {
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

    pub fn get_private_prop(&self, key: &str) -> Res<Option<PrivateMember>> {
        let private_props = self.private_props.try_borrow()?;

        let mut prop = private_props.get(key).cloned();

        if prop.is_none() {
            let proto = self.inner.try_borrow()?.prototype()?.value;

            if let Some(class) = proto.downcast::<Self>()? {
                prop = class.get_private_prop(key)?;
            }
        }

        Ok(prop)
    }
}
