use crate::{Function};
use std::cell::RefCell;
use std::rc::Rc;
use crate::Value;
use crate::object::Object;

pub fn get_console() -> Value {
    let mut console = Object::new();

    console.define_property(
        "log".into(),
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


    console.into()
}
