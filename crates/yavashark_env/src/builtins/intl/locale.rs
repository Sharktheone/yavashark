use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::{MutObject, Realm};

#[object]
#[derive(Debug)]
pub struct Locale {}

impl Locale {
    pub fn new(realm: &mut Realm) -> Self {
        Self {
            inner: RefCell::new(MutableLocale {
                object: MutObject::with_proto(realm.intrinsics.intl_locale.clone()),
            }),
        }
    }
}


#[props]
impl Locale {
    #[constructor]
    fn construct() {}

}
