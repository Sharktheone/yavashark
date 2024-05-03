use crate::context::Context;
use crate::NativeFunction;
use crate::object::Object;
use crate::Value;

pub fn get_console(ctx: &mut Context) -> Value {
    let mut console = Object::new(ctx);

    console.define_property(
        "log".into(),
        NativeFunction::new("log", |args, _| {
            let mut str = String::new();

            for arg in args {
                str.push_str(&arg.to_string());
                str.push(' ');
            }

            str.pop();

            println!("{}", str);

            Ok(Value::Undefined)
        }, ctx).into(),
    );


    console.into()
}
