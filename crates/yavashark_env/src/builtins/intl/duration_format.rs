use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::{MutObject, Realm};

#[object]
#[derive(Debug)]
pub struct DurationFormat {}

impl DurationFormat {
    pub fn new(realm: &mut Realm) -> Self {
        Self {
            inner: RefCell::new(MutableDurationFormat {
                object: MutObject::with_proto(realm.intrinsics.intl_duration_format.clone()),
            }),
        }
    }
}


#[props]
impl DurationFormat {
    #[constructor]
    fn construct() {}

}
