use std::cell::RefCell;
use crate::{Error, MutObject, Object, ObjectHandle, Realm, Value, ValueResult};
use yavashark_macro::object;
use yavashark_value::Func;

#[object(function)]
#[derive(Debug)]
pub struct BoundFunction {
    #[gc]
    func: Value,
    #[gc]
    bound_this: Value,
    // #[gc]
    // bound_args: Vec<Value>, TODO: we currently can't have a Vec<Value> in a #[object]
}

impl Func<Realm> for BoundFunction {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        self.func.call(realm, args, self.bound_this.copy())
    }
}

impl BoundFunction {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(func: Value, this: Value, _: Vec<Value>, realm: &Realm) -> ValueResult {
        let f = func.as_object()?;

        if !f.is_function() {
            return Err(Error::ty("Function.bind must be called on a function"));
        }

        Ok(ObjectHandle::new(Self {
            func,
            inner: RefCell::new(MutableBoundFunction { 
                object: MutObject::with_proto(realm.intrinsics.func.clone().into()),
            }),
            bound_this: this,
        })
        .into())
    }
}
