use std::collections::HashMap;

use yavashark_macro::{object, properties};
use yavashark_value::{ConstructValue, Func};

use crate::{Context, Error, Object, Value, ValueResult};

#[object(function, custom_constructor)]
#[derive(Debug)]
pub struct Class {
    pub private_props: HashMap<String, Value>,
    #[gc]
    pub prototype: Value,
}

impl Func<Context> for Class {
    fn call(
        &mut self,
        _ctx: &mut Context,
        _args: Vec<Value>,
        _this: Value,
    ) -> Result<Value, Error> {
        Err(Error::new(
            "Class constructor cannot be invoked without 'new'",
        ))
    }
}


impl ConstructValue<Context> for Class {
    fn get_constructor_value(&self, ctx: &mut Context) -> Option<yavashark_value::Value<Context>> {
        todo!()
    }
}

impl Class {
    #[must_use]
    pub fn new(ctx: &Context) -> Self {
        Self::new_with_proto(ctx.proto.func.clone().into())
    }

    #[must_use]
    pub fn new_with_proto(proto: Value) -> Self {
        let object = Object::raw_with_proto(proto);

        Self {
            object,
            private_props: HashMap::new(),
            prototype: Value::Undefined,
        }
    }

    pub fn set_private_prop(&mut self, key: String, value: Value) {
        self.private_props.insert(key, value);
    }

    #[must_use]
    pub fn get_private_prop(&self, key: &str) -> Option<&Value> {
        self.private_props.get(key)
    }
}



#[properties]
impl Class {
    #[constructor(raw)]
    pub fn construct(args: Vec<Value>, this: Value, ctx: &mut Context) -> ValueResult {
        Ok(Value::Undefined)
    }
}


#[object]
#[derive(Debug)]
pub struct ClassInstance {
    pub(crate) private_props: HashMap<String, Value>,
}


impl ClassInstance {
    #[must_use]
    pub fn new(ctx: &Context) -> Self {
        Self {
            private_props: HashMap::new(),
            object: Object::raw(ctx),
        }
    }

    pub fn set_private_prop(&mut self, key: String, value: Value) {
        self.private_props.insert(key, value);
    }

    #[must_use]
    pub fn get_private_prop(&self, key: &str) -> Option<&Value> {
        self.private_props.get(key)
    }
}