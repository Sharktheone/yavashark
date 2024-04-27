use crate::{Function, Object, Value};

pub fn get_console() -> Value {
    let mut console = Object::new();


    console.define_property("log".to_string(), Function::native_obj(Box::new(|args, _| {
        for arg in args {
            print!("{}", arg);
        }

        println!();

        Ok(Value::Undefined)
    })).into());
}