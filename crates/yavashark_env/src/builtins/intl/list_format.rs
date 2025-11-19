use crate::array::Array;
use crate::value::Obj;
use crate::{MutObject, Object, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{data_object, object, props};

#[data_object]
pub enum LocaleMatcher {
    Lookup,
    #[name("best fit")]
    BestFit,
}

#[data_object]
pub enum Type {
    Conjunction,
    Disjunction,
    Unit,
}

#[data_object]
pub enum Style {
    Long,
    Short,
    Narrow,
}

#[data_object]
pub struct ListFormatOptions {
    #[prop("localeMatcher")]
    pub locale_matcher: Option<LocaleMatcher>,
    #[prop("type")]
    pub type_: Option<Type>,
    pub style: Option<Style>,
}

#[object]
#[derive(Debug)]
pub struct ListFormat {}

impl ListFormat {
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableListFormat {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .intl_list_format
                        .get(realm)?
                        .clone(),
                ),
            }),
        })
    }
}

#[props(intrinsic_name = intl_list_format, to_string_tag = "Intl.ListFormat")]
impl ListFormat {
    #[constructor]
    fn construct(
        _locales: Option<String>,
        _options: Option<ListFormatOptions>,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        Ok(Self::new(realm)?.into_object())
    }

    #[prop("supportedLocalesOf")]
    fn supported_locales_of(
        _locales: String,
        _options: Option<ObjectHandle>,
    ) -> Vec<String> {
        Vec::new()
    }

    fn format(&self, _list: ObjectHandle) -> String {
        String::new()
    }

    #[prop("formatToParts")]
    fn format_to_parts(&self, _list: ObjectHandle, realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Array::from_realm(realm)?.into_object())
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, realm: &Realm) -> ObjectHandle {
        Object::new(realm)
    }
}
