use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::{MutObject, Realm};

#[object]
#[derive(Debug)]
pub struct RelativeTimeFormat {}

impl RelativeTimeFormat {
    pub fn new(realm: &mut Realm) -> Self {
        Self {
            inner: RefCell::new(MutableRelativeTimeFormat {
                object: MutObject::with_proto(realm.intrinsics.intl_relative_time_format.clone()),
            }),
        }
    }
}


#[props]
impl RelativeTimeFormat {
    #[constructor]
    fn construct() {}

}
