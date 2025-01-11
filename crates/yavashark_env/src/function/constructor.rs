use std::fmt;
use std::fmt::{Debug, Formatter};

use yavashark_macro::object;
use yavashark_value::{Constructor, Func};

use crate::realm::Realm;
use crate::{Error, Object, ObjectHandle, ObjectProperty, Value, ValueResult};

type ValueFn = Box<dyn Fn(&mut Realm, &Value) -> Value>;

#[object(function, constructor, direct(constructor))]
pub struct NativeConstructor {
    /// The name of the constructor
    pub name: String,
    /// The function that is called when the constructor is called
    pub f: Box<dyn Fn() -> Value>,
    /// The function that returns the constructor value
    pub f_value: Option<ValueFn>,
    #[gc]
    /// The prototype of the constructor
    pub proto: Value,
    /// Can this constructor be called without `new`?
    pub special: bool,
}

impl Debug for NativeConstructor {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "NativeConstructor({})", self.name)
    }
}

impl Constructor<Realm> for NativeConstructor {
    fn get_constructor(&self) -> Result<ObjectProperty, Error> {
        Ok(self.constructor.clone())
    }

    fn special_constructor(&self) -> bool {
        self.special
    }

    fn value(&self, realm: &mut Realm) -> ValueResult {
        if let Some(f) = &self.f_value {
            return Ok(f(realm, &self.proto));
        }

        Ok(Object::with_proto(self.proto.clone()).into())
    }

    fn proto(&self, _realm: &mut Realm) -> ValueResult {
        Ok(self.proto.clone())
    }
}

impl Func<Realm> for NativeConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, this: Value) -> ValueResult {
        if self.special {
            (self.f)().call(realm, args, this.copy())?;

            Ok(Value::Undefined)
        } else {
            Err(Error::ty_error(format!(
                "Constructor {} requires 'new'",
                self.name
            )))
        }
    }
}

impl NativeConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(
        name: String,
        f: impl Fn() -> Value + 'static,
        value: Option<ValueFn>,
        realm: &Realm,
    ) -> ObjectHandle {
        Self::with_proto(
            name,
            f,
            value,
            realm.intrinsics.func.clone().into(),
            realm.intrinsics.func.clone().into(),
        )
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn with_proto(
        name: String,
        f: impl Fn() -> Value + 'static,
        value: Option<ValueFn>,
        proto: Value,
        self_proto: Value,
    ) -> ObjectHandle {
        let f_value = value.map(|f| Box::new(f) as ValueFn);

        let this = Self {
            name,
            f: Box::new(f),
            f_value,
            proto,
            special: false,
            object: Object::raw_with_proto(self_proto),
            constructor: Value::Undefined.into(),
        };

        let handle = ObjectHandle::new(this);

        #[allow(clippy::expect_used)]
        {
            let constructor = handle.clone();
            let mut this = handle.get();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            this.constructor = constructor.into();
        }

        handle
    }

    pub fn special(
        name: String,
        f: impl Fn() -> Value + 'static,
        value: Option<ValueFn>,
        realm: &Realm,
    ) -> ObjectHandle {
        Self::special_with_proto(
            name,
            f,
            value,
            realm.intrinsics.func.clone().into(),
            realm.intrinsics.func.clone().into(),
        )
    }

    #[allow(clippy::missing_panics_doc)]
    pub fn special_with_proto(
        name: String,
        f: impl Fn() -> Value + 'static,
        value: Option<ValueFn>,
        proto: Value,
        self_proto: Value,
    ) -> ObjectHandle {
        let f_value = value.map(|f| Box::new(f) as ValueFn);

        let this = Self {
            name,
            f: Box::new(f),
            f_value,
            proto,
            special: true,
            object: Object::raw_with_proto(self_proto),
            constructor: Value::Undefined.into(),
        };

        let handle = ObjectHandle::new(this);

        #[allow(clippy::expect_used)]
        {
            let constructor = handle.clone();
            let mut this = handle.get();

            let this = this.as_any();

            let this = this.downcast_ref::<Self>().expect("unreachable");

            this.constructor = constructor.into();
        }

        handle
    }
}
