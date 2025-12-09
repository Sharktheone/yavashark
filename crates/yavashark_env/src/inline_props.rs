use crate::value::property_key::{InternalPropertyKey, PropertyKey};
use crate::value::{
    Attributes, BoxedObj, DefinePropertyResult, MutObj, ObjectImpl, Property, PropertyDescriptor,
};
use crate::{MutObject, ObjectHandle, ObjectOrNull, Realm, Res, Value, Variable};
use std::any::TypeId;
use std::cell::RefCell;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::ptr::NonNull;
use yavashark_garbage::GcRef;

pub enum UpdatePropertyResult {
    Handled,
    NotHandled(Value),
    Setter(ObjectHandle, Value),
    ReadOnly,
}

#[allow(unused_variables)]
pub trait PropertiesHook {
    fn set_property(
        &self,
        key: &InternalPropertyKey,
        value: Value,
        realm: &mut Realm,
    ) -> Res<UpdatePropertyResult> {
        Ok(UpdatePropertyResult::NotHandled(value))
    }
    fn get_property(&self, key: &InternalPropertyKey, realm: &mut Realm) -> Res<Option<Property>> {
        Ok(None)
    }

    fn contains_property(&self, key: &InternalPropertyKey) -> Res<bool> {
        Ok(false)
    }

    fn properties(&self, realm: &mut Realm) -> Res<impl Iterator<Item = (PropertyKey, Property)>> {
        Ok(::core::iter::empty())
    }
    fn keys(&self, realm: &mut Realm) -> Res<impl Iterator<Item = PropertyKey>> {
        Ok(::core::iter::empty())
    }
    fn values(&self, realm: &mut Realm) -> Res<impl Iterator<Item = Property>> {
        Ok(::core::iter::empty())
    }

    fn enumerable_properties(
        &self,
        realm: &mut Realm,
    ) -> Res<impl Iterator<Item = (PropertyKey, Property)>> {
        self.properties(realm)
    }
    fn enumerable_keys(&self, realm: &mut Realm) -> Res<impl Iterator<Item = PropertyKey>> {
        self.keys(realm)
    }
    fn enumerable_values(&self, realm: &mut Realm) -> Res<impl Iterator<Item = Property>> {
        self.values(realm)
    }
    fn delete_property(&self, key: &InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        Ok(false)
    }

    fn get_descriptor(
        &self,
        key: &InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<PropertyDescriptor>> {
        Ok(None)
    }

    fn gc_refs(&self) -> impl Iterator<Item = GcRef<BoxedObj>> {
        ::core::iter::empty()
    }
}

#[derive(Debug)]
pub struct InlineObject<P: ?Sized> {
    pub inner: RefCell<MutObject>,
    pub props: P,
}

impl<P: PropertiesHook + Debug + 'static> InlineObject<P> {
    pub const fn with_inner(props: P, inner: MutObject) -> Self {
        Self {
            props,
            inner: RefCell::new(inner),
        }
    }

    pub fn new(props: P, realm: &Realm) -> Self {
        Self {
            props,
            inner: RefCell::new(MutObject::new(realm)),
        }
    }

    pub fn with_proto(props: P, proto: impl Into<ObjectOrNull>) -> Self {
        Self {
            props,
            inner: RefCell::new(MutObject::with_proto(proto)),
        }
    }
}

impl<P: PropertiesHook + Debug + 'static> ObjectImpl for InlineObject<P> {
    type Inner = MutObject;

    fn get_wrapped_object(&self) -> impl DerefMut<Target = impl MutObj> {
        self.inner.borrow_mut()
    }

    fn get_inner(&self) -> impl Deref<Target = Self::Inner> {
        self.inner.borrow()
    }

    fn get_inner_mut(&self) -> impl DerefMut<Target = Self::Inner> {
        self.inner.borrow_mut()
    }
    fn define_property(
        &self,
        name: InternalPropertyKey,
        value: crate::value::Value,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        Ok(match self.props.set_property(&name, value, realm)? {
            UpdatePropertyResult::Handled => DefinePropertyResult::Handled,
            UpdatePropertyResult::NotHandled(value) => self
                .get_wrapped_object()
                .define_property(name, value, realm)?,
            UpdatePropertyResult::Setter(set, value) => DefinePropertyResult::Setter(set, value),
            UpdatePropertyResult::ReadOnly => DefinePropertyResult::ReadOnly,
        })
    }

    fn define_property_attributes(
        &self,
        name: InternalPropertyKey,
        value: crate::value::Variable,
        realm: &mut Realm,
    ) -> Res<DefinePropertyResult> {
        Ok(match self.props.set_property(&name, value.value, realm)? {
            UpdatePropertyResult::Handled => DefinePropertyResult::Handled,
            UpdatePropertyResult::NotHandled(v) => {
                self.get_wrapped_object().define_property_attributes(
                    name,
                    Variable::with_attributes(v, value.properties),
                    realm,
                )?
            }
            UpdatePropertyResult::Setter(set, value) => DefinePropertyResult::Setter(set, value),
            UpdatePropertyResult::ReadOnly => DefinePropertyResult::ReadOnly,
        })
    }

    fn resolve_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        Ok(match self.props.get_property(&name, realm)? {
            Some(prop) => Some(prop),
            None => self.get_wrapped_object().resolve_property(name, realm)?,
        })
    }

    fn get_own_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        Ok(match self.props.get_property(&name, realm)? {
            Some(prop) => Some(prop),
            None => self.get_wrapped_object().get_own_property(name, realm)?,
        })
    }

    fn delete_property(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<Property>> {
        if self.props.delete_property(&name, realm)? {
            return Ok(Some(Property::Value(
                Value::Undefined,
                Attributes::config(),
            )));
        }

        self.get_wrapped_object().delete_property(name, realm)
    }

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        Ok(self.props.contains_property(&name)?
            || self.get_wrapped_object().contains_own_key(name, realm)?)
    }

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        Ok(self.props.contains_property(&name)?
            || self.get_wrapped_object().contains_key(name, realm)?)
    }

    fn properties(&self, realm: &mut Realm) -> Res<Vec<(PropertyKey, Property)>> {
        let mut result = self.get_wrapped_object().properties(realm)?;

        let inline = self.props.properties(realm)?.collect::<Vec<_>>();

        for (key, prop) in inline {
            result.push((key, prop));
        }

        Ok(result)
    }

    fn keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        let mut result = self.get_wrapped_object().keys(realm)?;

        for key in self.props.keys(realm)? {
            result.push(key);
        }

        Ok(result)
    }

    fn values(&self, realm: &mut Realm) -> Res<Vec<Property>> {
        let mut result = self.get_wrapped_object().values(realm)?;

        let inline = self.props.values(realm)?.collect::<Vec<_>>();

        for prop in inline {
            result.push(prop);
        }

        Ok(result)
    }

    fn enumerable_properties(
        &self,
        realm: &mut Realm,
    ) -> Res<Vec<(PropertyKey, Property)>> {
        let mut result = self.get_wrapped_object().enumerable_properties(realm)?;

        let inline = self.props.enumerable_properties(realm)?.collect::<Vec<_>>();

        for (key, prop) in inline {
            result.push((key, prop));
        }

        Ok(result)
    }

    fn enumerable_keys(&self, realm: &mut Realm) -> Res<Vec<PropertyKey>> {
        let mut result = self.get_wrapped_object().enumerable_keys(realm)?;

        for key in self.props.enumerable_keys(realm)? {
            result.push(key);
        }

        Ok(result)
    }

    fn enumerable_values(&self, realm: &mut Realm) -> Res<Vec<Property>> {
        let mut result = self.get_wrapped_object().enumerable_values(realm)?;

        let inline = self.props.enumerable_values(realm)?.collect::<Vec<_>>();

        for prop in inline {
            result.push(prop);
        }

        Ok(result)
    }

    fn get_property_descriptor(
        &self,
        name: InternalPropertyKey,
        realm: &mut Realm,
    ) -> Res<Option<PropertyDescriptor>> {
        Ok(match self.props.get_descriptor(&name, realm)? {
            Some(desc) => Some(desc),
            None => self
                .get_wrapped_object()
                .get_property_descriptor(name, realm)?,
        })
    }
    fn gc_refs(&self) -> Vec<GcRef<BoxedObj>> {
        let mut inner_refs = self.get_inner().gc_refs();
        let props_refs = self.props.gc_refs();

        inner_refs.extend(props_refs);

        inner_refs
    }

    unsafe fn inner_downcast(&self, ty: TypeId) -> Option<NonNull<()>> {
        if ty == TypeId::of::<Self>() {
            return Some(NonNull::from(self).cast());
        }

        if ty == TypeId::of::<P>() {
            return Some(NonNull::from(&self.props).cast());
        }

        None
    }
}
