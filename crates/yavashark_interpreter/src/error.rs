use crate::{Error, NativeFunction, Value};

use crate::context::Context;
use crate::FunctionHandle;

pub fn get_error(ctx: &mut Context) -> Value {
    NativeFunction::new(
        "error",
        |args, _, _| {
            let message = args.first().map(std::string::ToString::to_string);

            Ok(message.unwrap_or("<error>".to_string()).into()) //TODO: Error object
        },
        ctx,
    )
    .into()
}
