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
        _locales: Option<Value>,
        _options: Option<Value>,
    ) -> Res<ObjectHandle> {
        Ok(Object::null())
    }

    #[get("compare")]
    fn compare(&self) -> i8 {
        0
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self) -> Res<ObjectHandle> {
        Ok(Object::null())
    }

    #[prop("supportedLocalesOf")]
    fn supported_locales_of(
        _locales: Option<Value>,
        _options: Option<Value>,
    ) -> Vec<String> {
        Vec::new()
    }
}
