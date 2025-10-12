use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::{MutObject, Realm};

#[object]
#[derive(Debug)]
pub struct ListFormat {}

impl ListFormat {
    pub fn new(realm: &mut Realm) -> Self {
        Self {
            inner: RefCell::new(MutableListFormat {
                object: MutObject::with_proto(realm.intrinsics.intl_list_format.clone()),
            }),
        }
    }
}


#[props]
impl ListFormat {
    #[constructor]
    fn construct() {}

}
