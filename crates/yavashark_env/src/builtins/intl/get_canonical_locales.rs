use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::{MutObject, Realm};

#[object]
#[derive(Debug)]
pub struct GetCanonicalLocales {}

impl GetCanonicalLocales {
    pub fn new(realm: &mut Realm) -> Self {
        Self {
            inner: RefCell::new(MutableGetCanonicalLocales {
                object: MutObject::with_proto(realm.intrinsics.intl_get_canonical_locales.clone()),
            }),
        }
    }
}


#[props]
impl GetCanonicalLocales {
    #[constructor]
    fn construct() {}

}
