use yavashark_macro::object;

use crate::context::Context;
use crate::{Error, NativeFunction, Value};

pub fn get_error(ctx: &Context) -> Value {
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

#[object]
#[derive(Debug)]
#[allow(dead_code)]
struct ErrorObj {
    error: Error,
}
