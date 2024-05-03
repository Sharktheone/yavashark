use crate::{Error, NativeFunction, Value};

use crate::context::Context;
use crate::Function;

pub fn get_error(ctx: &mut Context) -> Value {
    NativeFunction::new(
        "error",
        |args, _| {
            let message = args.first().map(|v| Some(v.to_string())).unwrap_or(None);

            Ok(message.unwrap_or("<error>".to_string()).into()) //TODO: Error object
        },
        ctx,
    )
    .into()
}
