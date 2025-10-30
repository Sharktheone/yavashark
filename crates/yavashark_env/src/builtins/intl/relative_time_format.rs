use crate::array::Array;
use crate::value::Obj;
use crate::{MutObject, Object, ObjectHandle, Realm, Res};
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct RelativeTimeFormat {}

impl RelativeTimeFormat {
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableRelativeTimeFormat {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .intl_relative_time_format
                        .get(realm)?
                        .clone(),
                ),
            }),
        })
    }
}

#[props(intrinsic_name = intl_relative_time_format, to_string_tag = "Intl.RelativeTimeFormat")]
impl RelativeTimeFormat {
    #[constructor]
    fn construct(
        _locales: Option<String>,
        _options: Option<ObjectHandle>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        Ok(Self::new(realm)?.into_object())
    }

    #[prop("supportedLocalesOf")]
    fn supported_locales_of(
        _locales: String,
        _options: Option<ObjectHandle>,
        realm: &mut Realm,
    ) -> Vec<String> {
        Vec::new()
    }

    fn format(&self, _duration: ObjectHandle) -> String {
        String::new()
    }

    #[prop("formatToParts")]
    fn format_to_parts(&self, _duration: ObjectHandle, realm: &mut Realm) -> Vec<String> {
        Vec::new()
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, realm: &Realm) -> ObjectHandle {
        Object::new(realm)
    }
}
