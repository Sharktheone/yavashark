use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::{MutObject, Realm};

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


#[props]
impl PluralRules {
    #[constructor]
    fn construct() {}

}
