use crate::utils::ProtoDefault;
use crate::{MutObject, Object, ObjectHandle, Realm, Res, Value, ValueResult};
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_string::YSString;
use yavashark_value::{Constructor, CustomName, Func, Obj};

#[object(name)]
#[derive(Debug)]
pub struct BooleanObj {
    #[mutable]
    #[primitive]
    boolean: bool,
}

impl CustomName for BooleanObj {
    fn custom_name(&self) -> String {
        "Boolean".to_string()
    }
}

impl ProtoDefault for BooleanObj {
    fn proto_default(realm: &Realm) -> Self {
        Self {
            inner: RefCell::new(MutableBooleanObj {
                object: MutObject::with_proto(realm.intrinsics.boolean.clone().into()),
                boolean: false,
            }),
        }
    }

    fn null_proto_default() -> Self {
        Self {
            inner: RefCell::new(MutableBooleanObj {
                object: MutObject::null(),
                boolean: false,
            }),
        }
    }
}

#[object(constructor, function, to_string, name)]
#[derive(Debug)]
pub struct BooleanConstructor {}

impl CustomName for BooleanConstructor {
    fn custom_name(&self) -> String {
        "Boolean".to_string()
    }
}

impl BooleanConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Res<ObjectHandle> {
        let this = Self {
            inner: RefCell::new(MutableBooleanConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        Ok(this.into_object())
    }

    pub fn override_to_string(&self, _: &mut Realm) -> Res<YSString> {
        Ok("function Boolean() { [native code] }".into())
    }

    pub fn override_to_string_internal(&self) -> Res<YSString> {
        Ok("function Boolean() { [native code] }".into())
    }
}

impl Constructor<Realm> for BooleanConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let boolean = args.first().is_some_and(|v| v.is_object() || v.is_truthy());

        let obj = BooleanObj::new(realm, boolean);

        Ok(obj.into())
    }
}
impl Func<Realm> for BooleanConstructor {
    fn call(&self, _realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let boolean = args.first().is_some_and(|v| v.is_object() || v.is_truthy());

        Ok(boolean.into())
    }
}

impl BooleanObj {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &mut Realm, boolean: bool) -> ObjectHandle {
        Self {
            inner: RefCell::new(MutableBooleanObj {
                object: MutObject::with_proto(realm.intrinsics.boolean.clone().into()),
                boolean,
            }),
        }
        .into_object()
    }
}

#[properties_new(default_null(boolean), constructor(BooleanConstructor::new))]
impl BooleanObj {
    #[prop("valueOf")]
    fn value_of(&self) -> bool {
        let inner = self.inner.borrow();

        inner.boolean
    }

    #[prop("toString")]
    fn to_js_string(&self) -> String {
        let inner = self.inner.borrow();

        inner.boolean.to_string()
    }
}
