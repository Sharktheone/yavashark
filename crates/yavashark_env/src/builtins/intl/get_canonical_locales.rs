use crate::{NativeFunction, ObjectHandle, Realm, Value};

pub fn get_get_canonical_locales(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "getCanonicalLocales",
        |args, _, realm| {
            let _ = args;
            let _ = realm;

            Ok(Value::Undefined)
        },
        realm,
        1,
    )
}
