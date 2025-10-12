use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::{MutObject, Realm};

#[object]
#[derive(Debug)]
pub struct NumberFormat {}

impl NumberFormat {
    pub fn new(realm: &mut Realm) -> Self {
        Self {
            inner: RefCell::new(MutableNumberFormat {
                object: MutObject::with_proto(realm.intrinsics.intl_number_format.clone()),
            }),
        }
    }
}


#[props]
impl NumberFormat {
    #[constructor]
    fn construct() {}

}
