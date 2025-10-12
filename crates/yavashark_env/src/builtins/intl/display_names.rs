use std::cell::RefCell;
use yavashark_macro::{object, props};
use crate::{MutObject, Realm};

#[object]
#[derive(Debug)]
pub struct DisplayNames {}

impl DisplayNames {
    pub fn new(realm: &mut Realm) -> Self {
        Self {
            inner: RefCell::new(MutableDisplayNames {
                object: MutObject::with_proto(realm.intrinsics.intl_display_names.clone()),
            }),
        }
    }
}


#[props]
impl DisplayNames {
    #[constructor]
    fn construct() {}
    
}
