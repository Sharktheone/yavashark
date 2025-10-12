use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::{MutObject, Realm};

#[object]
#[derive(Debug)]
pub struct SupportedValuesOf {}

impl SupportedValuesOf {
    pub fn new(realm: &mut Realm) -> Self {
        Self {
            inner: RefCell::new(MutableSupportedValuesOf {
                object: MutObject::with_proto(realm.intrinsics.intl_supported_locales_of.clone()),
            }),
        }
    }
}


#[props]
impl SupportedValuesOf {
    #[constructor]
    fn construct() {}

}
