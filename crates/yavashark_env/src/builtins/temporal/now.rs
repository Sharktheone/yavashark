use yavashark_macro::{object, props};
use crate::{ObjectHandle, Realm, Res};
use crate::builtins::temporal::instant::Instant;

#[object]
#[derive(Debug)]
pub struct Now {}

#[props]
impl Now {
    fn instant(#[realm] realm: &Realm) -> Res<ObjectHandle> {
        Instant::now(realm)
    }
}
