use crate::array::Array;
use crate::value::IntoValue;
use crate::{NativeFunction, ObjectHandle, Realm};

pub fn get_get_canonical_locales(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "getCanonicalLocales",
        |args, _, realm| {
            let _ = args;

            Ok(Array::from_realm(realm)?.into_value())
        },
        realm,
        1,
    )
}
