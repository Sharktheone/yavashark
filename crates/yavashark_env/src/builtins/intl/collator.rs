use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::{MutObject, Realm};

#[object]
#[derive(Debug)]
pub struct Collator {}

impl Collator {
    pub fn new(realm: &mut Realm) -> Self {
        Self {
            inner: RefCell::new(MutableCollator {
                object: MutObject::with_proto(realm.intrinsics.intl_collator.clone()),
            }),
        }
    }
}


#[props]
impl Collator {
    #[constructor]
    fn construct() {}
}
