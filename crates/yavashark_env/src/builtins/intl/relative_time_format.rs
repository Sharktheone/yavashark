use crate::array::Array;
use crate::value::Obj;
use crate::{MutObject, Object, ObjectHandle, Realm, Res};
use std::cell::RefCell;
use yavashark_macro::{data_object, object, props};
use crate::builtins::intl::utils::{LocaleMatcher, Style};

#[data_object]
pub enum Numeric {
    Always,
    Auto,
}
#[data_object]
pub struct RelativeTimeFormatOptions {
    #[prop("localeMatcher")]
    pub locale_matcher: Option<LocaleMatcher>,
    pub numeric: Option<Numeric>,
    pub style: Option<Style>,
}

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
        _options: Option<RelativeTimeFormatOptions>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        Ok(Self::new(realm)?.into_object())
    }

    fn supported_locales_of(_locales: String, _options: Option<ObjectHandle>) -> Vec<String> {
        Vec::new()
    }

    fn format(&self, _value: f32, _unit: &str) -> String {
        String::new()
    }

    #[prop("formatToParts")]
    fn format_to_parts(&self, _value: f32, _unit: &str, realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Array::from_realm(realm)?.into_object())
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, realm: &Realm) -> ObjectHandle {
        Object::new(realm)
    }
}
