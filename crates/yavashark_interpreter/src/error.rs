use crate::{Error, Value};

use crate::{Function};

pub fn get_error() -> Value {
    Function::native_val(Box::new(|args| {
        let message = args.first()
            .map(|v| Some(v.to_string()))
            .unwrap_or(None);

        Ok(message.unwrap_or("<error>".to_string()).into()) //TODO: Error object
    }))
}