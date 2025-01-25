
use std::cell::RefCell;
use yavashark_macro::{object, properties_new};
use yavashark_value::{Constructor, Func, Obj};
use crate::{MutObject, Object, ObjectHandle, Realm, Value, ValueResult};
use crate::builtins::{StringConstructor, StringObj};

#[object]
#[derive(Debug)]
pub struct BooleanObj {
    #[mutable]
    #[primitive]
    boolean: bool,
}

#[object(constructor, function)]
#[derive(Debug)]
pub struct BooleanConstructor {}

impl BooleanConstructor {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(_: &Object, func: &Value) -> crate::Result<ObjectHandle> {
        let mut this = Self {
            inner: RefCell::new(MutableBooleanConstructor {
                object: MutObject::with_proto(func.copy()),
            }),
        };

        Ok(this.into_object())
    }
}

impl Constructor<Realm> for BooleanConstructor {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        let boolean = args.first().map_or(false, Value::is_truthy);

        let obj = BooleanObj::new(realm, boolean)?;

        Ok(obj.into())
    }
}
impl Func<Realm> for BooleanConstructor {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let boolean = args.first().map_or(false, Value::is_truthy);

        Ok(boolean.into())
    }
}

impl BooleanObj {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(realm: &mut Realm, boolean: bool) -> ValueResult {
        let mut this = Self {
            inner: RefCell::new(MutableBooleanObj {
                object: MutObject::with_proto(realm.intrinsics.boolean.clone().into()),
                boolean,
            }),
        };

        Ok(this.into_object().into())
    }
}

#[properties_new(constructor(BooleanConstructor::new))]
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

