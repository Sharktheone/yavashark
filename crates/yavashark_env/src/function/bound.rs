use crate::{Error, Object, ObjectHandle, Realm, Value, ValueResult};
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
    fn call(&mut self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        self.func.call(realm, args, self.bound_this.copy())
    }
}

impl BoundFunction {
    pub fn new(func: Value, this: Value, args: Vec<Value>, realm: &Realm) -> ValueResult {
        let f = func.as_object()?;

        if !f.is_function() {
            return Err(Error::ty("Function.bind must be called on a function"));
        }

        Ok(ObjectHandle::new(Self {
            func,
            object: Object::raw(realm),
            bound_this: this,
        })
        .into())
    }
}
