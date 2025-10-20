use crate::array::Array;
use crate::value::Obj;
use crate::{MutObject, Object, ObjectHandle, Realm, Res};
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct DateTimeFormat {}

impl DateTimeFormat {
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableDateTimeFormat {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .intl_date_time_format
                        .get(realm)?
                        .clone(),
                ),
            }),
        })
    }
}

#[props(intrinsic_name = intl_date_time_format, to_string_tag = "Intl.DateTimeFormat")]
impl DateTimeFormat {
    #[call_constructor]
    fn construct(
        _locales: Option<String>,
        _options: Option<ObjectHandle>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        Ok(Self::new(realm)?.into_object())
    }

    #[prop("supportedLocalesOf")]
    fn supported_locales_of(
        _locales: Option<String>,
        _options: Option<ObjectHandle>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        Ok(Array::from_realm(realm)?.into_object())
    }

    fn format(&self) -> String {
        String::new()
    }

    #[prop("formatRange")]
    fn format_range(&self, _start: String, _end: String) -> String {
        String::new()
    }

    #[prop("formatRangeToParts")]
    fn format_range_to_parts(
        &self,
        _start: String,
        _end: String,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        Ok(Array::from_realm(realm)?.into_object())
    }

    #[prop("formatToParts")]
    fn format_to_parts(&self, _date: String, realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Array::from_realm(realm)?.into_object())
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, realm: &Realm) -> ObjectHandle {
        Object::new(realm)
    }
}
