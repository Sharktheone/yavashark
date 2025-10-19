use crate::array::Array;
use crate::value::Obj;
use crate::{MutObject, Object, ObjectHandle, Realm};
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct PluralRules {}

impl PluralRules {
    pub fn new(realm: &mut Realm) -> Self {
        Self {
            inner: RefCell::new(MutablePluralRules {
                object: MutObject::with_proto(realm.intrinsics.intl_plural_rules.clone()),
            }),
        }
    }
}

#[props(intrinsic_name = intl_plural_rules, to_string_tag = "Intl.PluralRules")]
impl PluralRules {
    #[constructor]
    fn construct(
        _locales: Option<String>,
        _options: Option<ObjectHandle>,
        realm: &mut Realm,
    ) -> ObjectHandle {
        Self::new(realm).into_object()
    }

    #[prop("supportedLocalesOf")]
    fn supported_locales_of(
        _locales: String,
        _options: Option<ObjectHandle>,
        realm: &Realm,
    ) -> ObjectHandle {
        Array::from_realm(realm).into_object()
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, realm: &Realm) -> ObjectHandle {
        Object::new(realm)
    }

    fn select(&self, _duration: ObjectHandle) -> String {
        String::new()
    }

    #[prop("selectRange")]
    fn select_range(&self, _duration: ObjectHandle, _realm: &Realm) -> String {
        String::new()
    }
}
