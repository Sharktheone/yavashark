use crate::NativeFunction;
use crate::object::Object;
use crate::Value;

pub fn get_console() -> Value {
    let mut console = Object::new();

    console.define_property(
        "log".into(),
        NativeFunction::new_boxed("log".to_string(), Box::new(|args| {
            let mut str = String::new();

            for arg in args {
                str.push_str(&arg.to_string());
                str.push(' ');
            }

            str.pop();

            println!("{}", str);

            Ok(Value::Undefined)
        })).into(),
    );


    console.into()
}
