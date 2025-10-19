use crate::{Object, ObjectHandle, Realm, Res};

#[allow(unused_variables)]
pub trait Intrinsic {
    fn initialize(realm: &mut Realm) -> Res<ObjectHandle>;
    fn get_intrinsic(realm: &mut Realm) -> Res<ObjectHandle>;

    fn get_global(realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Object::null())
    }
}
