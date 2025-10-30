use crate::{MutObject, Object, ObjectHandle, Realm, Res, Value};
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct NumberFormat {
}

impl NumberFormat {
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableNumberFormat {
                object: MutObject::with_proto(
                    realm
                        .intrinsics
                        .clone_public()
                        .intl_number_format
                        .get(realm)?
                        .clone(),
                ),
            }),
        })
    }
}

#[props(intrinsic_name = intl_number_format, to_string_tag = "Intl.NumberFormat")]
impl NumberFormat {
    #[constructor]
    fn construct(
        _locales: Option<Value>,
        _options: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> Res<Self> {
        Self::new(realm)
    }

    #[get("format")]
    fn format(&self) -> Res<String> {
        Ok(String::new())
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Object::new(realm))
    }

    pub fn supported_locales_of(
        _locales: Value,
        _options: Option<ObjectHandle>,
    ) -> Vec<String> {
        Vec::new()
    }
}
