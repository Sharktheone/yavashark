use crate::array::Array;
use crate::{NativeFunction, ObjectHandle, Realm};
use crate::value::IntoValue;

pub fn get_supported_values_of(realm: &mut Realm) -> ObjectHandle {
    NativeFunction::with_len(
        "supportedValuesOf",
        |args, _, realm| {
            let _ = args;

            Ok(Array::from_realm(realm)?.into_value())
        },
        realm,
        1,
    )
}
