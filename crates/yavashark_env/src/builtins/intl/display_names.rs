use crate::array::Array;
use crate::value::Obj;
use crate::{MutObject, Object, ObjectHandle, Realm, Res};
use std::cell::RefCell;
use yavashark_macro::{object, props};

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
    fn construct(_locales: String, _options: ObjectHandle, realm: &mut Realm) -> Res<ObjectHandle> {
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

    fn of(&self, _code: String) -> String {
        String::new()
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, realm: &Realm) -> ObjectHandle {
        Object::new(realm)
    }
}
