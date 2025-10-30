use crate::array::Array;
use crate::builtins::intl::utils::{
    canonicalize_locale_list, get_option_string, validate_currency_code,
};
use crate::conversion::downcast_obj;
use crate::value::Obj;
use crate::{Error, MutObject, NativeFunction, Object, ObjectHandle, Realm, Res, Value, Variable};
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
        locales: Option<Value>,
        options: Option<Value>,
        #[realm] realm: &mut Realm,
    ) -> Res<Self> {
        Self::new(realm)
    }

    #[get("format")]
    fn format(&self, #[realm] realm: &mut Realm) -> Res<String> {
        Ok(String::new())
    }

    #[prop("resolvedOptions")]
    fn resolved_options(&self, #[realm] realm: &mut Realm) -> Res<ObjectHandle> {
        Ok(Object::new(realm))
    }

    pub fn supported_locales_of(
        locales: Value,
        _options: Option<ObjectHandle>,
        #[realm] realm: &mut Realm,
    ) -> Vec<String> {
        Vec::new()
    }
}
