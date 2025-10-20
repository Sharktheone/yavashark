use crate::{MutObject, Realm, Res};
use std::cell::RefCell;
use yavashark_macro::{object, props};

#[object]
#[derive(Debug)]
pub struct Locale {}

impl Locale {
    #[allow(unused)]
    pub fn new(realm: &mut Realm) -> Res<Self> {
        Ok(Self {
            inner: RefCell::new(MutableLocale {
                object: MutObject::with_proto(realm.intrinsics.clone_public().intl_locale.get(realm)?.clone()),
            }),
        })
    }
}

#[props(intrinsic_name = intl_locale, to_string_tag = "Intl.Locale")]
impl Locale {
    #[constructor]
    fn construct() {}
}
