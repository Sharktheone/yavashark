use crate::console::print::PrettyPrint;
use crate::object::Object;
use crate::realm::Realm;
use crate::NativeFunction;
use crate::Value;

mod error;
pub mod print;
pub mod sink;

#[must_use]
pub fn get_console(realm: &mut Realm) -> Value {
    let console = Object::new(realm);

    let _ = console.define_property(
        "log".into(),
        NativeFunction::new(
            "log",
            |args, _, realm| {
                let mut str = String::new();

                for arg in args {
                    str.push_str(&arg.pretty_print(realm));
                    str.push(' ');
                }

                str.pop();

                if !sink::call_log_sink(&str) {
                    #[cfg(not(target_arch = "wasm32"))]
                    println!("{str}");
                    #[cfg(target_arch = "wasm32")]
                    log::info!("YAVASHARK_LOG: {str}");
                }

                Ok(Value::Undefined)
            },
            realm,
        )
        .into(),
        realm,
    ); // This can only fail if we have an existing borrow to the object, which we clearly don't

    let _ = console.define_property(
        "printNativeStacktrace".into(),
        NativeFunction::new(
            "printNativeStacktrace",
            |_, _, _| {
                let bt = std::backtrace::Backtrace::force_capture();
                eprintln!("{bt}");
                Ok(Value::Undefined)
            },
            realm,
    )
        .into(),
        realm,
    );

    console.into()
}
