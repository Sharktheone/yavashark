use yavashark_macro::object;

use crate::context::Context;
use crate::{ControlFlow, Error, NativeFunction, Object, ObjectHandle, Result, Value};

pub fn get_error(ctx: &Context) -> Value {
    NativeFunction::special(
        "error",
        |args, this, ctx| {

            let Value::Object(obj) = this else {
                return Err(Error::ty_error("Error must be called as a constructor".to_string()));
            };


            let Ok(obj) = obj.get_mut() else {
                return Err(Error::ty_error("Error must be called as a constructor".to_string()));
            };

            let message = args.first().map(std::string::ToString::to_string).unwrap_or("".to_string());

            let err = ErrorObj::new_from(message, ctx);

            Ok(err.into())
        },
        ctx,
    )
    .into()
}

#[object(to_string)]
#[derive(Debug)]
#[allow(dead_code)]
pub struct ErrorObj {
    error: Error,
}


impl ErrorObj {
    pub fn new(error: Error, ctx: &Context) -> ObjectHandle {
        let this = Self {
            object: Object::raw(ctx),
            error,
        };

        ObjectHandle::new(this)
    }

    pub fn new_from(message: String, ctx: &Context) -> ObjectHandle {
        let this = Self {
            object: Object::raw(ctx),
            error: Error::unknown_error(message),
        };

        ObjectHandle::new(this)
    }
    
    pub fn override_to_string(&self, _: &mut Context) -> Result<String> {
        Ok(self.error.to_string())
    }
    
    pub fn override_to_string_internal(&self) -> String {
        self.error.to_string()
    }
}