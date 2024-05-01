use crate::{Function, Object, Value};
use std::cell::RefCell;
use std::rc::Rc;

pub fn get_console() -> Value {
    let mut console = Object::new();

    console.define_property(
        "log".to_string(),
        Function::native_val(Box::new(|args | {
            let mut str = String::new();

            for arg in args {
                str.push_str(&arg.to_string());
                str.push(' ');
            }

            str.pop();

            println!("{}", str);

            Ok(Value::Undefined)
        })),
    );

    let console = Rc::new(RefCell::new(console));

    Value::Object(console)
}
