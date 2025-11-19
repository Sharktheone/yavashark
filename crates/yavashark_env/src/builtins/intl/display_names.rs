use crate::value::Obj;
use crate::{MutObject, Object, ObjectHandle, Realm, Res};
use std::cell::RefCell;
use yavashark_macro::{data_object, object, props};

#[data_object]
pub enum LocaleMatcher {
    Lookup,
    #[name("best fit")]
    BestFit,
}

#[data_object]
pub enum Style {
    Narrow,
    Short,
    Long,
}

#[data_object]
pub enum Type {
    Language,
    Region,
    Script,
    Currency,
    Calendar,
    DateTimeField,
}

#[data_object]
pub enum LanguageDisplay {
    Dialect,
    Standard,
}

#[data_object]
pub enum Fallback {
    Code,
    None,
}

#[data_object]
pub struct DisplayNamesOptions {
    #[prop("localeMatcher")]
    pub locale_matcher: Option<LocaleMatcher>,
    pub style: Option<Style>,
    #[prop("type")]
    pub type_: Option<Type>,
    #[prop("languageDisplay")]
    pub language_display: Option<LanguageDisplay>,
    pub fallback: Option<Fallback>,
}

#[object]
#[derive(Debug)]
pub struct DisplayNames {}

impl DisplayNames {
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableDisplayNames {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .intl_display_names
                        .get(realm)?
                        .clone(),
                ),
            }),
        })
    }
}

#[props(intrinsic_name = intl_display_names, to_string_tag = "Intl.DisplayNames")]
impl DisplayNames {
    #[constructor]
    fn construct(
        _locales: String,
        _options: DisplayNamesOptions,
        realm: &mut Realm,
    ) -> Res<ObjectHandle> {
        Ok(Self::new(realm)?.into_object())
    }

    #[prop("supportedLocalesOf")]
    fn supported_locales_of(_locales: String, _options: Option<ObjectHandle>) -> Vec<String> {
        Vec::new()
    }

    fn of(&self, _code: String) -> String {
        String::new()
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, realm: &Realm) -> ObjectHandle {
        Object::new(realm)
    }
}
