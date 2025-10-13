use crate::{NativeFunction, ObjectHandle, Realm, Value};



pub fn get_supported_values_of(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "supportedValuesOf",
        |args, _, realm| {
            let _ = args;
            let _ = realm;

            Ok(Value::Undefined)
        },
        realm,
        1,
    )
}