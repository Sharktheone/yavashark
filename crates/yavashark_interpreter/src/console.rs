mod print;

use crate::console::print::PrettyPrint;
use crate::context::Context;
use crate::object::Object;
use crate::NativeFunction;
use crate::Value;

pub fn get_console(ctx: &mut Context) -> Value {
    let mut console = Object::new(ctx);

    console.define_property(
        "log".into(),
        NativeFunction::new(
            "log",
            |args, _, ctx| {
                let mut str = String::new();

                for arg in args {
                    str.push_str(&arg.pretty_print(ctx));
                    str.push(' ');
                }

                str.pop();

                println!("{str}");

                Ok(Value::Undefined)
            },
            ctx,
        )
        .into(),
    );

    console.into()
}
