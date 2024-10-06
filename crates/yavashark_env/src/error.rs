use crate::context::Context;
use crate::{Error, NativeFunction, Object, ObjectHandle, Result, Value};
use yavashark_macro::object;

#[must_use]
pub fn get_error(ctx: &Context) -> Value {
    NativeFunction::special(
        "error",
        |args, this, ctx| {
            let message = args
                .first()
                .map_or(String::new(), std::string::ToString::to_string);

            let err = ErrorObj::raw_from(message, ctx);

            this.exchange(Box::new(err))?;

            Ok(Value::Undefined)
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
    #[allow(clippy::new_ret_no_self)]
    #[must_use]
    pub fn new(error: Error, ctx: &Context) -> ObjectHandle {
        let this = Self {
            object: Object::raw(ctx),
            error,
        };

        ObjectHandle::new(this)
    }

    #[must_use]
    pub fn new_from(message: String, ctx: &Context) -> ObjectHandle {
        let this = Self {
            object: Object::raw(ctx),
            error: Error::unknown_error(message),
        };

        ObjectHandle::new(this)
    }

    #[must_use]
    pub fn raw_from(message: String, ctx: &Context) -> Self {
        Self {
            object: Object::raw(ctx),
            error: Error::unknown_error(message),
        }
    }

    pub fn override_to_string(&self, _: &mut Context) -> Result<String> {
        Ok(self.error.to_string())
    }

    #[must_use]
    pub fn override_to_string_internal(&self) -> String {
        self.error.to_string()
    }
}
