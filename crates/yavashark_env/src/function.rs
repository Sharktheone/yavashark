use crate::inline_props::{PropertiesHook, UpdatePropertyResult};
use crate::realm::Realm;
use crate::value::{Attributes, DefinePropertyResult, MutObj, Obj, ObjectImpl, Property, PropertyDescriptor};
use crate::{
    Error, InternalPropertyKey, MutObject, Object, ObjectHandle, ObjectOrNull, Res, Value,
    ValueResult, Variable,
};
pub use class::*;
pub use constructor::*;
pub use function_prototype::FunctionPrototype;
use std::cell::{Cell, RefCell, RefMut};
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use yavashark_macro::inline_props;

mod bound;
mod class;
mod constructor;
pub mod function_prototype;

type NativeFn = Box<dyn Fn(Vec<Value>, Value, &mut Realm) -> ValueResult>;

pub struct NativeFunctionBuilder(NativeFunction, bool);

#[inline_props]
pub struct NativeFunctionProps {
    #[readonly]
    #[no_enumerable]
    pub length: usize,
    #[readonly]
    #[no_enumerable]
    pub name: &'static str,
    pub constructor: Option<ObjectHandle>,
}

pub struct MutNativeFunction {
    pub object: MutObject,
}

pub struct NativeFunction {
    pub f: NativeFn,
    pub constructor: bool,
    inner: RefCell<MutNativeFunction>,
    pub props: NativeFunctionProps,
}

impl ObjectImpl for NativeFunction {
    type Inner = MutNativeFunction;

    fn get_wrapped_object(&self) -> impl DerefMut<Target = impl MutObj> {
        RefMut::map(self.inner.borrow_mut(), |inner| &mut inner.object)
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

    fn contains_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        Ok(self.props.contains_property(&name)?
            || self.get_wrapped_object().contains_key(name, realm)?)
    }

    fn contains_own_key(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<bool> {
        Ok(self.props.contains_property(&name)?
            || self.get_wrapped_object().contains_own_key(name, realm)?)
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

    fn call(&self, args: Vec<Value>, this: Value, realm: &mut Realm) -> ValueResult {
        (self.f)(args, this, realm)
    }

    fn is_callable(&self) -> bool {
        true
    }

    // fn to_string(&self, _: &mut Realm) -> Result<YSString, crate::error::Error> {
    //     Ok(format!("function {}() {{ [native code] }}", self.name).into())
    // }
    //
    // fn to_string_internal(&self) -> Result<YSString, crate::error::Error> {
    //     Ok(format!("function {}() {{ [native code] }}", self.name).into())
    // }

    fn construct(&self, args: Vec<Value>, realm: &mut Realm) -> Res<ObjectHandle> {
        if !self.constructor {
            return Err(Error::ty_error(format!(
                "{} is not a constructor",
                self.props.name
            )));
        }

        let proto = Obj::resolve_property(self, "prototype".into(), realm)?.map_or_else(
            || realm.intrinsics.func.clone().into(),
            |p| p.assert_value(),
        );

        let proto: ObjectOrNull = proto.value.try_into()?;

        let obj = Object::with_proto(proto).into();

        (self.f)(args, obj, realm)?.to_object()
    }

    fn is_constructable(&self) -> bool {
        self.constructor
    }

    fn name(&self) -> String {
        self.props.name.into()
    }

    fn get_property_descriptor(&self, name: InternalPropertyKey, realm: &mut Realm) -> Res<Option<PropertyDescriptor>> {
        Ok(match self.props.get_descriptor(&name, realm)? {
            Some(prop) => Some(prop),
            None => self.get_wrapped_object().get_property_descriptor(name, realm)?,
        })
    }
}

impl NativeFunction {
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn new_boxed(name: &'static str, f: NativeFn, realm: &mut Realm) -> ObjectHandle {
        let this = Self {
            f,
            constructor: false,

            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone()),
            }),
            props: NativeFunctionProps {
                length: 0,
                name,
                constructor: RefCell::new(None),
                __deleted_properties: Cell::new(0),
                __written_properties: Cell::new(0),
            },
        };

        let handle = ObjectHandle::new(this);

        #[allow(clippy::expect_used)]
        {
            let this = handle.downcast::<Self>().expect("unreachable");

            let mut ctor = this.props.constructor.borrow_mut();

            *ctor = Some(handle.clone());
        }

        handle
    }

    #[allow(clippy::new_ret_no_self, clippy::missing_panics_doc)]
    pub fn new(
        name: &'static str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        realm: &mut Realm,
    ) -> ObjectHandle {
        let this = Self {
            f: Box::new(f),
            constructor: false,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone()),
            }),
            props: NativeFunctionProps {
                length: 0,
                name,
                constructor: RefCell::new(None),
                __deleted_properties: Cell::new(0),
                __written_properties: Cell::new(0),
            },
        };

        let handle = ObjectHandle::new(this);

        #[allow(clippy::expect_used)]
        {
            let this = handle.downcast::<Self>().expect("unreachable");

            let mut ctor = this.props.constructor.borrow_mut();
            *ctor = Some(handle.clone());
        }

        handle
    }

    #[allow(clippy::new_ret_no_self, clippy::missing_panics_doc)]
    pub fn with_len(
        name: &'static str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        realm: &mut Realm,
        len: usize,
    ) -> ObjectHandle {
        let this = Self {
            f: Box::new(f),
            constructor: false,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone()),
            }),
            props: NativeFunctionProps {
                length: len,
                name,
                constructor: RefCell::new(None),
                __deleted_properties: Cell::new(0),
                __written_properties: Cell::new(0),
            },
        };

        let handle = ObjectHandle::new(this);
        #[allow(clippy::expect_used)]
        {
            let this = handle.downcast::<Self>().expect("unreachable");

            let mut ctor = this.props.constructor.borrow_mut();
            *ctor = Some(handle.clone());
        }

        handle
    }

    #[allow(clippy::new_ret_no_self, clippy::missing_panics_doc)]
    pub fn special(
        name: &'static str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        realm: &mut Realm,
    ) -> ObjectHandle {
        let this = Self {
            f: Box::new(f),
            constructor: true,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone()),
            }),
            props: NativeFunctionProps {
                length: 0,
                name,
                constructor: RefCell::new(None),
                __deleted_properties: Cell::new(0),
                __written_properties: Cell::new(0),
            },
        };

        let handle = ObjectHandle::new(this);

        #[allow(clippy::expect_used)]
        {
            let this = handle.downcast::<Self>().expect("unreachable");

            let mut ctor = this.props.constructor.borrow_mut();

            *ctor = Some(handle.clone());
        }

        handle
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn with_proto(
        name: &'static str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        proto: ObjectHandle,
        _realm: &mut Realm,
    ) -> ObjectHandle {
        let this = Self {
            f: Box::new(f),
            constructor: false,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(proto),
            }),
            props: NativeFunctionProps {
                length: 0,
                name,
                constructor: RefCell::new(None),
                __deleted_properties: Cell::new(0),
                __written_properties: Cell::new(0),
            },
        };

        let handle = ObjectHandle::new(this);

        #[allow(clippy::expect_used)]
        {
            let this = handle.downcast::<Self>().expect("unreachable");

            let mut ctor = this.props.constructor.borrow_mut();
            *ctor = Some(handle.clone());
        }

        handle
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn with_proto_and_len(
        name: &'static str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        proto: ObjectHandle,
        len: usize,
        realm: &mut Realm,
    ) -> ObjectHandle {
        let this = Self {
            f: Box::new(f),
            constructor: false,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(proto),
            }),
            props: NativeFunctionProps {
                length: len,
                name,
                constructor: RefCell::new(None),
                __deleted_properties: Cell::new(0),
                __written_properties: Cell::new(0),
            },
        };

        let handle = ObjectHandle::new(this);
        let _ =
            handle.define_property_attributes("name".into(), Variable::config(name.into()), realm);

        #[allow(clippy::expect_used)]
        {
            let this = handle.downcast::<Self>().expect("unreachable");

            let mut ctor = this.props.constructor.borrow_mut();
            *ctor = Some(handle.clone());
        }

        handle
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn special_with_proto(
        name: &'static str,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
        proto: ObjectHandle,
        _realm: &mut Realm,
    ) -> ObjectHandle {
        let this = Self {
            f: Box::new(f),
            constructor: true,
            inner: RefCell::new(MutNativeFunction {
                object: MutObject::with_proto(proto),
            }),
            props: NativeFunctionProps {
                length: 0,
                name,
                constructor: RefCell::new(None),
                __deleted_properties: Cell::new(0),
                __written_properties: Cell::new(0),
            },
        };

        let handle = ObjectHandle::new(this);

        #[allow(clippy::expect_used)]
        {
            let this = handle.downcast::<Self>().expect("unreachable");

            let mut ctor = this.props.constructor.borrow_mut();
            *ctor = Some(handle.clone());
        }

        handle
    }

    #[must_use]
    pub fn builder() -> NativeFunctionBuilder {
        NativeFunctionBuilder(
            Self {
                f: Box::new(|_, _, _| Ok(Value::Undefined)),
                constructor: false,
                inner: RefCell::new(MutNativeFunction {
                    object: MutObject::with_proto(None),
                }),
                props: NativeFunctionProps {
                    length: 0,
                    name: "",
                    constructor: RefCell::new(None),
                    __deleted_properties: Cell::new(0),
                    __written_properties: Cell::new(0),
                },
            },
            true,
        )
    }
}

impl NativeFunctionBuilder {
    #[must_use]
    pub const fn name(mut self, name: &'static str) -> Self {
        self.0.props.name = name;
        self
    }

    #[must_use]
    pub fn func(mut self, f: NativeFn) -> Self {
        self.0.f = f;
        self
    }

    #[must_use]
    pub fn boxed_func(
        mut self,
        f: impl Fn(Vec<Value>, Value, &mut Realm) -> ValueResult + 'static,
    ) -> Self {
        self.0.f = Box::new(f);
        self
    }

    /// Note: Overwrites a potential prototype that was previously set
    #[must_use]
    pub fn object(self, object: MutObject) -> Self {
        let mut inner = self.0.inner.borrow_mut();
        inner.object = object;

        drop(inner);

        self
    }

    /// Note: Overwrites a potential object that was previously set
    #[must_use]
    pub fn proto(self, proto: ObjectHandle) -> Self {
        let mut inner = self.0.inner.borrow_mut();

        inner.object.prototype = proto.into();
        drop(inner);

        self
    }

    /// Note: Overrides the prototype of the object
    #[must_use]
    pub fn context(self, realm: &Realm) -> Self {
        let mut inner = self.0.inner.borrow_mut();

        inner.object.prototype = realm.intrinsics.func.clone().into();
        drop(inner);

        self
    }

    #[must_use]
    pub const fn special_constructor(mut self, special: bool) -> Self {
        self.0.constructor = special;
        self
    }

    #[must_use]
    pub fn constructor(mut self, constructor: ObjectHandle) -> Self {
        let mut ctor = self.0.props.constructor.borrow_mut();
        *ctor = Some(constructor);

        drop(ctor);

        self.1 = false;
        self
    }

    /// Builds the function handle.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn build(self, _realm: &mut Realm) -> ObjectHandle {
        let handle = ObjectHandle::new(self.0);

        #[allow(clippy::expect_used)]
        {
            let this = handle.downcast::<NativeFunction>().expect("unreachable");

            if self.1 {
                let mut ctor = this.props.constructor.borrow_mut();
                *ctor = Some(handle.clone());
            }
        }

        handle
    }
}

impl Debug for NativeFunction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[Function: {}]", self.props.name)
    }
}
