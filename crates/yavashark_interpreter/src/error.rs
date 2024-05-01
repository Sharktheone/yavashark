use crate::Error;

use crate::{Function, Value};

pub fn get_error() -> Value {
    Function::native_val(Box::new(|args| {
        let message = args.first()
            .map(|v| Some(v.to_string()))
            .unwrap_or(None);

        Ok(Error::unknown(message).into())
    }))
}