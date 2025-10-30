use crate::value::Obj;
use crate::{
    Object, ObjectHandle, Realm, Res, Value
};
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct Collator {}

#[props(intrinsic_name = intl_collator, to_string_tag = "Intl.Collator")]
impl Collator {
    #[constructor]
    fn construct(
        locales: Option<Value>,
        options: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        Ok(Object::null())
    }

    #[get("compare")]
    fn compare(&self, #[realm] realm: &mut Realm) -> i8 {
        0
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Object::null())
    }
}
