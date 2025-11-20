use crate::builtins::intl::utils::HourCycle;
use crate::{MutObject, Realm, Res};
use std::cell::RefCell;
use yavashark_macro::{data_object, object, props};

#[data_object]
pub enum CaseFirst {
    Upper,
    Lower,
    False,
}

#[data_object]
pub struct LocaleOptions {
    pub language: Option<String>,
    pub script: Option<String>,
    pub region: Option<String>,
    pub calendar: Option<String>,
    pub collation: Option<String>,
    #[prop("hourCycle")]
    pub hour_cycle: Option<HourCycle>,
    #[prop("caseFirst")]
    pub case_first: Option<CaseFirst>,
    pub numeric: Option<bool>,
    #[prop("numberingSystem")]
    pub numbering_system: Option<String>,
}

#[object]
#[derive(Debug)]
pub struct Locale {}

impl Locale {
    #[allow(unused)]
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableLocale {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .intl_locale
                        .get(realm)?
                        .clone(),
                ),
            }),
        })
    }
}

#[props(intrinsic_name = intl_locale, to_string_tag = "Intl.Locale")]
impl Locale {
    #[constructor]
    fn construct(_tag: String, _options: Option<LocaleOptions>, realm: &mut Realm) -> Res<Self> {
        Self::new(realm)
    }

    #[get("baseName")]
    fn base_name(&self) -> String {
        String::new()
    }

    #[get("calendar")]
    fn calendar(&self) -> String {
        String::new()
    }

    #[get("caseFirst")]
    fn case_first(&self) -> String {
        String::new()
    }

    #[get("collation")]
    fn collation(&self) -> String {
        String::new()
    }

    #[get("hourCycle")]
    fn hour_cycle(&self) -> String {
        String::new()
    }

    #[get("language")]
    fn language(&self) -> String {
        String::new()
    }

    #[get("numberingSystem")]
    fn numbering_system(&self) -> String {
        String::new()
    }

    #[get("numeric")]
    fn numeric(&self) -> bool {
        false
    }

    #[get("region")]
    fn region(&self) -> String {
        String::new()
    }

    #[get("script")]
    fn script(&self) -> String {
        String::new()
    }

    #[prop("maximize")]
    fn maximize(&self, realm: &mut Realm) -> Res<Self> {
        Self::new(realm)
    }

    #[prop("minimize")]
    fn minimize(&self, realm: &mut Realm) -> Res<Self> {
        Self::new(realm)
    }

    #[prop("toString")]
    fn to_string(&self) -> String {
        String::new()
    }
}
