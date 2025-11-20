use crate::{Object, ObjectHandle, Realm, Res, Value};
use yavashark_macro::{data_object, object, props};
use crate::builtins::intl::utils::LocaleMatcherOptions;

#[data_object]
pub enum Usage {
    Sort,
    Search,
}

#[data_object]
pub enum Sensitivity {
    Base,
    Accent,
    Case,
    Variant,
}

#[data_object]
pub enum CaseFirst {
    Upper,
    Lower,
    False,
}

#[data_object]
pub struct CollatorOptions {
    pub usage: Option<Usage>,
    #[prop("localeMatcher")]
    pub locale_matcher: Option<String>,
    pub numeric: Option<bool>,
    #[prop("caseFirst")]
    pub case_first: Option<CaseFirst>,
    pub sensitivity: Option<Sensitivity>,
    pub collation: Option<String>,
    #[prop("ignorePunctuation")]
    pub ignore_punctuation: Option<bool>,
}

#[object]
#[derive(Debug)]
pub struct Collator {}

#[props(intrinsic_name = intl_collator, to_string_tag = "Intl.Collator")]
impl Collator {
    #[constructor]
    fn construct(
        _locales: Option<String>,
        _options: Option<CollatorOptions>,
        #[realm] _realm: &mut Realm,
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
    fn supported_locales_of(_locales: String, _options: Option<LocaleMatcherOptions>) -> Vec<String> {
        Vec::new()
    }
}
