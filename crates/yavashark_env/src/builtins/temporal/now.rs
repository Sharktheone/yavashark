use crate::builtins::temporal::instant::Instant;
use crate::{ObjectHandle, Realm, Res};
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct Now {}

#[props]
impl Now {
    fn instant(#[realm] realm: &Realm) -> Res<ObjectHandle> {
        Instant::now(realm)
    }
}
