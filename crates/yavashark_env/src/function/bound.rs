use crate::value::{Constructor, Func};
use crate::{Error, MutObject, ObjectHandle, Realm, Value, ValueResult, Variable};
use std::cell::RefCell;
use yavashark_macro::object;

#[object(function, constructor)]
#[derive(Debug)]
pub struct BoundFunction {
    #[gc]
    func: Value,
    #[gc]
    bound_this: Value,
    // #[gc]
    bound_args: Vec<Value>, //TODO: this is a memleak!
}

impl Func for BoundFunction {
    fn call(&self, realm: &mut Realm, args: Vec<Value>, _this: Value) -> ValueResult {
        let args = if self.bound_args.is_empty() {
            args
        } else {
            let mut bound_args = self.bound_args.clone();
            bound_args.extend(args);
            bound_args
        };

        self.func.call(realm, args, self.bound_this.copy())
    }
}

impl Constructor for BoundFunction {
    fn construct(&self, realm: &mut Realm, args: Vec<Value>) -> ValueResult {
        self.func.as_object()?.construct(realm, args)
    }
}

impl BoundFunction {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(func: Value, this: Value, args: Vec<Value>, realm: &Realm) -> ValueResult {
        let f = func.as_object()?;

        if !f.is_function() {
            return Err(Error::ty("Function.bind must be called on a function"));
        }

        let length = f
            .get_property_opt(&"length".into())?
            .unwrap_or(Value::Undefined.into());

        let length = Variable::config(length.value);

        let obj = ObjectHandle::new(Self {
            func,
            inner: RefCell::new(MutableBoundFunction {
                object: MutObject::with_proto(realm.intrinsics.func.clone()),
            }),
            bound_this: this,
            bound_args: args,
        });

        obj.define_variable("length".into(), length)?;

        Ok(obj.into())
    }
}
