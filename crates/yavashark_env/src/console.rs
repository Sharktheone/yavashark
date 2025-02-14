use crate::console::print::PrettyPrint;
use crate::object::Object;
use crate::realm::Realm;
use crate::NativeFunction;
use crate::Value;

mod error;
pub mod print;

#[must_use]
pub fn get_console(realm: &Realm) -> Value {
    let console = Object::new(realm);

    let _ = console.define_property(
        "log".into(),
        NativeFunction::new(
            "log",
            |args, _, _| {
                let mut str = String::new();

                for arg in args {
                    str.push_str(&arg.pretty_print());
                    str.push(' ');
                }

                str.pop();

                println!("{str}");

                Ok(Value::Undefined)
            },
            realm,
        )
        .into(),
    ); // This can only fail if we have an existing borrow to the object, which we clearly don't

    console.into()
}
