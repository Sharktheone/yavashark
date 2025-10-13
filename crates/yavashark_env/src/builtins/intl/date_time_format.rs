use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::{MutObject, Realm};

#[object]
#[derive(Debug)]
pub struct DateTimeFormat {}

impl DateTimeFormat {
    pub fn new(realm: &mut Realm) -> Self {
        Self {
            inner: RefCell::new(MutableDateTimeFormat {
                object: MutObject::with_proto(realm.intrinsics.intl_date_time_format.clone()),
            }),
        }
    }
}


#[props]
impl DateTimeFormat {
    #[call_constructor]
    fn construct() {}

}
