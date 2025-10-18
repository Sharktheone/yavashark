use crate::{Error, NativeFunction, ObjectHandle, Realm, Res};

pub fn get_throw_type_error(
    realm: &mut Realm,
) -> Res<ObjectHandle> {
    let throw_type_error = NativeFunction::with_len("", |_, _, _| {
        Err(Error::ty(""))
    }, realm, 0);


    Ok(throw_type_error)
}